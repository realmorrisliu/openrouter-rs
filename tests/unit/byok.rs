use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::byok::{self, ByokKey, CreateByokKeyRequest, UpdateByokKeyRequest},
    types::{ApiResponse, PaginationOptions},
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

fn assert_management_auth_header(request_text: &str) {
    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer mgmt-key")
            || request_lower.contains("authorization:bearer mgmt-key"),
        "authorization header should include management key, request:\n{request_text}"
    );
}

#[test]
fn test_create_byok_key_request_serialization() {
    let request = CreateByokKeyRequest::builder()
        .provider("openai")
        .key("sk-provider")
        .name("Production OpenAI")
        .workspace_id("ws_123")
        .allowed_models(vec!["openai/gpt-5".to_string()])
        .allowed_user_ids(vec!["user_123".to_string()])
        .disabled(false)
        .is_fallback(true)
        .build()
        .expect("create BYOK request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value["provider"], "openai");
    assert_eq!(value["key"], "sk-provider");
    assert_eq!(value["name"], "Production OpenAI");
    assert_eq!(value["workspace_id"], "ws_123");
    assert_eq!(value["allowed_models"], serde_json::json!(["openai/gpt-5"]));
    assert_eq!(value["allowed_user_ids"], serde_json::json!(["user_123"]));
    assert_eq!(value["disabled"], false);
    assert_eq!(value["is_fallback"], true);
}

#[test]
fn test_update_byok_key_request_can_clear_nullable_fields() {
    let request = UpdateByokKeyRequest::builder()
        .clear_name()
        .clear_allowed_models()
        .clear_allowed_user_ids()
        .build()
        .expect("update BYOK request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value.get("name"), Some(&serde_json::Value::Null));
    assert_eq!(value.get("allowed_models"), Some(&serde_json::Value::Null));
    assert_eq!(
        value.get("allowed_user_ids"),
        Some(&serde_json::Value::Null)
    );
}

#[test]
fn test_update_byok_key_request_preserves_empty_allowlists() {
    let request = UpdateByokKeyRequest::builder()
        .allowed_models(Vec::<String>::new())
        .allowed_user_ids(Vec::<String>::new())
        .build()
        .expect("update BYOK request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value.get("allowed_models"), Some(&serde_json::json!([])));
    assert_eq!(value.get("allowed_user_ids"), Some(&serde_json::json!([])));

    let value_after_clear = UpdateByokKeyRequest::builder()
        .clear_allowed_models()
        .clear_allowed_user_ids()
        .allowed_models(["openai/gpt-5"])
        .allowed_user_ids(["user_123"])
        .build()
        .expect("update BYOK request should build");
    let value_after_clear_json =
        serde_json::to_value(&value_after_clear).expect("request should serialize");
    assert_eq!(
        value_after_clear_json.get("allowed_models"),
        Some(&serde_json::json!(["openai/gpt-5"]))
    );
    assert_eq!(
        value_after_clear_json.get("allowed_user_ids"),
        Some(&serde_json::json!(["user_123"]))
    );

    let clear_after_value = UpdateByokKeyRequest::builder()
        .allowed_models(["openai/gpt-5"])
        .allowed_user_ids(["user_123"])
        .clear_allowed_models()
        .clear_allowed_user_ids()
        .build()
        .expect("update BYOK request should build");
    let clear_after_value_json =
        serde_json::to_value(&clear_after_value).expect("request should serialize");
    assert_eq!(
        clear_after_value_json.get("allowed_models"),
        Some(&serde_json::Value::Null)
    );
    assert_eq!(
        clear_after_value_json.get("allowed_user_ids"),
        Some(&serde_json::Value::Null)
    );
}

#[test]
fn test_byok_key_response_deserialization() {
    let raw = r#"{
        "data": {
            "id": "11111111-2222-3333-4444-555555555555",
            "provider": "openai",
            "workspace_id": "550e8400-e29b-41d4-a716-446655440000",
            "label": "sk-...AbCd",
            "name": "Production OpenAI",
            "disabled": false,
            "is_fallback": false,
            "allowed_models": null,
            "allowed_api_key_hashes": ["hash_123"],
            "allowed_user_ids": null,
            "sort_order": 0,
            "created_at": "2025-08-24T10:30:00Z"
        }
    }"#;

    let parsed: ApiResponse<ByokKey> =
        serde_json::from_str(raw).expect("BYOK response should deserialize");
    assert_eq!(parsed.data.provider, "openai");
    assert_eq!(parsed.data.name.as_deref(), Some("Production OpenAI"));
    assert_eq!(
        parsed.data.allowed_api_key_hashes,
        Some(vec!["hash_123".to_string()])
    );
    assert_eq!(parsed.data.allowed_models, None);
}

#[tokio::test]
async fn test_list_byok_keys_path_query_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[],"total_count":0}"#);

    let result = byok::list_byok_keys(
        &base_url,
        "mgmt-key",
        Some(PaginationOptions::with_offset_and_limit(2, 10)),
        Some("ws_123"),
        Some("openai"),
    )
    .await
    .expect("list BYOK keys should succeed");
    assert!(result.data.is_empty());

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/byok?offset=2&limit=10&workspace_id=ws_123&provider=openai HTTP/1.1"
    );
    assert_management_auth_header(&captured.request_text);

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_create_byok_key_path_body_and_auth_header() {
    let response_body = r#"{
        "data": {
            "id": "11111111-2222-3333-4444-555555555555",
            "provider": "openai",
            "workspace_id": "550e8400-e29b-41d4-a716-446655440000",
            "label": "sk-...AbCd",
            "name": "Production OpenAI",
            "disabled": false,
            "is_fallback": false,
            "allowed_models": null,
            "allowed_api_key_hashes": null,
            "allowed_user_ids": null,
            "sort_order": 0,
            "created_at": "2025-08-24T10:30:00Z"
        }
    }"#;
    let (base_url, rx, server) = spawn_json_server(response_body);
    let request = CreateByokKeyRequest::builder()
        .provider("openai")
        .key("sk-provider")
        .name("Production OpenAI")
        .build()
        .expect("create BYOK request should build");

    let created = byok::create_byok_key(&base_url, "mgmt-key", &request)
        .await
        .expect("create BYOK key should succeed");
    assert_eq!(created.provider, "openai");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "POST /api/v1/byok HTTP/1.1");
    assert_management_auth_header(&captured.request_text);
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be JSON");
    assert_eq!(body["provider"], "openai");
    assert_eq!(body["key"], "sk-provider");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_update_delete_byok_key_paths() {
    let key_body = r#"{
        "data": {
            "id": "byok/team 1",
            "provider": "openai",
            "workspace_id": "ws_123",
            "label": "sk-...AbCd",
            "name": "Production OpenAI",
            "disabled": false,
            "is_fallback": false,
            "allowed_models": null,
            "allowed_api_key_hashes": null,
            "allowed_user_ids": null,
            "sort_order": 0,
            "created_at": "2025-08-24T10:30:00Z"
        }
    }"#;

    let (base_url, rx, server) = spawn_json_server(key_body);
    let _ = byok::get_byok_key(&base_url, "mgmt-key", "byok/team 1")
        .await
        .expect("get BYOK key should succeed");
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/byok/byok%2Fteam%201 HTTP/1.1"
    );
    assert_management_auth_header(&captured.request_text);
    server.join().expect("server thread should finish");

    let (base_url, rx, server) = spawn_json_server(key_body);
    let request = UpdateByokKeyRequest::builder()
        .name("Updated OpenAI")
        .disabled(true)
        .build()
        .expect("update BYOK request should build");
    let _ = byok::update_byok_key(&base_url, "mgmt-key", "byok/team 1", &request)
        .await
        .expect("update BYOK key should succeed");
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "PATCH /api/v1/byok/byok%2Fteam%201 HTTP/1.1"
    );
    assert_management_auth_header(&captured.request_text);
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be JSON");
    assert_eq!(body["name"], "Updated OpenAI");
    assert_eq!(body["disabled"], true);
    server.join().expect("server thread should finish");

    let (base_url, rx, server) = spawn_json_server(r#"{"deleted":true}"#);
    let deleted = byok::delete_byok_key(&base_url, "mgmt-key", "byok/team 1")
        .await
        .expect("delete BYOK key should succeed");
    assert!(deleted);
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "DELETE /api/v1/byok/byok%2Fteam%201 HTTP/1.1"
    );
    assert_management_auth_header(&captured.request_text);
    server.join().expect("server thread should finish");
}
