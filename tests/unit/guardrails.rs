use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::guardrails::{
        self, AssignedCountResponse, BulkKeyAssignmentRequest, CreateGuardrailRequest, Guardrail,
        GuardrailKeyAssignmentsResponse, GuardrailListResponse, GuardrailMemberAssignmentsResponse,
        UnassignedCountResponse, UpdateGuardrailRequest,
    },
    types::ApiResponse,
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
fn test_create_guardrail_request_serialization() {
    let request = CreateGuardrailRequest::builder()
        .name("Production")
        .description("Production guardrail")
        .limit_usd(100.0)
        .reset_interval("monthly")
        .allowed_providers(vec!["openai".to_string(), "anthropic".to_string()])
        .allowed_models(vec![
            "openai/gpt-4.1".to_string(),
            "anthropic/claude-sonnet-4".to_string(),
        ])
        .enforce_zdr(true)
        .build()
        .expect("create guardrail request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value["name"], "Production");
    assert_eq!(value["description"], "Production guardrail");
    assert_eq!(value["limit_usd"], 100.0);
    assert_eq!(value["reset_interval"], "monthly");
    assert_eq!(value["allowed_providers"][0], "openai");
    assert_eq!(value["allowed_models"][1], "anthropic/claude-sonnet-4");
    assert_eq!(value["enforce_zdr"], true);
}

#[test]
fn test_update_guardrail_request_serialization() {
    let request = UpdateGuardrailRequest::builder()
        .name("Updated")
        .enforce_zdr(false)
        .build()
        .expect("update guardrail request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value["name"], "Updated");
    assert_eq!(value["enforce_zdr"], false);
    assert!(value.get("description").is_none());
    assert!(value.get("allowed_models").is_none());
}

#[test]
fn test_guardrail_response_deserialization() {
    let raw = r#"{
        "data": {
            "id": "gr_123",
            "name": "Production Guardrail",
            "description": "Guardrail for production traffic",
            "limit_usd": 100,
            "reset_interval": "monthly",
            "allowed_providers": ["openai"],
            "allowed_models": ["openai/gpt-4.1"],
            "enforce_zdr": true,
            "created_at": "2025-01-01T00:00:00.000Z",
            "updated_at": "2025-01-02T00:00:00.000Z"
        }
    }"#;

    let parsed: ApiResponse<Guardrail> =
        serde_json::from_str(raw).expect("guardrail response should deserialize");
    assert_eq!(parsed.data.id, "gr_123");
    assert_eq!(parsed.data.name, "Production Guardrail");
    assert_eq!(parsed.data.allowed_providers.unwrap_or_default().len(), 1);
    assert_eq!(parsed.data.enforce_zdr, Some(true));
}

#[test]
fn test_guardrail_list_and_assignment_deserialization() {
    let list_raw = r#"{
        "data": [{
            "id": "gr_123",
            "name": "Production Guardrail",
            "description": "Guardrail for production traffic",
            "limit_usd": 100,
            "reset_interval": "monthly",
            "allowed_providers": ["openai"],
            "allowed_models": ["openai/gpt-4.1"],
            "enforce_zdr": true,
            "created_at": "2025-01-01T00:00:00.000Z",
            "updated_at": "2025-01-02T00:00:00.000Z"
        }],
        "total_count": 1
    }"#;

    let key_assignments_raw = r#"{
        "data": [{
            "id": "gka_1",
            "key_hash": "sk-or-v1-abc",
            "guardrail_id": "gr_123",
            "key_name": "Production Key",
            "key_label": "prod",
            "assigned_by": "user_1",
            "created_at": "2025-01-01T00:00:00.000Z"
        }],
        "total_count": 1
    }"#;

    let member_assignments_raw = r#"{
        "data": [{
            "id": "gma_1",
            "user_id": "user_2",
            "organization_id": "org_1",
            "guardrail_id": "gr_123",
            "assigned_by": "user_1",
            "created_at": "2025-01-01T00:00:00.000Z"
        }],
        "total_count": 1
    }"#;

    let list: GuardrailListResponse =
        serde_json::from_str(list_raw).expect("guardrail list should deserialize");
    let key_assignments: GuardrailKeyAssignmentsResponse =
        serde_json::from_str(key_assignments_raw).expect("key assignments should deserialize");
    let member_assignments: GuardrailMemberAssignmentsResponse =
        serde_json::from_str(member_assignments_raw)
            .expect("member assignments should deserialize");

    assert_eq!(list.total_count, 1.0);
    assert_eq!(key_assignments.total_count, 1.0);
    assert_eq!(member_assignments.total_count, 1.0);
    assert_eq!(key_assignments.data[0].assigned_by, "user_1");
    assert_eq!(member_assignments.data[0].assigned_by, "user_1");
}

#[test]
fn test_assignment_counter_response_deserialization() {
    let assigned_raw = r#"{"assigned_count":2}"#;
    let unassigned_raw = r#"{"unassigned_count":1}"#;

    let assigned: AssignedCountResponse =
        serde_json::from_str(assigned_raw).expect("assigned_count should deserialize");
    let unassigned: UnassignedCountResponse =
        serde_json::from_str(unassigned_raw).expect("unassigned_count should deserialize");

    assert_eq!(assigned.assigned_count, 2.0);
    assert_eq!(unassigned.unassigned_count, 1.0);
}

#[tokio::test]
async fn test_list_guardrails_with_pagination_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[],"total_count":0}"#);

    let result = guardrails::list_guardrails(&base_url, "mgmt-key", Some(10), Some(25))
        .await
        .expect("list guardrails should succeed");
    assert_eq!(result.total_count, 0.0);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/guardrails?offset=10&limit=25 HTTP/1.1"
    );
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
async fn test_bulk_assign_keys_encodes_id_and_sends_body() {
    let (base_url, rx, server) = spawn_json_server(r#"{"assigned_count":2}"#);
    let request = BulkKeyAssignmentRequest::builder()
        .key_hashes(vec!["hash_1".to_string(), "hash_2".to_string()])
        .build()
        .expect("bulk assignment request should build");

    let response =
        guardrails::bulk_assign_keys_to_guardrail(&base_url, "mgmt-key", "team/prod 1", &request)
            .await
            .expect("bulk assign should succeed");
    assert_eq!(response.assigned_count, 2.0);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/guardrails/team%2Fprod%201/assignments/keys HTTP/1.1"
    );
    let request_json: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid JSON");
    assert_eq!(request_json["key_hashes"][0], "hash_1");
    assert_eq!(request_json["key_hashes"][1], "hash_2");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_member_assignments_global_path() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[],"total_count":0}"#);

    let result = guardrails::list_member_assignments(&base_url, "mgmt-key", Some(1), Some(2))
        .await
        .expect("list member assignments should succeed");
    assert_eq!(result.total_count, 0.0);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/guardrails/assignments/members?offset=1&limit=2 HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_delete_guardrail_uses_id_path() {
    let (base_url, rx, server) = spawn_json_server(r#"{"deleted":true}"#);

    let deleted = guardrails::delete_guardrail(&base_url, "mgmt-key", "gr/test 1")
        .await
        .expect("delete guardrail should succeed");
    assert!(deleted);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "DELETE /api/v1/guardrails/gr%2Ftest%201 HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}
