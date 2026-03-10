use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::api_keys::{self, ApiKeyDetails},
    types::PaginationOptions,
};

struct CapturedRequest {
    request_line: String,
    request_text: String,
    body_text: String,
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
        let request_text = format!("{header_text}{body_text}");
        tx.send(CapturedRequest {
            request_line,
            request_text,
            body_text,
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

#[test]
fn test_api_key_details_deserializes_official_provisioning_field() {
    let raw = r#"{
        "label":"default",
        "usage":1.5,
        "is_free_tier":false,
        "is_provisioning_key":true,
        "rate_limit":{"requests":1000,"interval":"1m"},
        "limit":100.0,
        "limit_remaining":98.5
    }"#;

    let parsed: ApiKeyDetails =
        serde_json::from_str(raw).expect("api key details should deserialize");
    assert!(parsed.is_management_key);
}

#[test]
fn test_api_key_details_deserializes_management_alias_field() {
    let raw = r#"{
        "label":"default",
        "usage":1.5,
        "is_free_tier":false,
        "is_management_key":true,
        "rate_limit":{"requests":1000,"interval":"1m"},
        "limit":100.0,
        "limit_remaining":98.5
    }"#;

    let parsed: ApiKeyDetails =
        serde_json::from_str(raw).expect("api key details should deserialize from alias");
    assert!(parsed.is_management_key);
}

#[test]
fn test_api_key_details_deserializes_when_both_management_and_legacy_fields_exist() {
    let raw = r#"{
        "label":"default",
        "usage":1.5,
        "is_free_tier":false,
        "is_management_key":true,
        "is_provisioning_key":true,
        "rate_limit":{"requests":1000,"interval":"1m"},
        "limit":100.0,
        "limit_remaining":98.5
    }"#;

    let parsed: ApiKeyDetails = serde_json::from_str(raw)
        .expect("api key details should deserialize when both fields exist");
    assert!(parsed.is_management_key);
}

#[tokio::test]
async fn test_get_current_api_key_path_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"label":"default","usage":1.5,"is_free_tier":false,"is_management_key":true,"rate_limit":{"requests":1000,"interval":"1m"},"limit":100.0,"limit_remaining":98.5}}"#,
    );

    let details = api_keys::get_current_api_key(&base_url, "api-key")
        .await
        .expect("get current key should succeed");
    assert_eq!(details.label, "default");
    assert!(details.is_management_key);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/key HTTP/1.1");

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
async fn test_delete_api_key_uses_single_api_v1_prefix() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<String>();

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

        let request_line = String::from_utf8_lossy(&request_bytes)
            .lines()
            .next()
            .unwrap_or_default()
            .to_string();
        tx.send(request_line)
            .expect("server should send request line");

        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        stream
            .write_all(response)
            .expect("server should write response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let deleted = api_keys::delete_api_key(&base_url, "test-key", "key-hash")
        .await
        .expect("delete request should succeed");

    assert!(deleted, "delete endpoint should return true on success");

    let request_line = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request line");
    assert_eq!(request_line, "DELETE /api/v1/keys/key-hash HTTP/1.1");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_api_keys_serializes_pagination_query() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<String>();

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

        let request_line = String::from_utf8_lossy(&request_bytes)
            .lines()
            .next()
            .unwrap_or_default()
            .to_string();
        tx.send(request_line)
            .expect("server should send request line");

        let response = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"data\":[]}";
        stream
            .write_all(response)
            .expect("server should write response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let keys = api_keys::list_api_keys(
        &base_url,
        "mgmt-key",
        Some(PaginationOptions::with_offset(5)),
        Some(true),
    )
    .await
    .expect("list API keys should succeed");
    assert!(keys.is_empty(), "response payload should parse");

    let request_line = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request line");
    assert_eq!(
        request_line,
        "GET /api/v1/keys?offset=5&include_disabled=true HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_create_api_key_sends_path_body_and_auth_header() {
    let (base_url, rx, server) =
        spawn_json_server(r#"{"data":{"hash":"hash_123","name":"new-key","disabled":false}}"#);

    let created = api_keys::create_api_key(&base_url, "mgmt-key", "new-key", Some(25.0))
        .await
        .expect("create API key should succeed");
    assert_eq!(created.hash.as_deref(), Some("hash_123"));
    assert_eq!(created.name.as_deref(), Some("new-key"));

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "POST /api/v1/keys HTTP/1.1");
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer mgmt-key")
            || request_lower.contains("authorization:bearer mgmt-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );

    let request_json: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid JSON");
    assert_eq!(request_json["name"], "new-key");
    assert_eq!(request_json["limit"], 25.0);

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_api_key_sends_path_and_auth_header() {
    let (base_url, rx, server) =
        spawn_json_server(r#"{"data":{"hash":"hash_123","name":"new-key","disabled":false}}"#);

    let fetched = api_keys::get_api_key(&base_url, "mgmt-key", "hash_123")
        .await
        .expect("get API key should succeed");
    assert_eq!(fetched.hash.as_deref(), Some("hash_123"));

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/keys/hash_123 HTTP/1.1");
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer mgmt-key")
            || request_lower.contains("authorization:bearer mgmt-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_update_api_key_sends_path_body_and_auth_header() {
    let (base_url, rx, server) =
        spawn_json_server(r#"{"data":{"hash":"hash_123","name":"renamed-key","disabled":true}}"#);

    let updated = api_keys::update_api_key(
        &base_url,
        "mgmt-key",
        "hash_123",
        Some("renamed-key".to_string()),
        Some(true),
        Some(99.0),
    )
    .await
    .expect("update API key should succeed");
    assert_eq!(updated.name.as_deref(), Some("renamed-key"));

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "PATCH /api/v1/keys/hash_123 HTTP/1.1"
    );
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer mgmt-key")
            || request_lower.contains("authorization:bearer mgmt-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );

    let request_json: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid JSON");
    assert_eq!(request_json["name"], "renamed-key");
    assert_eq!(request_json["disabled"], true);
    assert_eq!(request_json["limit"], 99.0);

    server.join().expect("server thread should finish");
}
