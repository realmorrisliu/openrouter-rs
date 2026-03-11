use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use futures_util::StreamExt;
use openrouter_rs::{
    api::chat::{
        ChatCompletionRequestBuilder, Message, send_chat_completion, stream_chat_completion,
    },
    types::Role,
};

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

#[tokio::test]
async fn test_send_chat_completion_sets_stream_false_and_headers() {
    let (base_url, rx, server) = spawn_server(
        r#"{"id":"gen-123","choices":[{"message":{"role":"assistant","content":"Hello from mock"}}],"created":1700000000,"model":"test-model","object":"chat.completion"}"#,
        "application/json",
    );

    let request = ChatCompletionRequestBuilder::default()
        .model("openai/gpt-4o-mini")
        .messages(vec![Message::new(Role::User, "hello")])
        .build()
        .expect("chat request should build");
    let x_title = Some("openrouter-rs-tests".to_string());
    let http_referer = Some("https://github.com/realmorrisliu/openrouter-rs".to_string());

    let response = send_chat_completion(&base_url, "api-key", &x_title, &http_referer, &request)
        .await
        .expect("send_chat_completion should succeed");
    assert_eq!(response.id, "gen-123");
    assert_eq!(response.choices[0].content(), Some("Hello from mock"));

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/chat/completions HTTP/1.1"
    );

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
    assert_eq!(request_json["model"], "openai/gpt-4o-mini");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_stream_chat_completion_sets_stream_true_and_parses_sse() {
    let (base_url, rx, server) = spawn_server(
        concat!(
            "data: {\"id\":\"gen-stream-001\",\"choices\":[{\"delta\":{\"role\":\"assistant\",\"content\":\"Hello\"},\"index\":0}],\"created\":1700000000,\"model\":\"test-model\",\"object\":\"chat.completion.chunk\"}\r\n",
            "\r\n",
            "data: [DONE]\r\n",
            "\r\n"
        ),
        "text/event-stream",
    );

    let request = ChatCompletionRequestBuilder::default()
        .model("openai/gpt-4o-mini")
        .messages(vec![Message::new(Role::User, "hello")])
        .build()
        .expect("chat request should build");

    let x_title = Some("openrouter-rs-tests".to_string());
    let http_referer = Some("https://github.com/realmorrisliu/openrouter-rs".to_string());

    let mut stream = stream_chat_completion(&base_url, "api-key", &x_title, &http_referer, &request)
        .await
        .expect("stream_chat_completion should succeed");
    let mut chunks = Vec::new();
    while let Some(item) = stream.next().await {
        chunks.push(item.expect("stream chunk should parse"));
    }
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].id, "gen-stream-001");
    assert_eq!(chunks[0].choices[0].content(), Some("Hello"));

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/chat/completions HTTP/1.1"
    );

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
    assert_eq!(request_json["stream"], true);

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_stream_chat_completion_parses_multiline_sse_data_frames() {
    let (base_url, _rx, server) = spawn_server(
        concat!(
            ": keep-alive\r\n",
            "event: message\r\n",
            "data: {\r\n",
            "data:   \"id\":\"gen-stream-002\",\r\n",
            "data:   \"choices\":[{\"delta\":{\"role\":\"assistant\",\"content\":\"Hello again\"},\"index\":0}],\r\n",
            "data:   \"created\":1700000001,\r\n",
            "data:   \"model\":\"test-model\",\r\n",
            "data:   \"object\":\"chat.completion.chunk\"\r\n",
            "data: }\r\n",
            "\r\n",
            "data: [DONE]\r\n",
            "\r\n"
        ),
        "text/event-stream",
    );

    let request = ChatCompletionRequestBuilder::default()
        .model("openai/gpt-4o-mini")
        .messages(vec![Message::new(Role::User, "hello")])
        .build()
        .expect("chat request should build");

    let mut stream = stream_chat_completion(&base_url, "api-key", &None, &None, &request)
        .await
        .expect("stream_chat_completion should succeed");
    let mut chunks = Vec::new();
    while let Some(item) = stream.next().await {
        chunks.push(item.expect("stream chunk should parse"));
    }

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].id, "gen-stream-002");
    assert_eq!(chunks[0].choices[0].content(), Some("Hello again"));

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_send_chat_completion_returns_contextual_parse_error_on_invalid_json() {
    let (base_url, _rx, server) = spawn_server("{\"id\":\"broken\"", "application/json");

    let request = ChatCompletionRequestBuilder::default()
        .model("openai/gpt-4o-mini")
        .messages(vec![Message::new(Role::User, "hello")])
        .build()
        .expect("chat request should build");

    let error = send_chat_completion(&base_url, "api-key", &None, &None, &request)
        .await
        .expect_err("invalid JSON should fail");

    match error {
        openrouter_rs::error::OpenRouterError::Unknown(message) => {
            assert!(message.contains("Failed to deserialize chat completion response"));
            assert!(message.contains("body preview"));
            assert!(message.contains("{\"id\":\"broken\""));
        }
        other => panic!("expected contextual parse error, got {other:?}"),
    }

    server.join().expect("server thread should finish");
}
