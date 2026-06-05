use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::api::generation;
use openrouter_rs::types::ApiResponse;

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

        let request_text = String::from_utf8_lossy(&request_bytes).to_string();
        let request_line = request_text.lines().next().unwrap_or_default().to_string();

        tx.send(CapturedRequest {
            request_line,
            request_text,
        })
        .expect("server should send captured request");

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

#[tokio::test]
async fn test_get_generation_path_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"gen_123","total_cost":0.1,"created_at":"2026-03-02T00:00:00Z","model":"openai/gpt-4o-mini","origin":"chat","usage":42.0,"is_byok":false}}"#,
    );

    let response = generation::get_generation(&base_url, "api-key", "gen_123")
        .await
        .expect("get generation should succeed");
    assert_eq!(response.id, "gen_123");
    assert_eq!(response.model, "openai/gpt-4o-mini");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/generation?id=gen_123 HTTP/1.1"
    );

    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include API key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_generation_content_path_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"input":{"prompt":"What is the meaning of life?"},"output":{"completion":"42"}}}"#,
    );

    let response = generation::get_generation_content(&base_url, "api-key", "gen_123")
        .await
        .expect("get generation content should succeed");
    assert_eq!(response.input["prompt"], "What is the meaning of life?");
    assert_eq!(response.output["completion"], "42");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/generation/content?id=gen_123 HTTP/1.1"
    );

    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include API key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_generation_response_deserializes_num_fetches() {
    let raw = r#"{
        "data": {
            "id": "gen_123",
            "total_cost": 0.1,
            "created_at": "2026-04-22T00:00:00Z",
            "model": "openai/gpt-4o-mini",
            "origin": "chat",
            "usage": 42.0,
            "is_byok": false,
            "native_tokens_reasoning": 8,
            "num_fetches": 3,
            "preset_id": "preset_123",
            "response_cache_source_id": "gen_original",
            "service_tier": "priority"
        }
    }"#;

    let parsed: ApiResponse<generation::GenerationData> =
        serde_json::from_str(raw).expect("generation response should deserialize");

    assert_eq!(parsed.data.id, "gen_123");
    assert_eq!(parsed.data.native_tokens_reasoning, Some(8));
    assert_eq!(parsed.data.num_fetches, Some(3));
    assert_eq!(parsed.data.preset_id.as_deref(), Some("preset_123"));
    assert_eq!(
        parsed.data.response_cache_source_id.as_deref(),
        Some("gen_original")
    );
    assert_eq!(parsed.data.service_tier.as_deref(), Some("priority"));
}

#[test]
fn test_generation_response_accepts_stt_origin() {
    let raw = r#"{
        "data": {
            "id": "gen_stt",
            "total_cost": 0.02,
            "created_at": "2026-04-29T00:00:00Z",
            "model": "openai/whisper-1",
            "origin": "stt",
            "usage": 12.0,
            "is_byok": false
        }
    }"#;

    let parsed: ApiResponse<generation::GenerationData> =
        serde_json::from_str(raw).expect("generation response should deserialize");

    assert_eq!(parsed.data.id, "gen_stt");
    assert_eq!(parsed.data.origin, "stt");
}
