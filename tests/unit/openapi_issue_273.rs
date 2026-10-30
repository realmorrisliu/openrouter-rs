use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
};

use openrouter_rs::{
    OpenRouterClient,
    api::{
        auth::OAuthTokenExchangeRequest,
        batches::{Batch, CreateBatchRequest},
        discovery::{ListUserModelsParams, PublicEndpoint},
        guardrails::{CreateGuardrailRequest, Guardrail, UpdateGuardrailRequest},
        images::ImageProviderOptions,
        models::Endpoint,
        observability::{
            CreateObservabilityDestinationRequest, ObservabilityDestination,
            UpdateObservabilityDestinationRequest,
        },
        responses::ResponsesRequest,
        videos::VideoGenerationResponse,
    },
    types::CompletionsResponse,
};
use serde_json::{Value, json};

struct CapturedRequest {
    line: String,
    headers: String,
}

fn spawn_server(
    response_body: Vec<u8>,
    content_type: &str,
) -> (
    String,
    mpsc::Receiver<CapturedRequest>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener binds");
    let addr = listener.local_addr().expect("local address");
    let (tx, rx) = mpsc::channel();
    let content_type = content_type.to_string();
    let task = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 2048];
        let header_end = loop {
            let count = stream.read(&mut chunk).expect("read request");
            if count == 0 {
                break None;
            }
            bytes.extend_from_slice(&chunk[..count]);
            if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                break Some(end + 4);
            }
        }
        .expect("request has headers");
        let headers = String::from_utf8_lossy(&bytes[..header_end]).to_string();
        let line = headers.lines().next().unwrap_or_default().to_string();
        let content_length = headers
            .lines()
            .find_map(|line| {
                line.to_ascii_lowercase()
                    .strip_prefix("content-length:")
                    .and_then(|length| length.trim().parse::<usize>().ok())
            })
            .unwrap_or(0);
        while bytes.len() < header_end + content_length {
            let count = stream.read(&mut chunk).expect("read body");
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&chunk[..count]);
        }
        tx.send(CapturedRequest { line, headers })
            .expect("send capture");
        let mut response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            response_body.len()
        )
        .into_bytes();
        response.extend_from_slice(&response_body);
        stream.write_all(&response).expect("write response");
    });
    (format!("http://{addr}/api/v1"), rx, task)
}

fn client(base_url: String) -> OpenRouterClient {
    OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("test-key")
        .build()
        .expect("build client")
}

fn attribution_client(base_url: String) -> OpenRouterClient {
    OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("test-key")
        .http_referer("https://example.com/app")
        .x_title("drift-test")
        .app_categories(["docs", "sdk"])
        .build()
        .expect("build client")
}

#[test]
fn issue_273_batch_lifecycle_and_callback_fields() {
    let request = CreateBatchRequest::builder()
        .endpoint("chat/completions")
        .model("openai/gpt-4o")
        .requests(Vec::new())
        .callback_url("https://example.com/batches/hook")
        .build()
        .expect("batch request builds");
    let value = serde_json::to_value(&request).expect("serialize batch request");
    assert_eq!(
        value["callback_url"],
        Value::String("https://example.com/batches/hook".into())
    );
    let wire = serde_json::to_string(&request).expect("serialize batch request to JSON");
    let callback = wire
        .find("\"callback_url\"")
        .expect("callback_url is present");
    let requests = wire.find("\"requests\"").expect("requests is present");
    assert!(
        callback < requests,
        "callback_url must appear before requests: {wire}"
    );

    let batch: Batch = serde_json::from_value(json!({
        "id": "batch-1",
        "object": "batch",
        "endpoint": "chat/completions",
        "model": "openai/gpt-4o",
        "completion_window": "24h",
        "status": "in_progress",
        "created_at": 1_700_000_000,
        "expires_at": 1_700_086_400,
        "finalized_at": null,
        "provider": "openai",
        "request_counts": {"total": 1, "completed": 0, "failed": 0},
        "usage": null,
        "results": null,
        "error": null
    }))
    .expect("batch deserializes");
    assert_eq!(batch.expires_at, Some(1_700_086_400));
    assert_eq!(batch.provider.as_deref(), Some("openai"));

    let unassigned: Batch = serde_json::from_value(json!({
        "id": "batch-2",
        "object": "batch",
        "endpoint": "chat/completions",
        "model": "openai/gpt-4o",
        "completion_window": "24h",
        "status": "validating",
        "created_at": 1_700_000_000,
        "expires_at": null,
        "finalized_at": null,
        "provider": null,
        "request_counts": {"total": 0, "completed": 0, "failed": 0},
        "usage": null,
        "results": null,
        "error": null
    }))
    .expect("unassigned batch deserializes");
    assert_eq!(unassigned.expires_at, None);
    assert_eq!(unassigned.provider, None);
}

