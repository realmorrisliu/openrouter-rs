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
