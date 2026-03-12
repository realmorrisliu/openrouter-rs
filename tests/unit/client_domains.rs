use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    OpenRouterClient,
    api::{auth, chat, credits, embeddings, guardrails, messages, responses},
    error::OpenRouterError,
    types::{ModelCategory, PaginationOptions, Role, SupportedParameters},
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

#[tokio::test]
async fn test_chat_domain_requires_api_key() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");
    let request = chat::ChatCompletionRequest::builder()
        .model("openai/gpt-4.1")
        .messages(vec![chat::Message::new(Role::User, "hello")])
        .build()
        .expect("chat request should build");

    let result = client.chat().create(&request).await;
    assert!(matches!(result, Err(OpenRouterError::KeyNotConfigured)));

    let raw_stream_result = client.chat().stream(&request).await;
    assert!(matches!(
        raw_stream_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let tool_aware_result = client.chat().stream_tool_aware(&request).await;
    assert!(matches!(
        tool_aware_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let stream_result = client.chat().stream_unified(&request).await;
    assert!(matches!(
        stream_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));
}

#[tokio::test]
async fn test_responses_domain_requires_api_key() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");
    let request = responses::ResponsesRequest::builder()
        .model("openai/gpt-4.1")
        .input("hello".into())
        .build()
        .expect("responses request should build");

    let result = client.responses().create(&request).await;
    assert!(matches!(result, Err(OpenRouterError::KeyNotConfigured)));

    let raw_stream_result = client.responses().stream(&request).await;
    assert!(matches!(
        raw_stream_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let stream_result = client.responses().stream_unified(&request).await;
    assert!(matches!(
        stream_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));
}

#[tokio::test]
async fn test_messages_domain_requires_api_key() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");
    let request = messages::AnthropicMessagesRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .max_tokens(16)
        .messages(vec![messages::AnthropicMessage::user("hello")])
        .build()
        .expect("messages request should build");

    let result = client.messages().create(&request).await;
    assert!(matches!(result, Err(OpenRouterError::KeyNotConfigured)));

    let raw_stream_result = client.messages().stream(&request).await;
    assert!(matches!(
        raw_stream_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let stream_result = client.messages().stream_unified(&request).await;
    assert!(matches!(
        stream_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));
}

#[tokio::test]
async fn test_models_domain_requires_api_key() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");

    let result = client.models().list().await;
    assert!(matches!(result, Err(OpenRouterError::KeyNotConfigured)));
}

#[tokio::test]
async fn test_models_domain_renamed_methods_require_api_key() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");

    let user_models = client.models().list_user_models().await;
    assert!(matches!(
        user_models,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let model_count = client.models().get_model_count().await;
    assert!(matches!(
        model_count,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let by_category = client
        .models()
        .list_by_category(ModelCategory::Programming)
        .await;
    assert!(matches!(
        by_category,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let by_parameter = client
        .models()
        .list_by_parameters(SupportedParameters::Tools)
        .await;
    assert!(matches!(
        by_parameter,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let endpoints = client.models().list_endpoints("openai", "gpt-4.1").await;
    assert!(matches!(endpoints, Err(OpenRouterError::KeyNotConfigured)));

    let providers = client.models().list_providers().await;
    assert!(matches!(providers, Err(OpenRouterError::KeyNotConfigured)));

    let zdr_endpoints = client.models().list_zdr_endpoints().await;
    assert!(matches!(
        zdr_endpoints,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let embedding_request = embeddings::EmbeddingRequest::builder()
        .model("openai/text-embedding-3-small")
        .input("hello")
        .build()
        .expect("embedding request should build");
    let embedding = client.models().create_embedding(&embedding_request).await;
    assert!(matches!(embedding, Err(OpenRouterError::KeyNotConfigured)));

    let embedding_models = client.models().list_embedding_models().await;
    assert!(matches!(
        embedding_models,
        Err(OpenRouterError::KeyNotConfigured)
    ));
}

#[tokio::test]
async fn test_management_domain_requires_management_key() {
    let client = OpenRouterClient::builder()
        .api_key("user-key")
        .build()
        .expect("client should build");

    let result = client
        .management()
        .create_api_key("new-key", Some(10.0))
        .await;
    assert!(matches!(result, Err(OpenRouterError::KeyNotConfigured)));
}

#[test]
fn test_client_accessors_and_key_mutators_cover_public_surface() {
    let mut client = OpenRouterClient::builder()
        .build()
        .expect("client should build");

    client.set_api_key("api-key");
    client.clear_api_key();
    client.set_management_key("management-key");
    client.clear_management_key();
}

#[tokio::test]
async fn test_management_domain_remaining_methods_require_configured_key() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");

    let auth_code_request = auth::CreateAuthCodeRequest::builder()
        .callback_url("https://example.com/callback")
        .build()
        .expect("auth code request should build");
    let coinbase_request = credits::CoinbaseChargeRequest::builder()
        .amount(10.0)
        .sender("0x1234")
        .chain_id(8453)
        .build()
        .expect("coinbase request should build");
    let create_guardrail_request = guardrails::CreateGuardrailRequest::builder()
        .name("Production")
        .build()
        .expect("create guardrail request should build");
    let update_guardrail_request = guardrails::UpdateGuardrailRequest::builder()
        .name("Updated")
        .build()
        .expect("update guardrail request should build");
    let bulk_key_request = guardrails::BulkKeyAssignmentRequest::builder()
        .key_hashes(vec!["hash_1".to_string()])
        .build()
        .expect("bulk key request should build");
    let bulk_member_request = guardrails::BulkMemberAssignmentRequest::builder()
        .member_user_ids(vec!["user_1".to_string()])
        .build()
        .expect("bulk member request should build");

    assert!(matches!(
        client.management().get_current_api_key_info().await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client.management().delete_api_key("hash").await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .update_api_key("hash", Some("updated".to_string()), Some(true), Some(1.0))
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .list_api_keys(
                Some(PaginationOptions::with_offset_and_limit(1, 2)),
                Some(true)
            )
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client.management().get_api_key("hash").await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .create_auth_code(&auth_code_request)
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .create_coinbase_charge(&coinbase_request)
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client.management().get_credits().await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client.management().get_generation("gen_123").await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client.management().get_activity(Some("2026-03-11")).await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client.management().list_guardrails(None).await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .create_guardrail(&create_guardrail_request)
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client.management().get_guardrail("gr_123").await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .update_guardrail("gr_123", &update_guardrail_request)
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client.management().delete_guardrail("gr_123").await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .list_guardrail_key_assignments("gr_123", None)
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .create_guardrail_key_assignments("gr_123", &bulk_key_request)
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .delete_guardrail_key_assignments("gr_123", &bulk_key_request)
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .list_guardrail_member_assignments("gr_123", None)
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .create_guardrail_member_assignments("gr_123", &bulk_member_request)
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client
            .management()
            .delete_guardrail_member_assignments("gr_123", &bulk_member_request)
            .await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client.management().list_key_assignments(None).await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
    assert!(matches!(
        client.management().list_member_assignments(None).await,
        Err(OpenRouterError::KeyNotConfigured)
    ));
}

#[tokio::test]
async fn test_models_domain_list_endpoints_delegates_to_api_module() {
    let response_body = r#"{
        "data": {
            "id": "openai/gpt-4.1",
            "name": "GPT-4.1",
            "created": 1,
            "description": "Test model",
            "architecture": {
                "tokenizer": "cl100k_base",
                "instruct_type": "chat",
                "modality": "text->text"
            },
            "endpoints": [{
                "name": "openai",
                "context_length": 128000,
                "pricing": {"prompt": "1", "completion": "2"},
                "provider_name": "OpenAI",
                "supported_parameters": ["tools"],
                "quantization": null,
                "max_completion_tokens": 4096,
                "max_prompt_tokens": 128000,
                "status": null
            }]
        }
    }"#;
    let (base_url, rx, server) = spawn_json_server(response_body);
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("api-key")
        .build()
        .expect("client should build");

    let response = client
        .models()
        .list_endpoints("openai", "gpt-4.1")
        .await
        .expect("list_endpoints should succeed");
    assert_eq!(response.id, "openai/gpt-4.1");
    assert_eq!(response.endpoints.len(), 1);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/models/openai/gpt-4.1/endpoints HTTP/1.1"
    );
    assert!(
        captured
            .request_text
            .to_ascii_lowercase()
            .contains("authorization: bearer api-key")
            || captured
                .request_text
                .to_ascii_lowercase()
                .contains("authorization:bearer api-key"),
        "authorization header should include api key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_management_domain_list_guardrails_without_pagination_delegates() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[],"total_count":0}"#);
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .management_key("management-key")
        .build()
        .expect("client should build");

    let response = client
        .management()
        .list_guardrails(None)
        .await
        .expect("list_guardrails should succeed");
    assert_eq!(response.total_count, 0.0);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/guardrails HTTP/1.1");
    assert!(
        captured
            .request_text
            .to_ascii_lowercase()
            .contains("authorization: bearer management-key")
            || captured
                .request_text
                .to_ascii_lowercase()
                .contains("authorization:bearer management-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_management_domain_create_api_key_from_auth_code_delegates() {
    let (base_url, rx, server) =
        spawn_json_server(r#"{"key":"sk-or-v1-test","user_id":"user_123"}"#);
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .build()
        .expect("client should build");

    let response = client
        .management()
        .create_api_key_from_auth_code(
            "code-123",
            Some("verifier-456"),
            Some(auth::CodeChallengeMethod::S256),
        )
        .await
        .expect("auth code exchange should succeed");
    assert_eq!(response.key, "sk-or-v1-test");
    assert_eq!(response.user_id.as_deref(), Some("user_123"));

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "POST /api/v1/auth/keys HTTP/1.1");
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid JSON");
    assert_eq!(body["code"], "code-123");
    assert_eq!(body["code_verifier"], "verifier-456");
    assert_eq!(body["code_challenge_method"], "S256");

    server.join().expect("server thread should finish");
}