#[test]
fn issue_273_model_endpoint_service_tier_and_diarization() {
    let endpoint: Endpoint = serde_json::from_value(json!({
        "name": "OpenAI: GPT-4",
        "context_length": 8192,
        "pricing": {"prompt": "0.00003", "completion": "0.00006"},
        "provider_name": "OpenAI",
        "supported_parameters": ["temperature"],
        "quantization": "fp16",
        "service_tier": "priority",
        "supports_diarization": true
    }))
    .expect("endpoint deserializes");
    assert_eq!(endpoint.service_tier.as_deref(), Some("priority"));
    assert!(endpoint.supports_diarization);

    let public: PublicEndpoint = serde_json::from_value(json!({
        "name": "OpenAI: GPT-4",
        "model_id": "openai/gpt-4",
        "model_name": "GPT-4",
        "context_length": 8192,
        "pricing": {"prompt": "0.00003", "completion": "0.00006"},
        "provider_name": "OpenAI",
        "tag": "openai",
        "supports_implicit_caching": true,
        "service_tier": null,
        "supports_diarization": false
    }))
    .expect("public endpoint deserializes");
    assert_eq!(public.service_tier, None);
    assert!(!public.supports_diarization);
}

#[test]
fn issue_273_guardrail_safety_retention_fields() {
    let create = CreateGuardrailRequest::builder()
        .name("guardrail")
        .allow_safety_retention_google(true)
        .allow_safety_retention_other(false)
        .build()
        .expect("guardrail create builds");
    let value = serde_json::to_value(&create).expect("serialize guardrail create");
    assert_eq!(value["allow_safety_retention_google"], Value::Bool(true));
    assert_eq!(value["allow_safety_retention_other"], Value::Bool(false));

    let update = UpdateGuardrailRequest::builder()
        .allow_safety_retention_google(false)
        .build()
        .expect("guardrail update builds");
    let value = serde_json::to_value(&update).expect("serialize guardrail update");
    assert_eq!(value["allow_safety_retention_google"], Value::Bool(false));
    assert!(value.get("allow_safety_retention_other").is_none());

    let guardrail: Guardrail = serde_json::from_value(json!({
        "id": "guardrail-1",
        "name": "guardrail",
        "allow_safety_retention_google": true,
        "allow_safety_retention_other": null,
        "created_at": "2026-10-19T00:00:00Z"
    }))
    .expect("guardrail deserializes");
    assert_eq!(guardrail.allow_safety_retention_google, Some(true));
    assert_eq!(guardrail.allow_safety_retention_other, None);
}

#[test]
fn issue_273_observability_cost_allocation_flag() {
    let create = CreateObservabilityDestinationRequest::builder()
        .destination_type("webhook")
        .name("destination")
        .config(json!({"url": "https://example.com"}))
        .broadcast_generation_cost_allocation(true)
        .build()
        .expect("observability create builds");
    let value = serde_json::to_value(&create).expect("serialize observability create");
    assert_eq!(
        value["broadcast_generation_cost_allocation"],
        Value::Bool(true)
    );

    let update = UpdateObservabilityDestinationRequest::builder()
        .broadcast_generation_cost_allocation(false)
        .build()
        .expect("observability update builds");
    let value = serde_json::to_value(&update).expect("serialize observability update");
    assert_eq!(
        value["broadcast_generation_cost_allocation"],
        Value::Bool(false)
    );

    let destination: ObservabilityDestination = serde_json::from_value(json!({
        "id": "destination-1",
        "workspace_id": "workspace-1",
        "enabled": true,
        "privacy_mode": false,
        "sampling_rate": 1.0,
        "created_at": "2026-10-19T00:00:00Z",
        "updated_at": "2026-10-19T00:00:00Z",
        "type": "webhook",
        "config": {},
        "broadcast_generation_cost": false,
        "broadcast_generation_cost_allocation": true
    }))
    .expect("observability destination deserializes");
    assert!(destination.broadcast_generation_cost_allocation);
}

#[test]
fn issue_273_video_urls_and_chat_alignment_payloads() {
    let video: VideoGenerationResponse = serde_json::from_value(json!({
        "id": "gen-vid-1",
        "polling_url": "/api/v1/videos/gen-vid-1",
        "status": "completed",
        "urls": ["https://cdn.example.com/out.mp4"]
    }))
    .expect("video response deserializes");
    assert_eq!(
        video.urls.as_deref(),
        Some(&["https://cdn.example.com/out.mp4".to_string()][..])
    );

    let chat: CompletionsResponse = serde_json::from_value(json!({
        "id": "chat-1",
        "choices": [],
        "created": 1,
        "model": "openai/gpt-4o",
        "object": "chat.completion",
        "provider": "OpenAI",
        "alignment": {"calls": [{"call": 1, "outcome": "allowed", "rules": []}]},
        "safeguard_results": {
            "type": "dangerous_tool_use",
            "status": {"type": "available", "tool_uses": {}}
        }
    }))
    .expect("chat response deserializes");
    assert!(chat.alignment.is_some());
    assert!(chat.safeguard_results.is_some());
}

