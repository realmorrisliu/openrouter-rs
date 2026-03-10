use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::models::{self, EndpointData},
    types::{ApiResponse, ModelCategory, SupportedParameters},
};

struct CapturedRequest {
    request_line: String,
    request_text: String,
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
        let request_line = request_text.lines().next().unwrap_or_default().to_string();
        tx.send(CapturedRequest {
            request_line,
            request_text,
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

#[test]
fn test_model_endpoints_pricing_allows_missing_optional_fields() {
    let raw = r#"{
        "data": {
            "id": "qwen/qwen3.5-35b-a3b",
            "name": "Qwen 3.5 35B A3B",
            "created": 1735689600,
            "description": "Test endpoint data",
            "architecture": {
                "tokenizer": "Qwen",
                "instruct_type": "chat",
                "modality": "text->text"
            },
            "endpoints": [{
                "name": "Qwen: Qwen 3.5 35B A3B",
                "context_length": 262144,
                "pricing": {
                    "prompt": "0.00000025",
                    "completion": "0.000002",
                    "discount": 0
                },
                "provider_name": "Qwen",
                "supported_parameters": ["temperature", "top_p"],
                "quantization": null,
                "max_completion_tokens": 16384,
                "max_prompt_tokens": 262144,
                "status": 0
            }]
        }
    }"#;

    let parsed: ApiResponse<EndpointData> =
        serde_json::from_str(raw).expect("models endpoints payload should deserialize");

    assert_eq!(parsed.data.id, "qwen/qwen3.5-35b-a3b");
    assert_eq!(parsed.data.endpoints.len(), 1);

    let pricing = &parsed.data.endpoints[0].pricing;
    assert_eq!(pricing.prompt, "0.00000025");
    assert_eq!(pricing.completion, "0.000002");
    assert!(pricing.request.is_none());
    assert!(pricing.image.is_none());
}

#[tokio::test]
async fn test_list_model_endpoints_encodes_author_slug_and_auth_header() {
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

        let response = r#"{
            "data": {
                "id": "author/slug",
                "name": "Test Endpoint",
                "created": 1735689600,
                "description": "Test endpoint data",
                "architecture": {
                    "tokenizer": "test-tokenizer",
                    "instruct_type": "chat",
                    "modality": "text->text"
                },
                "endpoints": []
            }
        }"#;
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
    let endpoint_data =
        models::list_model_endpoints(&base_url, "api-key", "team/prod", "model alpha")
            .await
            .expect("list model endpoints should succeed");
    assert_eq!(endpoint_data.id, "author/slug");

    let request_text = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    let request_line = request_text.lines().next().unwrap_or_default().to_string();
    assert_eq!(
        request_line,
        "GET /api/v1/models/team%2Fprod/model%20alpha/endpoints HTTP/1.1"
    );

    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include API key, request:\n{}",
        request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_models_default_path_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[]}"#);

    let models = models::list_models(&base_url, "api-key", None, None)
        .await
        .expect("list models should succeed");
    assert!(models.is_empty(), "response payload should parse");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/models HTTP/1.1");
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
async fn test_list_models_with_category_query() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[]}"#);

    let models = models::list_models(&base_url, "api-key", Some(ModelCategory::Programming), None)
        .await
        .expect("list models by category should succeed");
    assert!(models.is_empty(), "response payload should parse");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/models?category=programming HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_models_with_supported_parameters_query() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[]}"#);

    let models = models::list_models(&base_url, "api-key", None, Some(SupportedParameters::TopP))
        .await
        .expect("list models by supported parameter should succeed");
    assert!(models.is_empty(), "response payload should parse");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/models?supported_parameters=top_p HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_models_ignores_filters_when_both_are_set() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[]}"#);

    let models = models::list_models(
        &base_url,
        "api-key",
        Some(ModelCategory::Programming),
        Some(SupportedParameters::TopP),
    )
    .await
    .expect("list models should succeed");
    assert!(models.is_empty(), "response payload should parse");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/models HTTP/1.1");

    server.join().expect("server thread should finish");
}
