use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::api::api_keys;

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
