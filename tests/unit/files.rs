use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::api::files::{self, FileDeleteResponse, FileListResponse, FileMetadata};

struct CapturedRequest {
    request_line: String,
    request_text: String,
    body_text: String,
}

fn spawn_server(
    response_body: &[u8],
    content_type: &str,
) -> (
    String,
    mpsc::Receiver<CapturedRequest>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let body = response_body.to_vec();
    let content_type = content_type.to_string();
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
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            content_type,
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write header");
        stream
            .write_all(&body)
            .expect("server should write response body");
    });

    (format!("http://{addr}/api/v1"), rx, server)
}

fn file_metadata_json() -> &'static str {
    r#"{
        "id": "file_123",
        "type": "file",
        "filename": "document.pdf",
        "mime_type": "application/pdf",
        "size_bytes": 1024,
        "created_at": "2026-06-15T00:00:00Z",
        "downloadable": true
    }"#
}

#[test]
fn test_file_payloads_deserialize() {
    let file: FileMetadata =
        serde_json::from_str(file_metadata_json()).expect("file metadata should deserialize");
    assert_eq!(file.id, "file_123");
    assert_eq!(file.filename, "document.pdf");
    assert!(file.downloadable);

    let list_raw = format!(
        r#"{{
            "data": [{}],
            "has_more": false,
            "first_id": "file_123",
            "last_id": "file_123",
            "cursor": null
        }}"#,
        file_metadata_json()
    );
    let list: FileListResponse =
        serde_json::from_str(&list_raw).expect("file list should deserialize");
    assert_eq!(list.data.len(), 1);
    assert!(!list.has_more);

    let deleted: FileDeleteResponse =
        serde_json::from_str(r#"{"id":"file_123","type":"file_deleted"}"#)
            .expect("file delete response should deserialize");
    assert_eq!(deleted.id, "file_123");
}

#[tokio::test]
async fn test_list_files_path_query_and_auth_header() {
    let (base_url, rx, server) = spawn_server(
        br#"{"data":[],"has_more":false,"first_id":null,"last_id":null,"cursor":null}"#,
        "application/json",
    );

    let files = files::list_files(
        &base_url,
        "api-key",
        Some(20),
        Some("cursor_1"),
        Some("ws_123"),
    )
    .await
    .expect("list files should succeed");
    assert!(files.data.is_empty());

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/files?limit=20&cursor=cursor_1&workspace_id=ws_123 HTTP/1.1"
    );
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
async fn test_get_download_delete_file_paths() {
    let (base_url, rx, server) = spawn_server(file_metadata_json().as_bytes(), "application/json");
    let metadata = files::get_file_metadata(&base_url, "api-key", "file 123", Some("ws_123"))
        .await
        .expect("get file metadata should succeed");
    assert_eq!(metadata.id, "file_123");
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture metadata request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/files/file%20123?workspace_id=ws_123 HTTP/1.1"
    );
    server.join().expect("metadata server thread should finish");

    let (base_url, rx, server) = spawn_server(b"file bytes", "application/octet-stream");
    let bytes = files::download_file_content(&base_url, "api-key", "file 123", None)
        .await
        .expect("download file content should succeed");
    assert_eq!(bytes, b"file bytes");
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture download request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/files/file%20123/content HTTP/1.1"
    );
    server.join().expect("download server thread should finish");

    let (base_url, rx, server) = spawn_server(
        br#"{"id":"file_123","type":"file_deleted"}"#,
        "application/json",
    );
    let deleted = files::delete_file(&base_url, "api-key", "file_123", Some("ws_123"))
        .await
        .expect("delete file should succeed");
    assert_eq!(deleted.id, "file_123");
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture delete request");
    assert_eq!(
        captured.request_line,
        "DELETE /api/v1/files/file_123?workspace_id=ws_123 HTTP/1.1"
    );
    server.join().expect("delete server thread should finish");
}

#[tokio::test]
async fn test_upload_file_uses_multipart_file_part() {
    let (base_url, rx, server) = spawn_server(file_metadata_json().as_bytes(), "application/json");
    let request = files::UploadFileRequest::builder()
        .filename("note.txt")
        .mime_type("text/plain")
        .content(b"hello from file".to_vec())
        .build()
        .expect("upload request should build");

    let uploaded = files::upload_file(&base_url, "api-key", &request, Some("ws_123"))
        .await
        .expect("upload file should succeed");
    assert_eq!(uploaded.id, "file_123");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture upload request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/files?workspace_id=ws_123 HTTP/1.1"
    );
    assert!(
        captured
            .request_text
            .to_ascii_lowercase()
            .contains("content-type: multipart/form-data"),
        "upload should use multipart/form-data, request:\n{}",
        captured.request_text
    );
    assert!(captured.body_text.contains(r#"name="file""#));
    assert!(captured.body_text.contains(r#"filename="note.txt""#));
    assert!(captured.body_text.contains("Content-Type: text/plain"));
    assert!(captured.body_text.contains("hello from file"));

    server.join().expect("server thread should finish");
}
