use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::organization::{self, OrganizationMembersResponse},
    types::PaginationOptions,
};

#[test]
fn test_organization_members_response_deserialization() {
    let raw = r#"{
        "data": [{
            "id": "user_123",
            "first_name": "Jane",
            "last_name": "Doe",
            "email": "jane@example.com",
            "role": "org:member"
        }],
        "total_count": 1
    }"#;

    let parsed: OrganizationMembersResponse =
        serde_json::from_str(raw).expect("organization members should deserialize");
    assert_eq!(parsed.total_count, 1);
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].email, "jane@example.com");
    assert_eq!(parsed.data[0].role, "org:member");
}

#[tokio::test]
async fn test_list_organization_members_path_pagination_and_auth_header() {
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

        let request_text = String::from_utf8_lossy(&request_bytes).to_string();
        tx.send(request_text)
            .expect("server should send request text");

        let response = r#"{"data":[],"total_count":0}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response.len(),
            response
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let response = organization::list_organization_members(
        &base_url,
        "mgmt-key",
        Some(PaginationOptions::with_offset_and_limit(3, 25)),
    )
    .await
    .expect("list organization members should succeed");
    assert_eq!(response.total_count, 0);

    let request_text = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    let request_line = request_text.lines().next().unwrap_or_default().to_string();
    assert_eq!(
        request_line,
        "GET /api/v1/organization/members?offset=3&limit=25 HTTP/1.1"
    );

    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer mgmt-key")
            || request_lower.contains("authorization:bearer mgmt-key"),
        "authorization header should include management key, request:\n{}",
        request_text
    );

    server.join().expect("server thread should finish");
}