#[test]
fn issue_273_image_provider_privacy_and_responses_stop() {
    let provider = ImageProviderOptions::default()
        .data_collection("deny")
        .zdr(true);
    assert_eq!(provider.data_collection.as_deref(), Some("deny"));
    assert_eq!(provider.zdr, Some(true));

    let decoded: ImageProviderOptions = serde_json::from_value(json!({
        "data_collection": "deny",
        "zdr": true
    }))
    .expect("image provider options deserialize");
    assert_eq!(decoded.data_collection.as_deref(), Some("deny"));
    assert_eq!(decoded.zdr, Some(true));

    let single = ResponsesRequest::builder()
        .model("openai/gpt-4o")
        .stop("END")
        .build()
        .expect("responses request builds");
    let value = serde_json::to_value(&single).expect("serialize responses request");
    assert_eq!(value["stop"], Value::String("END".into()));

    let multiple = ResponsesRequest::builder()
        .model("openai/gpt-4o")
        .stop(json!(["a", "b"]))
        .build()
        .expect("responses request builds");
    let value = serde_json::to_value(&multiple).expect("serialize responses request");
    assert_eq!(value["stop"], json!(["a", "b"]));
}

#[test]
fn issue_273_oauth_management_scope_is_accepted() {
    let request = OAuthTokenExchangeRequest::builder()
        .federation_policy_id("policy-1")
        .subject_token("header.payload.signature")
        .scope("management")
        .build()
        .expect("token exchange builds");
    let value = serde_json::to_value(&request).expect("serialize token exchange");
    assert_eq!(value["scope"], "management");
}

#[tokio::test]
async fn issue_273_vault_intern_connections_path_and_response() {
    let body = json!({"data": [{"key": "github"}, {"key": "linear"}]})
        .to_string()
        .into_bytes();
    let (base_url, rx, server) = spawn_server(body, "application/json");

    let connections = attribution_client(base_url)
        .vault()
        .list_connections_for_intern("intern-1")
        .await
        .expect("connections fetch succeeds");
    assert_eq!(
        connections
            .data
            .iter()
            .map(|connection| connection.key.as_str())
            .collect::<Vec<_>>(),
        vec!["github", "linear"]
    );

    let captured = rx.recv().expect("captured request");
    assert_eq!(
        captured.line,
        "GET /api/v1/vault/interns/intern-1/connections HTTP/1.1"
    );
    let headers = captured.headers.to_ascii_lowercase();
    assert!(headers.contains("http-referer: https://example.com/app"));
    assert!(headers.contains("x-title: drift-test"));
    assert!(headers.contains("x-openrouter-title: drift-test"));
    assert!(headers.contains("x-openrouter-categories: docs,sdk"));
    server.join().expect("server joins");
}

#[tokio::test]
async fn issue_273_user_model_sort_query() {
    let body = json!({"data": []}).to_string().into_bytes();
    let (base_url, rx, server) = spawn_server(body, "application/json");

    let params = ListUserModelsParams::builder()
        .sort("coding-high-to-low")
        .build()
        .expect("params build");
    let models = client(base_url)
        .models()
        .list_for_user_with_params(&params)
        .await
        .expect("user models fetch succeeds");
    assert!(models.is_empty());

    let captured = rx.recv().expect("captured request");
    assert_eq!(
        captured.line,
        "GET /api/v1/models/user?sort=coding-high-to-low HTTP/1.1"
    );
    server.join().expect("server joins");
}

#[tokio::test]
async fn issue_273_vault_store_keeps_attribution_headers() {
    let body = json!({
        "data": {
            "name": "TOKEN",
            "hosts": ["api.example.com"],
            "fingerprint": "sha256:abc",
            "created_at": "2026-10-19T00:00:00Z"
        }
    })
    .to_string()
    .into_bytes();
    let (base_url, rx, server) = spawn_server(body, "application/json");

    let request = openrouter_rs::api::vault::VaultSecretWriteRequest::builder()
        .value("secret")
        .hosts(["api.example.com"])
        .build()
        .expect("vault write builds");
    let stored = attribution_client(base_url)
        .vault()
        .store("TOKEN", &request)
        .await
        .expect("vault store succeeds");
    assert_eq!(stored.data.name, "TOKEN");

    let captured = rx.recv().expect("captured request");
    assert_eq!(captured.line, "PUT /api/v1/vault/secrets/TOKEN HTTP/1.1");
    let headers = captured.headers.to_ascii_lowercase();
    assert!(headers.contains("authorization: bearer test-key"));
    assert!(headers.contains("http-referer: https://example.com/app"));
    assert!(headers.contains("x-title: drift-test"));
    assert!(headers.contains("x-openrouter-categories: docs,sdk"));
    server.join().expect("server joins");
}
