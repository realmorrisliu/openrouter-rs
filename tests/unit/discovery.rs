use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::discovery::{
        self, ActivityItem, BigNumber, ModelsCountData, Provider, PublicEndpoint, UserModel,
    },
    types::ApiResponse,
};

#[test]
fn test_providers_response_deserialization() {
    let raw = r#"{
        "data": [{
            "name": "OpenAI",
            "slug": "openai",
            "privacy_policy_url": "https://openai.com/privacy",
            "terms_of_service_url": "https://openai.com/terms",
            "status_page_url": "https://status.openai.com"
        }]
    }"#;

    let parsed: ApiResponse<Vec<Provider>> =
        serde_json::from_str(raw).expect("providers response should deserialize");
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].slug, "openai");
    assert_eq!(
        parsed.data[0].status_page_url.as_deref(),
        Some("https://status.openai.com")
    );
}

#[test]
fn test_models_for_user_response_deserialization() {
    let raw = r#"{
        "data": [{
            "id": "openai/gpt-4.1",
            "canonical_slug": "openai/gpt-4.1",
            "hugging_face_id": null,
            "name": "GPT-4.1",
            "created": 1710000000,
            "description": "Test model",
            "pricing": {
                "prompt": "0.000002",
                "completion": 0.000008
            },
            "context_length": 128000,
            "architecture": {
                "tokenizer": "GPT",
                "instruct_type": "chatml",
                "modality": "text->text",
                "input_modalities": ["text"],
                "output_modalities": ["text"]
            },
            "top_provider": {
                "context_length": 128000,
                "max_completion_tokens": 16384,
                "is_moderated": true
            },
            "per_request_limits": null,
            "supported_parameters": ["temperature", "top_p"],
            "default_parameters": null,
            "expiration_date": null
        }]
    }"#;

    let parsed: ApiResponse<Vec<UserModel>> =
        serde_json::from_str(raw).expect("models/user response should deserialize");
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].canonical_slug, "openai/gpt-4.1");
    assert_eq!(parsed.data[0].supported_parameters.len(), 2);
    assert!(matches!(
        parsed.data[0].pricing.prompt,
        BigNumber::String(_)
    ));
    assert!(matches!(
        parsed.data[0].pricing.completion,
        BigNumber::Number(_)
    ));
}

#[test]
fn test_models_count_response_deserialization() {
    let raw = r#"{"data":{"count":150}}"#;
    let parsed: ApiResponse<ModelsCountData> =
        serde_json::from_str(raw).expect("models/count response should deserialize");
    assert_eq!(parsed.data.count, 150);
}

#[test]
fn test_zdr_endpoints_response_deserialization() {
    let raw = r#"{
        "data": [{
            "name": "OpenAI: GPT-4.1",
            "model_id": "openai/gpt-4.1-2025-04-14",
            "model_name": "GPT-4.1",
            "context_length": 128000,
            "pricing": {
                "prompt": "0.000002",
                "completion": "0.000008"
            },
            "provider_name": "OpenAI",
            "tag": "openai",
            "quantization": null,
            "max_completion_tokens": 16384,
            "max_prompt_tokens": 128000,
            "supported_parameters": ["temperature", "top_p"],
            "status": 0,
            "uptime_last_30m": 99.9,
            "supports_implicit_caching": true,
            "latency_last_30m": {
                "p50": 0.1,
                "p75": 0.2,
                "p90": 0.3,
                "p99": 0.5
            },
            "throughput_last_30m": null
        }]
    }"#;

    let parsed: ApiResponse<Vec<PublicEndpoint>> =
        serde_json::from_str(raw).expect("endpoints/zdr response should deserialize");
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].model_name, "GPT-4.1");
    assert_eq!(parsed.data[0].status, Some(0));
    assert!(parsed.data[0].throughput_last_30m.is_none());
}

#[test]
fn test_activity_response_deserialization() {
    let raw = r#"{
        "data": [{
            "date": "2025-08-24",
            "model": "openai/gpt-4.1",
            "model_permaslug": "openai/gpt-4.1-2025-04-14",
            "endpoint_id": "550e8400-e29b-41d4-a716-446655440000",
            "provider_name": "OpenAI",
            "usage": 0.015,
            "byok_usage_inference": 0.012,
            "requests": 5,
            "prompt_tokens": 50,
            "completion_tokens": 125,
            "reasoning_tokens": 25
        }]
    }"#;

    let parsed: ApiResponse<Vec<ActivityItem>> =
        serde_json::from_str(raw).expect("activity response should deserialize");
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].date, "2025-08-24");
    assert_eq!(parsed.data[0].requests, 5.0);
}

#[tokio::test]
async fn test_list_models_for_user_request_path() {
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

        let body = r#"{"data":[]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let models = discovery::list_models_for_user(&base_url, "test-key")
        .await
        .expect("models/user request should succeed");
    assert!(models.is_empty());

    let request_line = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request line");
    assert_eq!(request_line, "GET /api/v1/models/user HTTP/1.1");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_activity_with_date_query_and_auth_header() {
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
        tx.send(request_text).expect("server should send request");

        let body = r#"{"data":[]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let items = discovery::get_activity(&base_url, "mgmt-key", Some("2025-08-24"))
        .await
        .expect("activity request should succeed");
    assert!(items.is_empty());

    let request_text = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    let mut lines = request_text.lines();
    let request_line = lines.next().unwrap_or_default();
    assert_eq!(
        request_line,
        "GET /api/v1/activity?date=2025-08-24 HTTP/1.1"
    );

    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer mgmt-key")
            || request_lower.contains("authorization:bearer mgmt-key"),
        "authorization header should include management key, request:\n{request_text}"
    );

    server.join().expect("server thread should finish");
}
