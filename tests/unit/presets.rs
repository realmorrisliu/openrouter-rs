use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::{
        chat::{self, Message},
        messages::{self, AnthropicMessage},
        presets::{self, PresetWithDesignatedVersion},
        responses,
    },
    types::Role,
};
use serde_json::json;

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
        .expect("server should send request");

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

fn preset_response_body() -> &'static str {
    r#"{
        "data": {
            "id": "preset_1",
            "creator_user_id": "user_1",
            "workspace_id": "ws_1",
            "name": "my preset",
            "slug": "my-preset",
            "description": null,
            "status": "active",
            "designated_version_id": "version_1",
            "created_at": "2026-04-20T10:00:00Z",
            "updated_at": "2026-04-20T10:00:00Z",
            "status_updated_at": null,
            "designated_version": {
                "id": "version_1",
                "preset_id": "preset_1",
                "creator_id": "user_1",
                "version": 1,
                "system_prompt": "You are concise.",
                "config": {
                    "model": "openai/gpt-5",
                    "temperature": 0.7
                },
                "created_at": "2026-04-20T10:00:00Z",
                "updated_at": "2026-04-20T10:00:00Z"
            }
        }
    }"#
}

#[test]
fn test_preset_response_deserializes() {
    let parsed: serde_json::Value =
        serde_json::from_str(preset_response_body()).expect("response should parse as JSON");
    let preset: PresetWithDesignatedVersion =
        serde_json::from_value(parsed["data"].clone()).expect("preset should deserialize");

    assert_eq!(preset.id, "preset_1");
    assert_eq!(preset.status, "active");
    let version = preset
        .designated_version
        .expect("designated version should be present");
    assert_eq!(version.version, 1);
    assert_eq!(
        version.config.get("model").and_then(|value| value.as_str()),
        Some("openai/gpt-5")
    );
}

#[test]
fn test_list_presets_and_versions_deserialize() {
    let presets_raw = r#"{
        "data": [{
            "id": "preset_1",
            "creator_user_id": "user_1",
            "workspace_id": "ws_1",
            "name": "my preset",
            "slug": "my-preset",
            "description": null,
            "status": "active",
            "designated_version_id": "version_1",
            "created_at": "2026-04-20T10:00:00Z",
            "updated_at": "2026-04-20T10:00:00Z",
            "status_updated_at": null
        }],
        "total_count": 1
    }"#;
    let presets: presets::ListPresetsResponse =
        serde_json::from_str(presets_raw).expect("preset list should deserialize");
    assert_eq!(presets.total_count, 1);
    assert_eq!(presets.data[0].slug, "my-preset");

    let versions_raw = r#"{
        "data": [{
            "id": "version_1",
            "preset_id": "preset_1",
            "creator_id": "user_1",
            "version": 1,
            "system_prompt": null,
            "config": {"model":"openai/gpt-5"},
            "created_at": "2026-04-20T10:00:00Z",
            "updated_at": "2026-04-20T10:00:00Z"
        }],
        "total_count": 1
    }"#;
    let versions: presets::ListPresetVersionsResponse =
        serde_json::from_str(versions_raw).expect("preset version list should deserialize");
    assert_eq!(versions.data[0].version, 1);
}

