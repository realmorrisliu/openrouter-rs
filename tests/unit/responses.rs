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
    responses::{
        ResponsesRequest, ResponsesResponse, ResponsesStreamEvent, create_response, stream_response,
    },
};
use serde_json::json;

struct CapturedRequest {
    request_line: String,
    header_text: String,
    body_text: String,
}

fn spawn_server(
    response_body: &str,
    content_type: &str,
) -> (
    String,
    mpsc::Receiver<CapturedRequest>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let body = response_body.to_string();
    let content_type = content_type.to_string();
    let (tx, rx) = mpsc::channel::<CapturedRequest>();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("server should accept one connection");
        let mut request_bytes = Vec::new();
        let mut chunk = [0_u8; 1024];
        let header_end = loop {
            let read = stream.read(&mut chunk).expect("server should read request");
            if read == 0 {
                break None;
            }
            request_bytes.extend_from_slice(&chunk[..read]);
            if let Some(pos) = request_bytes
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
            {
                break Some(pos + 4);
            }
        }
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

        let mut body_bytes = request_bytes[header_end..].to_vec();
        while body_bytes.len() < content_length {
            let read = stream
                .read(&mut chunk)
                .expect("server should read request body");
            if read == 0 {
                break;
            }
            body_bytes.extend_from_slice(&chunk[..read]);
        }
        let body_text = String::from_utf8_lossy(&body_bytes[..content_length]).to_string();
        tx.send(CapturedRequest {
            request_line,
            header_text,
            body_text,
        })
        .expect("server should send request");

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            content_type,
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    (format!("http://{addr}/api/v1"), rx, server)
}

#[test]
fn test_responses_request_serialization() {
    let request = ResponsesRequest::builder()
        .model("openai/gpt-5")
        .input(json!([{
            "role": "user",
            "content": "Hello from responses API"
        }]))
        .instructions("Be concise")
        .metadata([("env", "test"), ("feature", "responses")])
        .tools(vec![json!({
            "type": "function",
            "name": "get_weather",
            "parameters": { "type": "object" }
        })])
        .tool_choice(json!("auto"))
        .parallel_tool_calls(true)
        .models(vec![
            "openai/gpt-5".to_string(),
            "openai/gpt-4o".to_string(),
        ])
        .max_output_tokens(256)
        .temperature(0.2)
        .top_p(0.9)
        .top_logprobs(5)
        .max_tool_calls(2)
        .presence_penalty(0.0)
        .frequency_penalty(0.0)
        .top_k(40.0)
        .image_config([("aspect_ratio", json!("16:9"))])
        .modalities(vec!["text".to_string(), "image".to_string()])
        .prompt_cache_key("cache-key-1")
        .previous_response_id("resp-prev")
        .include(vec!["reasoning.encrypted_content".to_string()])
        .background(false)
        .safety_identifier("user-123")
        .store(false)
        .service_tier("auto")
        .truncation("auto")
        .user("user-123")
        .session_id("session-abc")
        .trace(TraceOptions {
            trace_id: Some("trace-1".to_string()),
            trace_name: None,
            span_name: Some("responses.unit".to_string()),
            generation_name: None,
            parent_span_id: None,
            extra: Default::default(),
        })
        .plugins(vec![Plugin::new("web").option("max_results", 3)])
        .build()
        .expect("responses request should build");

    let value = serde_json::to_value(&request).expect("responses request should serialize");
    assert_eq!(value["model"], "openai/gpt-5");
    assert_eq!(value["instructions"], "Be concise");
    assert_eq!(value["metadata"]["env"], "test");
    assert_eq!(value["tool_choice"], "auto");
    assert_eq!(value["parallel_tool_calls"], true);
    assert_eq!(value["max_output_tokens"], 256);
    assert_eq!(value["modalities"][1], "image");
    assert_eq!(value["plugins"][0]["id"], "web");
    assert_eq!(value["trace"]["trace_id"], "trace-1");
    assert_eq!(value["trace"]["span_name"], "responses.unit");
}

#[test]
fn test_responses_response_deserialization() {
    let raw = r#"{
        "id": "resp-abc123",
        "object": "response",
        "created_at": 1704067200,
        "model": "gpt-4",
        "status": "completed",
        "output": [{
            "type": "message",
            "id": "msg-abc123",
            "status": "completed",
            "role": "assistant",
            "content": [{
                "type": "output_text",
                "text": "Hello!",
                "annotations": []
            }]
        }],
        "usage": {
            "input_tokens": 10,
            "output_tokens": 25,
            "total_tokens": 35
        }
    }"#;

    let response: ResponsesResponse =
        serde_json::from_str(raw).expect("responses payload should deserialize");
    assert_eq!(response.id.as_deref(), Some("resp-abc123"));
    assert_eq!(response.object_type.as_deref(), Some("response"));
    assert_eq!(response.status.as_deref(), Some("completed"));
    assert!(response.output.is_some());
    assert!(response.usage.is_some());
}

