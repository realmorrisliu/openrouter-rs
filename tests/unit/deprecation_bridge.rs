use openrouter_rs::{OpenRouterClient, error::OpenRouterError};

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
