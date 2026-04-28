use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::workspaces::{
        self, CreateWorkspaceRequest, UpdateWorkspaceRequest, Workspace, WorkspaceListResponse,
        WorkspaceMembersAddResponse, WorkspaceMembersRemoveResponse, WorkspaceMembersRequest,
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

#[test]
fn test_create_workspace_request_serialization() {
    let request = CreateWorkspaceRequest::builder()
        .name("Production")
        .slug("production")
        .description("Production environment")
        .default_text_model("openai/gpt-4o")
        .default_image_model("openai/dall-e-3")
        .default_provider_sort("price")
        .is_data_discount_logging_enabled(true)
        .is_observability_broadcast_enabled(false)
        .is_observability_io_logging_enabled(false)
        .io_logging_api_key_ids(vec![101, 202])
        .io_logging_sampling_rate(0.25)
        .build()
        .expect("create workspace request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value["name"], "Production");
    assert_eq!(value["slug"], "production");
    assert_eq!(value["description"], "Production environment");
    assert_eq!(value["default_text_model"], "openai/gpt-4o");
    assert_eq!(value["default_image_model"], "openai/dall-e-3");
    assert_eq!(value["default_provider_sort"], "price");
    assert_eq!(value["is_data_discount_logging_enabled"], true);
    assert_eq!(value["is_observability_broadcast_enabled"], false);
    assert_eq!(value["is_observability_io_logging_enabled"], false);
    assert_eq!(
        value["io_logging_api_key_ids"],
        serde_json::json!([101, 202])
    );
    assert_eq!(value["io_logging_sampling_rate"], 0.25);
}

#[test]
fn test_update_workspace_request_serialization() {
    let request = UpdateWorkspaceRequest::builder()
        .name("Updated")
        .slug("updated")
        .is_data_discount_logging_enabled(false)
        .io_logging_api_key_ids(Vec::<u64>::new())
        .io_logging_sampling_rate(1.0)
        .build()
        .expect("update workspace request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value["name"], "Updated");
    assert_eq!(value["slug"], "updated");
    assert_eq!(value["is_data_discount_logging_enabled"], false);
    assert_eq!(value["io_logging_api_key_ids"], serde_json::json!([]));
    assert_eq!(value["io_logging_sampling_rate"], 1.0);
    assert!(value.get("description").is_none());
}

#[test]
fn test_workspace_response_deserialization() {
    let raw = r#"{
        "data": {
            "id": "ws_123",
            "name": "Production",
            "slug": "production",
            "description": "Production environment",
            "default_text_model": "openai/gpt-4o",
            "default_image_model": "openai/dall-e-3",
            "default_provider_sort": "price",
            "io_logging_api_key_ids": [101, 202],
            "io_logging_sampling_rate": 0.5,
            "is_observability_io_logging_enabled": false,
            "is_observability_broadcast_enabled": false,
            "is_data_discount_logging_enabled": true,
            "created_at": "2025-01-01T00:00:00.000Z",
            "updated_at": "2025-01-02T00:00:00.000Z",
            "created_by": "user_123"
        }
    }"#;

    let parsed: ApiResponse<Workspace> =
        serde_json::from_str(raw).expect("workspace response should deserialize");
    assert_eq!(parsed.data.id, "ws_123");
    assert_eq!(parsed.data.slug, "production");
    assert_eq!(parsed.data.created_by.as_deref(), Some("user_123"));
    assert_eq!(parsed.data.io_logging_api_key_ids, Some(vec![101, 202]));
    assert_eq!(parsed.data.io_logging_sampling_rate, 0.5);
}

#[test]
fn test_workspace_list_and_member_response_deserialization() {
    let list_raw = r#"{
        "data": [{
            "id": "ws_123",
            "name": "Production",
            "slug": "production",
            "description": "Production environment",
            "default_text_model": "openai/gpt-4o",
            "default_image_model": "openai/dall-e-3",
            "default_provider_sort": "price",
            "io_logging_api_key_ids": null,
            "io_logging_sampling_rate": 1.0,
            "is_observability_io_logging_enabled": false,
            "is_observability_broadcast_enabled": false,
            "is_data_discount_logging_enabled": true,
            "created_at": "2025-01-01T00:00:00.000Z",
            "updated_at": "2025-01-02T00:00:00.000Z",
            "created_by": "user_123"
        }],
        "total_count": 1
    }"#;

    let add_raw = r#"{
        "added_count": 1,
        "data": [{
            "id": "wm_1",
            "workspace_id": "ws_123",
            "user_id": "user_123",
            "role": "member",
            "created_at": "2025-01-01T00:00:00.000Z"
        }]
    }"#;

    let remove_raw = r#"{"removed_count":2}"#;

    let list: WorkspaceListResponse =
        serde_json::from_str(list_raw).expect("workspace list should deserialize");
    let add: WorkspaceMembersAddResponse =
        serde_json::from_str(add_raw).expect("workspace member add should deserialize");
    let remove: WorkspaceMembersRemoveResponse =
        serde_json::from_str(remove_raw).expect("workspace member remove should deserialize");

    assert_eq!(list.total_count, 1.0);
    assert_eq!(list.data[0].io_logging_api_key_ids, None);
    assert_eq!(list.data[0].io_logging_sampling_rate, 1.0);
    assert_eq!(add.added_count, 1.0);
    assert_eq!(add.data[0].workspace_id, "ws_123");
    assert_eq!(remove.removed_count, 2.0);
}

