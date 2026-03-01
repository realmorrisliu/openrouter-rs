use openrouter_rs::{
    OpenRouterClient,
    api::{chat, messages, responses},
    error::OpenRouterError,
    types::{PaginationOptions, Role},
};

#[tokio::test]
async fn test_flat_05_style_inference_surface_requires_api_key() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");

    let chat_request = chat::ChatCompletionRequest::builder()
        .model("openai/gpt-4.1")
        .messages(vec![chat::Message::new(Role::User, "hello")])
        .build()
        .expect("chat request should build");

    let responses_request = responses::ResponsesRequest::builder()
        .model("openai/gpt-4.1")
        .input("hello".into())
        .build()
        .expect("responses request should build");

    let result = client.send_chat_completion(&chat_request).await;
    assert!(matches!(result, Err(OpenRouterError::KeyNotConfigured)));

    let stream = client.stream_response(&responses_request).await;
    assert!(matches!(stream, Err(OpenRouterError::KeyNotConfigured)));

    let models = client.list_models().await;
    assert!(matches!(models, Err(OpenRouterError::KeyNotConfigured)));
}

#[tokio::test]
async fn test_domain_06_style_inference_surface_requires_api_key() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");

    let chat_request = chat::ChatCompletionRequest::builder()
        .model("openai/gpt-4.1")
        .messages(vec![chat::Message::new(Role::User, "hello")])
        .build()
        .expect("chat request should build");

    let responses_request = responses::ResponsesRequest::builder()
        .model("openai/gpt-4.1")
        .input("hello".into())
        .build()
        .expect("responses request should build");

    let messages_request = messages::AnthropicMessagesRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .max_tokens(16)
        .messages(vec![messages::AnthropicMessage::user("hello")])
        .build()
        .expect("messages request should build");

    let chat_result = client.chat().create(&chat_request).await;
    assert!(matches!(
        chat_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let responses_stream = client.responses().stream(&responses_request).await;
    assert!(matches!(
        responses_stream,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let messages_result = client.messages().create(&messages_request).await;
    assert!(matches!(
        messages_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let models = client.models().list().await;
    assert!(matches!(models, Err(OpenRouterError::KeyNotConfigured)));
}

#[tokio::test]
async fn test_flat_05_style_management_surface_requires_management_key() {
    let client = OpenRouterClient::builder()
        .api_key("user-key")
        .build()
        .expect("client should build");

    let created = client.create_api_key("migration-smoke", Some(10.0)).await;
    assert!(matches!(created, Err(OpenRouterError::KeyNotConfigured)));

    let listed = client
        .list_api_keys(Some(PaginationOptions::with_offset(0)), Some(false))
        .await;
    assert!(matches!(listed, Err(OpenRouterError::KeyNotConfigured)));

    let activity = client.get_activity(None).await;
    assert!(matches!(activity, Err(OpenRouterError::KeyNotConfigured)));
}

#[tokio::test]
async fn test_domain_06_style_management_surface_requires_management_key() {
    let client = OpenRouterClient::builder()
        .api_key("user-key")
        .build()
        .expect("client should build");

    let management = client.management();

    let created = management
        .create_api_key("migration-smoke", Some(10.0))
        .await;
    assert!(matches!(created, Err(OpenRouterError::KeyNotConfigured)));

    let listed = management
        .list_api_keys(Some(PaginationOptions::with_offset(0)), Some(false))
        .await;
    assert!(matches!(listed, Err(OpenRouterError::KeyNotConfigured)));

    let activity = management.get_activity(None).await;
    assert!(matches!(activity, Err(OpenRouterError::KeyNotConfigured)));
}

#[tokio::test]
#[cfg(feature = "legacy-completions")]
async fn test_legacy_completion_surface_smoke() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");

    let request = openrouter_rs::api::legacy::completion::CompletionRequest::builder()
        .model("openai/gpt-4.1")
        .prompt("hello")
        .build()
        .expect("completion request should build");

    let legacy_result = client.send_completion_request(&request).await;
    assert!(matches!(
        legacy_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));

    let domain_result = client.legacy().completions().create(&request).await;
    assert!(matches!(
        domain_result,
        Err(OpenRouterError::KeyNotConfigured)
    ));
}
