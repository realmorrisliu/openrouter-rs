use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use futures_util::StreamExt;
use openrouter_rs::{OpenRouterClient, api::chat, types::Role};

struct CapturedRequest {
    request_line: String,
    request_text: String,
}

fn spawn_json_server(
    response_body: &str,
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
        .expect("request should include header terminator");

        let header_text = String::from_utf8_lossy(&request_bytes[..header_end]).to_string();
        let request_line = header_text.lines().next().unwrap_or_default().to_string();

        tx.send(CapturedRequest {
            request_line,
            request_text: header_text,
        })
        .expect("captured request should send");

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    (format!("http://{addr}/api/v1"), rx, server)
}

fn spawn_sse_server(
    response_body: &str,
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
        .expect("request should include header terminator");

        let header_text = String::from_utf8_lossy(&request_bytes[..header_end]).to_string();
        let request_line = header_text.lines().next().unwrap_or_default().to_string();

        tx.send(CapturedRequest {
            request_line,
            request_text: header_text,
        })
        .expect("captured request should send");

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    (format!("http://{addr}/api/v1"), rx, server)
}

fn chat_response_json() -> &'static str {
    r#"{
        "id": "gen-123",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "ok"
            }
        }],
        "created": 1700000000,
        "model": "openai/gpt-4.1-mini",
        "object": "chat.completion"
    }"#
}

fn build_chat_request() -> chat::ChatCompletionRequest {
    chat::ChatCompletionRequest::builder()
        .model("openai/gpt-4.1-mini")
        .messages(vec![chat::Message::new(Role::User, "hello")])
        .build()
        .expect("chat request should build")
}

#[tokio::test]
async fn test_default_x_title_header_is_sent() {
    let (base_url, rx, server) = spawn_json_server(chat_response_json());
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("api-key")
        .build()
        .expect("client should build");

    let request = build_chat_request();
    let _response = client
        .chat()
        .create(&request)
        .await
        .expect("chat request should succeed");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/chat/completions HTTP/1.1"
    );
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("x-title: openrouter-rs")
            || request_lower.contains("x-title:openrouter-rs"),
        "default x-title header should be present, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_explicit_x_title_overrides_default() {
    let (base_url, rx, server) = spawn_json_server(chat_response_json());
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("api-key")
        .x_title("openrouter-rs-tests")
        .build()
        .expect("client should build");

    let request = build_chat_request();
    let _response = client
        .chat()
        .create(&request)
        .await
        .expect("chat request should succeed");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/chat/completions HTTP/1.1"
    );
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("x-title: openrouter-rs-tests")
            || request_lower.contains("x-title:openrouter-rs-tests"),
        "explicit x-title header should be present, request:\n{}",
        captured.request_text
    );
    assert!(
        !request_lower.contains("x-title: openrouter-rs\r\n")
            && !request_lower.contains("x-title:openrouter-rs\r\n"),
        "default x-title should be replaced when custom title is set, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_default_x_title_header_is_sent_for_streaming_chat() {
    let (base_url, rx, server) = spawn_sse_server(concat!(
        "data: {\"id\":\"gen-stream-001\",\"choices\":[{\"delta\":{\"role\":\"assistant\",\"content\":\"Hello\"},\"index\":0}],\"created\":1700000000,\"model\":\"test-model\",\"object\":\"chat.completion.chunk\"}\r\n",
        "\r\n",
        "data: [DONE]\r\n",
        "\r\n"
    ));
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("api-key")
        .build()
        .expect("client should build");

    let request = build_chat_request();
    let mut stream = client
        .chat()
        .stream(&request)
        .await
        .expect("chat stream should succeed");
    while stream.next().await.is_some() {}

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/chat/completions HTTP/1.1"
    );
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("x-title: openrouter-rs")
            || request_lower.contains("x-title:openrouter-rs"),
        "default x-title header should be present for stream, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}