#[tokio::test]
async fn test_list_workspaces_path_pagination_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[],"total_count":0}"#);

    let result = workspaces::list_workspaces(
        &base_url,
        "mgmt-key",
        Some(PaginationOptions::with_offset_and_limit(3, 25)),
    )
    .await
    .expect("list workspaces should succeed");
    assert_eq!(result.total_count, 0.0);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/workspaces?offset=3&limit=25 HTTP/1.1"
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
async fn test_create_workspace_posts_body_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"ws_123","name":"Production","slug":"production","description":"Production","default_text_model":"openai/gpt-4o","default_image_model":"openai/dall-e-3","default_provider_sort":"price","io_logging_api_key_ids":null,"io_logging_sampling_rate":1.0,"is_observability_io_logging_enabled":false,"is_observability_broadcast_enabled":false,"is_data_discount_logging_enabled":true,"created_at":"2025-01-01T00:00:00.000Z","updated_at":null,"created_by":"user_123"}}"#,
    );
    let request = CreateWorkspaceRequest::builder()
        .name("Production")
        .slug("production")
        .build()
        .expect("request should build");

    let response = workspaces::create_workspace(&base_url, "mgmt-key", &request)
        .await
        .expect("create workspace should succeed");
    assert_eq!(response.id, "ws_123");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "POST /api/v1/workspaces HTTP/1.1");
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("body should be valid json");
    assert_eq!(body["name"], "Production");
    assert_eq!(body["slug"], "production");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_workspace_encodes_id_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"ws_123","name":"Production","slug":"production","description":null,"default_text_model":null,"default_image_model":null,"default_provider_sort":null,"io_logging_api_key_ids":null,"io_logging_sampling_rate":1.0,"is_observability_io_logging_enabled":false,"is_observability_broadcast_enabled":false,"is_data_discount_logging_enabled":true,"created_at":"2025-01-01T00:00:00.000Z","updated_at":null,"created_by":"user_123"}}"#,
    );

    let response = workspaces::get_workspace(&base_url, "mgmt-key", "team/prod 1")
        .await
        .expect("get workspace should succeed");
    assert_eq!(response.id, "ws_123");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/workspaces/team%2Fprod%201 HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_update_workspace_encodes_id_and_sends_body() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"ws_123","name":"Updated","slug":"updated","description":null,"default_text_model":null,"default_image_model":null,"default_provider_sort":null,"io_logging_api_key_ids":null,"io_logging_sampling_rate":1.0,"is_observability_io_logging_enabled":false,"is_observability_broadcast_enabled":false,"is_data_discount_logging_enabled":true,"created_at":"2025-01-01T00:00:00.000Z","updated_at":"2025-01-02T00:00:00.000Z","created_by":"user_123"}}"#,
    );
    let request = UpdateWorkspaceRequest::builder()
        .name("Updated")
        .build()
        .expect("request should build");

    let response = workspaces::update_workspace(&base_url, "mgmt-key", "team/prod 1", &request)
        .await
        .expect("update workspace should succeed");
    assert_eq!(response.name, "Updated");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "PATCH /api/v1/workspaces/team%2Fprod%201 HTTP/1.1"
    );
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("body should be valid json");
    assert_eq!(body["name"], "Updated");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_delete_workspace_encodes_id_path() {
    let (base_url, rx, server) = spawn_json_server(r#"{"deleted":true}"#);

    let deleted = workspaces::delete_workspace(&base_url, "mgmt-key", "team/prod 1")
        .await
        .expect("delete workspace should succeed");
    assert!(deleted);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "DELETE /api/v1/workspaces/team%2Fprod%201 HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_add_workspace_members_encodes_id_and_sends_body() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"added_count":1,"data":[{"id":"wm_1","workspace_id":"ws_123","user_id":"user_123","role":"member","created_at":"2025-01-01T00:00:00.000Z"}]}"#,
    );
    let request = WorkspaceMembersRequest::builder()
        .user_ids(vec!["user_123".to_string()])
        .build()
        .expect("request should build");

    let response =
        workspaces::add_workspace_members(&base_url, "mgmt-key", "team/prod 1", &request)
            .await
            .expect("add members should succeed");
    assert_eq!(response.added_count, 1.0);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/workspaces/team%2Fprod%201/members/add HTTP/1.1"
    );
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("body should be valid json");
    assert_eq!(
        body.get("user_ids")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(1)
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_remove_workspace_members_encodes_id_and_sends_body() {
    let (base_url, rx, server) = spawn_json_server(r#"{"removed_count":1}"#);
    let request = WorkspaceMembersRequest::builder()
        .user_ids(vec!["user_123".to_string()])
        .build()
        .expect("request should build");

    let response =
        workspaces::remove_workspace_members(&base_url, "mgmt-key", "team/prod 1", &request)
            .await
            .expect("remove members should succeed");
    assert_eq!(response.removed_count, 1.0);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/workspaces/team%2Fprod%201/members/remove HTTP/1.1"
    );
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("body should be valid json");
    assert_eq!(
        body.get("user_ids")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(1)
    );

    server.join().expect("server thread should finish");
}
