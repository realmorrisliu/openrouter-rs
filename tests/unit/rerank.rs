use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::rerank::{self, RerankRequest, RerankResponse},
    types::ProviderPreferences,
};

#[test]
fn test_rerank_request_serialization() {
    let mut provider = ProviderPreferences::default();
    provider.allow_fallbacks = Some(true);

    let request = RerankRequest::builder()
        .model("cohere/rerank-v3.5")
        .query("What is the capital of France?")
        .documents(vec![
            "Paris is the capital of France.".to_string(),
            "Berlin is the capital of Germany.".to_string(),
        ])
        .top_n(1)
        .provider(provider)
        .build()
        .expect("rerank request should build");

    let value = serde_json::to_value(&request).expect("rerank request should serialize");
    assert_eq!(value["model"], "cohere/rerank-v3.5");
    assert_eq!(value["query"], "What is the capital of France?");
    assert_eq!(value["documents"][0], "Paris is the capital of France.");
    assert_eq!(value["top_n"], 1);
    assert_eq!(value["provider"]["allow_fallbacks"], true);
}

#[test]
fn test_rerank_response_deserialization() {
    let raw = r#"{
        "id": "gen-rerank-123",
        "model": "cohere/rerank-v3.5",
        "provider": "Cohere",
        "results": [{
            "index": 0,
            "relevance_score": 0.98,
            "document": {"text": "Paris is the capital of France."}
        }],
        "usage": {
            "search_units": 1,
            "total_tokens": 150,
            "cost": 0.001
        }
    }"#;

    let parsed: RerankResponse =
        serde_json::from_str(raw).expect("rerank response should deserialize");
    assert_eq!(parsed.model, "cohere/rerank-v3.5");
    assert_eq!(parsed.results.len(), 1);
    assert_eq!(
        parsed.results[0].document.text,
        "Paris is the capital of France."
    );
    assert_eq!(
        parsed.usage.expect("usage should be present").search_units,
        Some(1)
    );
}

#[tokio::test]
async fn test_create_rerank_path_body_and_headers() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<(String, String, String)>();

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
        tx.send((request_line, request_text, body_text))
            .expect("server should send captured request");

        let response = r#"{
            "model":"cohere/rerank-v3.5",
            "results":[{"index":0,"relevance_score":0.98,"document":{"text":"Paris"}}]
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
    let request = RerankRequest::builder()
        .model("cohere/rerank-v3.5")
        .query("capital of France")
        .documents(vec!["Paris is the capital of France.".to_string()])
        .top_n(1)
        .build()
        .expect("rerank request should build");

    let response = rerank::create_rerank(
        &base_url,
        "api-key",
        &Some("openrouter-rs".to_string()),
        &Some("https://example.com".to_string()),
        &Some(vec!["cli-agent".to_string()]),
        &request,
    )
    .await
    .expect("rerank request should succeed");
    assert_eq!(response.results.len(), 1);

    let (request_line, request_text, body_text) = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(request_line, "POST /api/v1/rerank HTTP/1.1");

    let body_json: serde_json::Value =
        serde_json::from_str(&body_text).expect("body should be valid json");
    assert_eq!(body_json["model"], "cohere/rerank-v3.5");
    assert_eq!(body_json["query"], "capital of France");
    assert_eq!(body_json["top_n"], 1);

    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include api key, request:\n{}",
        request_text
    );
    assert!(
        request_lower.contains("x-title: openrouter-rs")
            || request_lower.contains("x-title:openrouter-rs"),
        "x-title header should be present, request:\n{}",
        request_text
    );
    assert!(
        request_lower.contains("http-referer: https://example.com")
            || request_lower.contains("http-referer:https://example.com"),
        "http-referer header should be present, request:\n{}",
        request_text
    );
    assert!(
        request_lower.contains("x-openrouter-categories: cli-agent")
            || request_lower.contains("x-openrouter-categories:cli-agent"),
        "x-openrouter-categories header should be present, request:\n{}",
        request_text
    );

    server.join().expect("server thread should finish");
}
