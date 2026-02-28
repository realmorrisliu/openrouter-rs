use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use futures_util::StreamExt;
use openrouter_rs::api::{
    chat::{Plugin, TraceOptions},
    messages::{
        AnthropicContentPart, AnthropicMessage, AnthropicMessagesMetadata,
        AnthropicMessagesRequest, AnthropicMessagesResponse, AnthropicMessagesStreamEvent,
        AnthropicOutputConfig, AnthropicOutputEffort, AnthropicRole, AnthropicSystemPrompt,
        AnthropicSystemTextBlock, AnthropicThinking, AnthropicTool, AnthropicToolChoice,
        stream_messages,
    },
};
use serde_json::json;

#[test]
fn test_anthropic_messages_request_serialization() {
    let request = AnthropicMessagesRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .max_tokens(512)
        .messages(vec![
            AnthropicMessage::user("hello"),
            AnthropicMessage::with_parts(
                AnthropicRole::User,
                vec![AnthropicContentPart::image_url(
                    "https://example.com/image.png",
                )],
            ),
        ])
        .system(AnthropicSystemPrompt::Blocks(vec![
            AnthropicSystemTextBlock::text("You are concise."),
        ]))
        .metadata(AnthropicMessagesMetadata::with_user_id("user-123"))
        .stop_sequences(vec!["<END>".to_string()])
        .temperature(0.2)
        .top_p(0.9)
        .top_k(40)
        .tools(vec![
            AnthropicTool::custom(
                "get_weather",
                "Get weather by city",
                json!({
                    "type": "object",
                    "properties": {
                        "city": { "type": "string" }
                    },
                    "required": ["city"]
                }),
            ),
            AnthropicTool::hosted("web_search_20250305", "web_search").option("max_uses", 2),
        ])
        .tool_choice(AnthropicToolChoice::auto())
        .thinking(AnthropicThinking::enabled(1024))
        .service_tier("auto")
        .plugins(vec![Plugin::new("web").option("max_results", 3)])
        .route("fallback")
        .user("user-123")
        .session_id("session-abc")
        .trace(TraceOptions {
            trace_id: Some("trace-1".to_string()),
            trace_name: None,
            span_name: Some("messages.unit".to_string()),
            generation_name: None,
            parent_span_id: None,
            extra: Default::default(),
        })
        .models(vec!["anthropic/claude-sonnet-4".to_string()])
        .output_config(AnthropicOutputConfig::with_effort(
            AnthropicOutputEffort::Medium,
        ))
        .build()
        .expect("messages request should build");

    let value = serde_json::to_value(&request).expect("messages request should serialize");

    assert_eq!(value["model"], "anthropic/claude-sonnet-4");
    assert_eq!(value["max_tokens"], 512);
    assert_eq!(value["messages"][0]["role"], "user");
    assert_eq!(value["messages"][0]["content"], "hello");
    assert_eq!(value["system"][0]["type"], "text");
    assert_eq!(value["metadata"]["user_id"], "user-123");
    assert_eq!(value["tools"][0]["name"], "get_weather");
    assert_eq!(value["tools"][1]["type"], "web_search_20250305");
    assert_eq!(value["tool_choice"]["type"], "auto");
    assert_eq!(value["thinking"]["type"], "enabled");
    assert_eq!(value["thinking"]["budget_tokens"], 1024);
    assert_eq!(value["output_config"]["effort"], "medium");
    assert_eq!(value["plugins"][0]["id"], "web");
}

