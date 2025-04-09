use super::test_utils::{create_test_client, rate_limit_delay};

use openrouter_rs::{
    api::chat::{ChatCompletionRequestBuilder, Message},
    error::OpenRouterError,
    types::{
        Role,
        completion::{Choice, CompletionsResponse},
    },
};

const TEST_MODEL: &str = "deepseek/deepseek-chat-v3-0324:free";

#[tokio::test]
async fn test_basic_chat_completion() -> Result<(), OpenRouterError> {
    let client = create_test_client();
    rate_limit_delay().await;

    let request = ChatCompletionRequestBuilder::new()
        .model(TEST_MODEL)
        .messages(vec![Message::new(
            Role::User,
            "Please reply simply with 'Hello' in English",
        )])
        .max_tokens(10)
        .temperature(0.1)
        .build()?;

    let response = client.send_chat_completion(&request).await?;
    validate_chat_response(&response)?;

    println!(
        "Test passed, model response: {:?}",
        get_first_content(&response)
    );
    Ok(())
}

fn validate_chat_response(response: &CompletionsResponse) -> Result<(), OpenRouterError> {
    assert!(!response.id.is_empty(), "Response ID should not be empty");

    let test_model_name = response.model.split(':').next().unwrap_or(&response.model);
    assert_eq!(response.model, test_model_name, "Model name mismatch");

    let content = get_first_content(response);
    assert!(!content.is_empty(), "Response content should not be empty");
    assert!(
        content.contains("Hello"),
        "Response should contain prompt content"
    );

    Ok(())
}

fn get_first_content(response: &CompletionsResponse) -> &str {
    match &response.choices[0] {
        Choice::NonStreaming(c) => c.message.content.as_deref().unwrap_or_default(),
        Choice::Streaming(c) => c.delta.content.as_deref().unwrap_or_default(),
        Choice::NonChat(c) => c.text.as_str(),
    }
}