#[test]
fn test_responses_stream_event_deserialization() {
    let raw = r#"{
        "type": "response.output_text.delta",
        "sequence_number": 4,
        "delta": "Hello"
    }"#;

    let event: ResponsesStreamEvent =
        serde_json::from_str(raw).expect("stream event should deserialize");
    assert_eq!(event.event_type, "response.output_text.delta");
    assert_eq!(event.sequence_number, Some(4));
    assert_eq!(
        event.data.get("delta").and_then(|value| value.as_str()),
        Some("Hello")
    );
}

#[test]
fn test_responses_stream_event_with_response_payload() {
    let raw = r#"{
        "type": "response.completed",
        "sequence_number": 10,
        "response": {
            "id": "resp-abc123",
            "status": "completed"
        }
    }"#;

    let event: ResponsesStreamEvent =
        serde_json::from_str(raw).expect("stream event with response should deserialize");
    assert_eq!(event.event_type, "response.completed");
    assert_eq!(event.sequence_number, Some(10));
    assert_eq!(
        event
            .data
            .get("response")
            .and_then(|value| value.get("id"))
            .and_then(|value| value.as_str()),
        Some("resp-abc123")
    );
}

#[tokio::test]
async fn test_create_response_sets_stream_false_and_headers() {
    let (base_url, rx, server) = spawn_server(
        r#"{"id":"resp_1","object":"response","status":"completed"}"#,
        "application/json",
    );

    let request = ResponsesRequest::builder()
        .model("openai/gpt-5")
        .input(json!([{"role":"user","content":"hello"}]))
        .build()
        .expect("responses request should build");
    let x_title = Some("openrouter-rs-tests".to_string());
    let http_referer = Some("https://github.com/realmorrisliu/openrouter-rs".to_string());

    let response = create_response(&base_url, "api-key", &x_title, &http_referer, &request)
        .await
        .expect("create response should succeed");
    assert_eq!(response.id.as_deref(), Some("resp_1"));
    assert_eq!(response.status.as_deref(), Some("completed"));

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "POST /api/v1/responses HTTP/1.1");

    let headers_lower = captured.header_text.to_ascii_lowercase();
    assert!(
        headers_lower.contains("authorization: bearer api-key")
            || headers_lower.contains("authorization:bearer api-key"),
        "authorization header should include API key, headers:\n{}",
        captured.header_text
    );
    assert!(
        headers_lower.contains("x-openrouter-title: openrouter-rs-tests")
            || headers_lower.contains("x-openrouter-title:openrouter-rs-tests"),
        "x-openrouter-title header should be present, headers:\n{}",
        captured.header_text
    );
    assert!(
        headers_lower.contains("x-title: openrouter-rs-tests")
            || headers_lower.contains("x-title:openrouter-rs-tests"),
        "x-title header should be present, headers:\n{}",
        captured.header_text
    );
    assert!(
        headers_lower.contains("http-referer: https://github.com/realmorrisliu/openrouter-rs")
            || headers_lower
                .contains("http-referer:https://github.com/realmorrisliu/openrouter-rs"),
        "http-referer header should be present, headers:\n{}",
        captured.header_text
    );

    let request_json: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid JSON");
    assert_eq!(request_json["stream"], false);
    assert_eq!(request_json["model"], "openai/gpt-5");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_stream_response_sets_stream_true_and_parses_sse() {
    let (base_url, rx, server) = spawn_server(
        concat!(
            "data: {\"type\":\"response.output_text.delta\",\"sequence_number\":1,\"delta\":\"Hi\"}\r\n",
            "data: [DONE]\r\n",
            "\r\n"
        ),
        "text/event-stream",
    );

    let request = ResponsesRequest::builder()
        .model("openai/gpt-5")
        .input(json!([{"role":"user","content":"hello"}]))
        .build()
        .expect("responses request should build");
    let x_title = Some("openrouter-rs-tests".to_string());
    let http_referer = Some("https://github.com/realmorrisliu/openrouter-rs".to_string());

    let mut stream = stream_response(&base_url, "api-key", &x_title, &http_referer, &request)
        .await
        .expect("stream response should succeed");
    let mut events = Vec::new();
    while let Some(item) = stream.next().await {
        events.push(item.expect("stream event should parse"));
    }
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "response.output_text.delta");
    assert_eq!(events[0].sequence_number, Some(1));
    assert_eq!(
        events[0].data.get("delta").and_then(|value| value.as_str()),
        Some("Hi")
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "POST /api/v1/responses HTTP/1.1");
    let request_json: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid JSON");
    assert_eq!(request_json["stream"], true);

    server.join().expect("server thread should finish");
}
