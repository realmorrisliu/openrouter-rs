use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{api::scim, types::PaginationOptions};

fn spawn_json_server(
    response_body: &str,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("listener should have address");
    let body = response_body.to_string();
    let (tx, rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("server should accept request");
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        let header_end = loop {
            let read = stream.read(&mut chunk).expect("server should read request");
            request.extend_from_slice(&chunk[..read]);
            if let Some(position) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                break position + 4;
            }
            assert_ne!(read, 0, "request should contain headers");
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                line.to_ascii_lowercase()
                    .strip_prefix("content-length:")?
                    .trim()
                    .parse::<usize>()
                    .ok()
            })
            .unwrap_or(0);
        while request.len() - header_end < content_length {
            let read = stream.read(&mut chunk).expect("server should read body");
            request.extend_from_slice(&chunk[..read]);
        }
        tx.send(String::from_utf8_lossy(&request).into_owned())
            .expect("request should be captured");
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
async fn test_list_scim_groups_path_and_response() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[{"id":"group-1","organization_id":"org-1","external_id":null,"display_name":"Engineering","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}],"total_count":1}"#,
    );

    let response = scim::list_scim_groups(
        &base_url,
        "mgmt-key",
        Some(PaginationOptions::with_offset_and_limit(5, 25)),
    )
    .await
    .expect("SCIM groups should list");

    assert_eq!(response.total_count, 1);
    assert_eq!(response.data[0].display_name, "Engineering");
    let request = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert!(request.starts_with("GET /api/v1/scim/groups?offset=5&limit=25 HTTP/1.1"));
    assert!(request.to_ascii_lowercase().contains("bearer mgmt-key"));
    server.join().expect("server should finish");
}

#[tokio::test]
async fn test_list_scim_group_mappings_path_and_response() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[{"id":"mapping-1","organization_id":"org-1","scim_group_id":"group-1","workspace_id":"workspace-1","role":"member","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}],"total_count":1}"#,
    );

    let response = scim::list_scim_group_mappings(&base_url, "mgmt-key", None)
        .await
        .expect("SCIM mappings should list");

    assert_eq!(response.total_count, 1);
    assert_eq!(response.data[0].role, "member");
    let request = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert!(request.starts_with("GET /api/v1/scim/group-mappings HTTP/1.1"));
    server.join().expect("server should finish");
}

#[tokio::test]
async fn test_create_scim_group_mapping_path_body_and_response() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"mapping-1","organization_id":"org-1","scim_group_id":"group-1","workspace_id":"workspace-1","role":"admin","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}}"#,
    );
    let body = scim::CreateScimGroupMappingRequest::builder()
        .scim_group_id("group-1")
        .workspace_id("workspace-1")
        .role("admin")
        .build()
        .expect("mapping request should build");

    let mapping = scim::create_scim_group_mapping(&base_url, "mgmt-key", &body)
        .await
        .expect("SCIM mapping should be created");

    assert_eq!(mapping.role, "admin");
    let request = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert!(request.starts_with("POST /api/v1/scim/group-mappings HTTP/1.1"));
    assert!(request.contains(r#""scim_group_id":"group-1""#));
    assert!(request.contains(r#""workspace_id":"workspace-1""#));
    assert!(request.contains(r#""role":"admin""#));
    server.join().expect("server should finish");
}

#[tokio::test]
async fn test_get_scim_group_mapping_encodes_id() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"mapping-1","organization_id":"org-1","scim_group_id":"group-1","workspace_id":"workspace-1","role":"member","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}}"#,
    );

    let mapping = scim::get_scim_group_mapping(&base_url, "mgmt-key", "mapping 1")
        .await
        .expect("SCIM mapping should be returned");

    assert_eq!(mapping.id, "mapping-1");
    let request = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert!(request.starts_with("GET /api/v1/scim/group-mappings/mapping%201 HTTP/1.1"));
    server.join().expect("server should finish");
}

#[tokio::test]
async fn test_update_scim_group_mapping_path_body_and_response() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"mapping-1","organization_id":"org-1","scim_group_id":"group-1","workspace_id":"workspace-1","role":"admin","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-02T00:00:00Z"}}"#,
    );
    let body = scim::UpdateScimGroupMappingRequest::builder()
        .role("admin")
        .build()
        .expect("update request should build");

    let mapping = scim::update_scim_group_mapping(&base_url, "mgmt-key", "mapping-1", &body)
        .await
        .expect("SCIM mapping should update");

    assert_eq!(mapping.role, "admin");
    let request = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert!(request.starts_with("PATCH /api/v1/scim/group-mappings/mapping-1 HTTP/1.1"));
    assert!(request.contains(r#""role":"admin""#));
    server.join().expect("server should finish");
}

#[tokio::test]
async fn test_delete_scim_group_mapping_requires_keep_members_query() {
    let (base_url, rx, server) = spawn_json_server(r#"{"deleted":true}"#);

    let deleted = scim::delete_scim_group_mapping(&base_url, "mgmt-key", "mapping-1", false)
        .await
        .expect("SCIM mapping should delete");

    assert!(deleted);
    let request = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert!(
        request.starts_with(
            "DELETE /api/v1/scim/group-mappings/mapping-1?keep_members=false HTTP/1.1"
        )
    );
    server.join().expect("server should finish");
}
