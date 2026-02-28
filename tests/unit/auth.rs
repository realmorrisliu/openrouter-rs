use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::api::auth::{self, CodeChallengeMethod, CreateAuthCodeRequest, UsageLimitType};
use serde_json::json;

#[test]
fn test_create_auth_code_request_serialization() {
    let request = CreateAuthCodeRequest::builder()
        .callback_url("https://myapp.com/auth/callback")
        .code_challenge("abc123")
        .code_challenge_method(CodeChallengeMethod::S256)
        .limit(100.0)
        .expires_at("2026-12-31T23:59:59Z")
        .key_label("My Custom Key")
        .usage_limit_type(UsageLimitType::Monthly)
        .spawn_agent("sdk")
        .spawn_cloud("aws")
        .build()
        .expect("request should build");

    let value = serde_json::to_value(request).expect("request should serialize");
    assert_eq!(value["callback_url"], "https://myapp.com/auth/callback");
    assert_eq!(value["code_challenge"], "abc123");
    assert_eq!(value["code_challenge_method"], "S256");
    assert_eq!(value["limit"], 100.0);
    assert_eq!(value["usage_limit_type"], "monthly");
    assert_eq!(value["spawn_agent"], "sdk");
    assert_eq!(value["spawn_cloud"], "aws");
}

#[test]
fn test_code_challenge_method_plain_serialization() {
    let value = serde_json::to_value(CodeChallengeMethod::Plain)
        .expect("CodeChallengeMethod::Plain should serialize");
    assert_eq!(value, json!("plain"));
}

#[test]
fn test_create_auth_code_response_deserialization() {
    let raw = r#"{
        "data": {
            "id": "auth_code_xyz789",
            "app_id": 12345,
            "created_at": "2025-08-24T10:30:00Z"
        }
    }"#;

    let response: openrouter_rs::types::ApiResponse<auth::AuthCodeData> =
        serde_json::from_str(raw).expect("response should deserialize");
    assert_eq!(response.data.id, "auth_code_xyz789");
    assert_eq!(response.data.app_id, 12345.0);
    assert_eq!(response.data.created_at, "2025-08-24T10:30:00Z");
}

#[tokio::test]
async fn test_create_auth_code_uses_auth_keys_code_path() {
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
            .expect("server should send request details");

        let response_body = r#"{"data":{"id":"auth_code_xyz789","app_id":12345,"created_at":"2025-08-24T10:30:00Z"}}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    let request = CreateAuthCodeRequest::builder()
        .callback_url("https://myapp.com/auth/callback")
        .code_challenge("abc123")
        .code_challenge_method(CodeChallengeMethod::S256)
        .build()
        .expect("request should build");

    let base_url = format!("http://{addr}/api/v1");
    let response = auth::create_auth_code(&base_url, "test-key", &request)
        .await
        .expect("create_auth_code should succeed");
    assert_eq!(response.id, "auth_code_xyz789");

    let (request_line, request_body) = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request details");
    assert_eq!(request_line, "POST /api/v1/auth/keys/code HTTP/1.1");
    let body_json: serde_json::Value =
        serde_json::from_str(&request_body).expect("request body should be json");
    assert_eq!(body_json["callback_url"], "https://myapp.com/auth/callback");
    assert_eq!(body_json["code_challenge_method"], "S256");

    server.join().expect("server thread should finish");
}
