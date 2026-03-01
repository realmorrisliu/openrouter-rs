use openrouter_rs::{
    OpenRouterClient,
    api::{chat, messages, responses},
    error::OpenRouterError,
    types::Role,
};

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