#[test]
fn test_anthropic_messages_response_deserialization() {
    let raw = r#"{
        "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "Hello there.",
                "citations": null
            }
        ],
        "model": "claude-sonnet-4-5-20250929",
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {
            "input_tokens": 12,
            "output_tokens": 15,
            "service_tier": "standard"
        }
    }"#;

    let response: AnthropicMessagesResponse =
        serde_json::from_str(raw).expect("messages response should deserialize");
    assert_eq!(response.id.as_deref(), Some("msg_01XFDUDYJgAACzvnptvVoYEL"));
    assert_eq!(response.object_type.as_deref(), Some("message"));
    assert_eq!(response.role.as_deref(), Some("assistant"));
    assert_eq!(response.stop_reason.as_deref(), Some("end_turn"));
    assert_eq!(
        response
            .usage
            .as_ref()
            .and_then(|usage| usage.output_tokens)
            .unwrap_or_default(),
        15
    );
    assert_eq!(response.content.len(), 1);
    match &response.content[0] {
        AnthropicContentPart::Text { text, .. } => assert_eq!(text, "Hello there."),
        _ => panic!("expected text content block"),
    }
}

#[test]
fn test_anthropic_messages_stream_event_deserialization() {
    let raw = r#"{
        "type": "content_block_delta",
        "index": 0,
        "delta": {
            "type": "text_delta",
            "text": "Hello"
        }
    }"#;

    let event: AnthropicMessagesStreamEvent =
        serde_json::from_str(raw).expect("stream event should deserialize");
    assert_eq!(event.event_type(), "content_block_delta");
    match event {
        AnthropicMessagesStreamEvent::ContentBlockDelta { index, delta } => {
            assert_eq!(index, 0);
            assert_eq!(delta["type"], "text_delta");
            assert_eq!(delta["text"], "Hello");
        }
        _ => panic!("expected content_block_delta"),
    }
}

#[tokio::test]
async fn test_stream_messages_parses_event_and_data_lines() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");

    let (tx, rx) = mpsc::channel::<(String, String)>();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("server should accept one connection");

        let mut request_bytes = Vec::new();
        let mut chunk = [0_u8; 1024];

        loop {
            let read = stream.read(&mut chunk).expect("server should read request");
            if read == 0 {
                break;
            }
            request_bytes.extend_from_slice(&chunk[..read]);
            if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let header_end = request_bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|idx| idx + 4)
            .expect("request should contain header terminator");

        let header_text = String::from_utf8_lossy(&request_bytes[..header_end]).to_string();
        let request_line = header_text.lines().next().unwrap_or_default().to_string();

        let content_length = header_text
            .lines()
            .find_map(|line| {
                let lower = line.to_ascii_lowercase();
                if lower.starts_with("content-length:") {
                    line.split(':').nth(1)?.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let mut body = request_bytes[header_end..].to_vec();
        while body.len() < content_length {
            let read = stream
                .read(&mut chunk)
                .expect("server should read request body");
            if read == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..read]);
        }
        let body_text = String::from_utf8_lossy(&body[..content_length]).to_string();
        tx.send((request_line, body_text))
            .expect("server should send request info");

        let response_body = concat!(
            "event: message_start\r\n",
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"anthropic/claude-sonnet-4\",\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\r\n",
            "\r\n",
            "event: content_block_delta\r\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}\r\n",
            "\r\n",
            "data: [DONE]\r\n",
            "\r\n"
        );

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    let request = AnthropicMessagesRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .max_tokens(128)
        .messages(vec![AnthropicMessage::user("hello")])
        .build()
        .expect("messages request should build");

    let base_url = format!("http://{addr}/api/v1");
    let mut stream = stream_messages(&base_url, "test-key", &None, &None, &request)
        .await
        .expect("stream_messages should succeed");

    let mut events = Vec::new();
    while let Some(item) = stream.next().await {
        events.push(item.expect("stream item should parse"));
    }

    let (request_line, request_body) = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request details");

    assert_eq!(request_line, "POST /api/v1/messages HTTP/1.1");
    let request_json: serde_json::Value =
        serde_json::from_str(&request_body).expect("request body should be valid JSON");
    assert_eq!(request_json["stream"], true);

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event, "message_start");
    assert_eq!(events[1].event, "content_block_delta");
    match &events[1].data {
        AnthropicMessagesStreamEvent::ContentBlockDelta { index, delta } => {
            assert_eq!(*index, 0);
            assert_eq!(delta["text"], "Hi");
        }
        _ => panic!("expected content_block_delta"),
    }

    server.join().expect("server thread should finish");
}
