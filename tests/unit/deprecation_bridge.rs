use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{OpenRouterClient, api::auth, error::OpenRouterError};
use serde_json::Value;

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
        .expect("request should include header terminator");

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

        tx.send(CapturedRequest {
            request_line,
            request_text: format!("{header_text}{body_text}"),
            body_text,
        })
        .expect("captured request should send");

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
#[allow(deprecated)]
async fn test_builder_provisioning_key_bridge_and_legacy_offset_bridge() {
    let (base_url, rx, server) =
        spawn_json_server(r#"{"data":[{"name":"ops","hash":"key_hash_1","disabled":false}]}"#);

    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .provisioning_key("mgmt-key")
        .build()
        .expect("client should build");

    let result = client
        .list_api_keys(Some(7.0_f64), Some(true))
        .await
        .expect("list_api_keys should succeed");
    assert_eq!(result.len(), 1);
    assert_eq!(
        result.first().and_then(|key| key.hash.as_deref()),
        Some("key_hash_1")
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/keys?offset=7&include_disabled=true HTTP/1.1"
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
#[allow(deprecated)]
async fn test_set_and_clear_provisioning_key_bridge() {
    let mut client = OpenRouterClient::builder()
        .build()
        .expect("client should build");
    client.set_provisioning_key("mgmt-key");
    client.clear_provisioning_key();

    let result = client.delete_api_key("hash").await;
    assert!(matches!(result, Err(OpenRouterError::KeyNotConfigured)));
}

#[tokio::test]
#[allow(deprecated)]
async fn test_models_legacy_aliases_forward_to_renamed_methods() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");

    let user_models = client.models().list_for_user().await;
    assert!(matches!(
        user_models,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let model_count = client.models().count().await;
    assert!(matches!(
        model_count,
        Err(OpenRouterError::KeyNotConfigured)
    ));
}

#[tokio::test]
#[allow(deprecated)]
async fn test_management_exchange_code_for_api_key_alias() {
    let (base_url, rx, server) =
        spawn_json_server(r#"{"key":"sk-or-v1-bridge","user_id":"user_1"}"#);

    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .build()
        .expect("client should build");

    let response = client
        .management()
        .exchange_code_for_api_key(
            "auth-code-1",
            Some("pkce-verifier-1"),
            Some(auth::CodeChallengeMethod::S256),
        )
        .await
        .expect("exchange should succeed");
    assert_eq!(response.key, "sk-or-v1-bridge");
    assert_eq!(response.user_id.as_deref(), Some("user_1"));

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(captured.request_line, "POST /api/v1/auth/keys HTTP/1.1");
    let body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(
        body.get("code").and_then(Value::as_str),
        Some("auth-code-1")
    );
    assert_eq!(
        body.get("code_verifier").and_then(Value::as_str),
        Some("pkce-verifier-1")
    );
    assert_eq!(
        body.get("code_challenge_method").and_then(Value::as_str),
        Some("S256")
    );

    server.join().expect("server thread should finish");
}

#[test]
#[allow(deprecated)]
#[cfg(feature = "legacy-completions")]
fn test_legacy_completion_module_path_alias_compiles() {
    let request = openrouter_rs::api::completion::CompletionRequest::builder()
        .model("openai/gpt-4.1")
        .prompt("hello")
        .build()
        .expect("completion request should build");

    let _ = request;
}
