use openrouter_rs::{OpenRouterClient, api::legacy::completion, error::OpenRouterError};

#[tokio::test]
async fn test_legacy_completions_domain_requires_api_key() {
    let client = OpenRouterClient::builder()
        .build()
        .expect("client should build");
    let request = completion::CompletionRequest::builder()
        .model("openai/gpt-4.1")
        .prompt("hello")
        .build()
        .expect("completion request should build");

    let result = client.legacy().completions().create(&request).await;
    assert!(matches!(result, Err(OpenRouterError::KeyNotConfigured)));
}
