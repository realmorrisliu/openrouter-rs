use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::observability::{
        self, CreateObservabilityDestinationRequest, ObservabilityDestination,
        ObservabilityFilterGroup, ObservabilityFilterRule, ObservabilityFilterRulesConfig,
        UpdateObservabilityDestinationRequest,
    },
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

fn destination_body() -> &'static str {
    r#"{
        "data": {
            "id": "99999999-aaaa-bbbb-cccc-dddddddddddd",
            "workspace_id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Production Langfuse",
            "enabled": true,
            "privacy_mode": false,
            "sampling_rate": 1.0,
            "api_key_hashes": null,
            "filter_rules": null,
            "created_at": "2025-08-24T10:30:00Z",
            "updated_at": "2025-08-24T15:45:00Z",
            "type": "langfuse",
            "config": {
                "baseUrl": "https://us.cloud.langfuse.com",
                "publicKey": "pk-l...EfGh",
                "secretKey": "sk-l...AbCd"
            }
        }
    }"#
}

#[test]
fn test_create_observability_destination_request_serialization() {
    let filter_rules = ObservabilityFilterRulesConfig::builder()
        .enabled(true)
        .groups(vec![
            ObservabilityFilterGroup::builder()
                .logic("and")
                .rules(vec![
                    ObservabilityFilterRule::builder()
                        .field("model")
                        .operator("equals")
                        .value(serde_json::json!("openai/gpt-5"))
                        .build()
                        .expect("filter rule should build"),
                ])
                .build()
                .expect("filter group should build"),
        ])
        .build()
        .expect("filter rules should build");

    let request = CreateObservabilityDestinationRequest::builder()
        .destination_type("langfuse")
        .name("Production Langfuse")
        .config(serde_json::json!({
            "baseUrl": "https://us.cloud.langfuse.com",
            "publicKey": "pk-l...EfGh",
            "secretKey": "sk-l...AbCd"
        }))
        .workspace_id("ws_123")
        .api_key_hashes(vec!["hash_123".to_string()])
        .enabled(true)
        .privacy_mode(false)
        .sampling_rate(0.5)
        .filter_rules(filter_rules)
        .build()
        .expect("create observability destination request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value["type"], "langfuse");
    assert_eq!(value["name"], "Production Langfuse");
    assert_eq!(value["workspace_id"], "ws_123");
    assert_eq!(value["api_key_hashes"], serde_json::json!(["hash_123"]));
    assert_eq!(value["enabled"], true);
    assert_eq!(value["privacy_mode"], false);
    assert_eq!(value["sampling_rate"], 0.5);
    assert_eq!(
        value["filter_rules"]["groups"][0]["rules"][0]["field"],
        "model"
    );
}

#[test]
fn test_observability_destination_response_deserialization() {
    let parsed: ApiResponse<ObservabilityDestination> =
        serde_json::from_str(destination_body()).expect("destination response should deserialize");
    assert_eq!(parsed.data.destination_type, "langfuse");
    assert_eq!(parsed.data.name.as_deref(), Some("Production Langfuse"));
    assert_eq!(parsed.data.config["publicKey"], "pk-l...EfGh");
    assert_eq!(parsed.data.api_key_hashes, None);
}

#[tokio::test]
async fn test_list_observability_destinations_path_query_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[],"total_count":0}"#);

    let result = observability::list_observability_destinations(
        &base_url,
        "mgmt-key",
        Some(PaginationOptions::with_offset_and_limit(3, 25)),
        Some("ws_123"),
    )
    .await
    .expect("list observability destinations should succeed");
    assert!(result.data.is_empty());

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/observability/destinations?offset=3&limit=25&workspace_id=ws_123 HTTP/1.1"
    );
    assert_management_auth_header(&captured.request_text);

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_create_observability_destination_path_body_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(destination_body());
    let request = CreateObservabilityDestinationRequest::builder()
        .destination_type("langfuse")
        .name("Production Langfuse")
        .config(serde_json::json!({
            "baseUrl": "https://us.cloud.langfuse.com",
            "publicKey": "pk-l...EfGh",
            "secretKey": "sk-l...AbCd"
        }))
        .build()
        .expect("create observability request should build");

    let created = observability::create_observability_destination(&base_url, "mgmt-key", &request)
        .await
        .expect("create observability destination should succeed");
    assert_eq!(created.destination_type, "langfuse");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/observability/destinations HTTP/1.1"
    );
    assert_management_auth_header(&captured.request_text);
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be JSON");
    assert_eq!(body["type"], "langfuse");
    assert_eq!(body["config"]["publicKey"], "pk-l...EfGh");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_update_delete_observability_destination_paths() {
    let (base_url, rx, server) = spawn_json_server(destination_body());
    let _ = observability::get_observability_destination(&base_url, "mgmt-key", "dest/team 1")
        .await
        .expect("get observability destination should succeed");
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/observability/destinations/dest%2Fteam%201 HTTP/1.1"
    );
    assert_management_auth_header(&captured.request_text);
    server.join().expect("server thread should finish");

    let (base_url, rx, server) = spawn_json_server(destination_body());
    let request = UpdateObservabilityDestinationRequest::builder()
        .name("Updated Langfuse")
        .enabled(false)
        .build()
        .expect("update observability request should build");
    let _ = observability::update_observability_destination(
        &base_url,
        "mgmt-key",
        "dest/team 1",
        &request,
    )
    .await
    .expect("update observability destination should succeed");
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "PATCH /api/v1/observability/destinations/dest%2Fteam%201 HTTP/1.1"
    );
    assert_management_auth_header(&captured.request_text);
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be JSON");
    assert_eq!(body["name"], "Updated Langfuse");
    assert_eq!(body["enabled"], false);
    server.join().expect("server thread should finish");

    let (base_url, rx, server) = spawn_json_server(r#"{"deleted":true}"#);
    let deleted =
        observability::delete_observability_destination(&base_url, "mgmt-key", "dest/team 1")
            .await
            .expect("delete observability destination should succeed");
    assert!(deleted);
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "DELETE /api/v1/observability/destinations/dest%2Fteam%201 HTTP/1.1"
    );
    assert_management_auth_header(&captured.request_text);
    server.join().expect("server thread should finish");
}