#[tokio::test]
async fn test_create_chat_completion_preset_path_body_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(preset_response_body());
    let request = chat::ChatCompletionRequest::builder()
        .model("openai/gpt-5")
        .messages(vec![Message::new(Role::User, "hello")])
        .temperature(0.7)
        .build()
        .expect("chat request should build");

    let preset =
        presets::create_chat_completion_preset(&base_url, "management-key", "my preset", &request)
            .await
            .expect("create chat preset should succeed");
    assert_eq!(preset.slug, "my-preset");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/presets/my%20preset/chat/completions HTTP/1.1"
    );
    assert!(
        captured
            .request_text
            .to_ascii_lowercase()
            .contains("authorization: bearer management-key")
            || captured
                .request_text
                .to_ascii_lowercase()
                .contains("authorization:bearer management-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be JSON");
    assert_eq!(body["model"], "openai/gpt-5");
    assert_eq!(body["temperature"], 0.7);

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_create_response_preset_path_body_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(preset_response_body());
    let request = responses::ResponsesRequest::builder()
        .model("openai/gpt-5")
        .input(json!("hello"))
        .instructions("You are concise.")
        .build()
        .expect("responses request should build");

    let preset =
        presets::create_response_preset(&base_url, "management-key", "my-preset", &request)
            .await
            .expect("create response preset should succeed");
    assert_eq!(preset.id, "preset_1");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/presets/my-preset/responses HTTP/1.1"
    );
    assert!(
        captured
            .request_text
            .to_ascii_lowercase()
            .contains("authorization: bearer management-key")
            || captured
                .request_text
                .to_ascii_lowercase()
                .contains("authorization:bearer management-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be JSON");
    assert_eq!(body["model"], "openai/gpt-5");
    assert_eq!(body["instructions"], "You are concise.");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_create_message_preset_path_body_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(preset_response_body());
    let request = messages::AnthropicMessagesRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .max_tokens(128)
        .messages(vec![AnthropicMessage::user("hello")])
        .build()
        .expect("messages request should build");

    let preset = presets::create_message_preset(&base_url, "management-key", "my-preset", &request)
        .await
        .expect("create message preset should succeed");
    assert_eq!(preset.id, "preset_1");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/presets/my-preset/messages HTTP/1.1"
    );
    assert!(
        captured
            .request_text
            .to_ascii_lowercase()
            .contains("authorization: bearer management-key")
            || captured
                .request_text
                .to_ascii_lowercase()
                .contains("authorization:bearer management-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be JSON");
    assert_eq!(body["model"], "anthropic/claude-sonnet-4");
    assert_eq!(body["max_tokens"], 128);

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_and_get_presets_paths_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[],"total_count":0}"#);
    let presets = presets::list_presets(
        &base_url,
        "management-key",
        Some(openrouter_rs::types::PaginationOptions::with_offset_and_limit(5, 10)),
    )
    .await
    .expect("list presets should succeed");
    assert_eq!(presets.total_count, 0);
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture list request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/presets?offset=5&limit=10 HTTP/1.1"
    );
    assert!(
        captured
            .request_text
            .to_ascii_lowercase()
            .contains("authorization: bearer management-key")
            || captured
                .request_text
                .to_ascii_lowercase()
                .contains("authorization:bearer management-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );
    server.join().expect("list server thread should finish");

    let (base_url, rx, server) = spawn_json_server(preset_response_body());
    let preset = presets::get_preset(&base_url, "management-key", "my preset")
        .await
        .expect("get preset should succeed");
    assert_eq!(preset.id, "preset_1");
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture get request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/presets/my%20preset HTTP/1.1"
    );
    server.join().expect("get server thread should finish");
}

#[tokio::test]
async fn test_list_and_get_preset_versions_paths() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[],"total_count":0}"#);
    let versions = presets::list_preset_versions(
        &base_url,
        "management-key",
        "my preset",
        Some(openrouter_rs::types::PaginationOptions::with_limit(25)),
    )
    .await
    .expect("list preset versions should succeed");
    assert_eq!(versions.total_count, 0);
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture versions request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/presets/my%20preset/versions?limit=25 HTTP/1.1"
    );
    server.join().expect("versions server thread should finish");

    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"version_1","preset_id":"preset_1","creator_id":"user_1","version":1,"system_prompt":null,"config":{"model":"openai/gpt-5"},"created_at":"2026-04-20T10:00:00Z","updated_at":"2026-04-20T10:00:00Z"}}"#,
    );
    let version = presets::get_preset_version(&base_url, "management-key", "my preset", "1")
        .await
        .expect("get preset version should succeed");
    assert_eq!(version.version, 1);
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture version request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/presets/my%20preset/versions/1 HTTP/1.1"
    );
    server.join().expect("version server thread should finish");
}
