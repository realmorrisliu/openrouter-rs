use super::test_utils::{create_test_client, rate_limit_delay};

use openrouter_rs::{
    api::chat::{ChatCompletionRequestBuilder, Message},
    error::OpenRouterError,
    types::{
        Effort, Role,
        completion::{Choice, CompletionsResponse},
    },
};

const TEST_MODEL: &str = "deepseek/deepseek-chat-v3-0324:free";

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_basic_chat_completion() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let request = ChatCompletionRequestBuilder::default()
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

#[allow(clippy::result_large_err)]
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

const REASONING_MODEL: &str = "openai/o3-mini";

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_basic_reasoning() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let request = ChatCompletionRequestBuilder::default()
        .model(REASONING_MODEL)
        .messages(vec![Message::new(
            Role::User,
            "Which is bigger: 9.11 or 9.9? Think step by step.",
        )])
        .max_tokens(500)
        .enable_reasoning()
        .build()?;

    let response = client.send_chat_completion(&request).await?;
    validate_chat_response(&response)?;

    // Test reasoning content
    let reasoning = response.choices[0].reasoning();
    assert!(reasoning.is_some(), "Reasoning should be present");
    assert!(
        !reasoning.unwrap().is_empty(),
        "Reasoning should not be empty"
    );

    println!(
        "Reasoning test passed, reasoning length: {}",
        reasoning.unwrap().len()
    );
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_reasoning_effort_levels() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;

    for effort in [Effort::Low, Effort::Medium, Effort::High] {
        rate_limit_delay().await;

        let request = ChatCompletionRequestBuilder::default()
            .model(REASONING_MODEL)
            .messages(vec![Message::new(
                Role::User,
                "Explain the theory of relativity briefly.",
            )])
            .max_tokens(300)
            .reasoning_effort(effort.clone())
            .build()?;

        let response = client.send_chat_completion(&request).await?;
        validate_chat_response(&response)?;

        let reasoning = response.choices[0].reasoning();
        assert!(
            reasoning.is_some(),
            "Reasoning should be present for {effort:?} effort"
        );

        println!(
            "Effort {:?} test passed, reasoning length: {}",
            effort,
            reasoning.unwrap().len()
        );
    }

    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_reasoning_max_tokens() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let request = ChatCompletionRequestBuilder::default()
        .model("anthropic/claude-3.7-sonnet")
        .messages(vec![Message::new(
            Role::User,
            "What's the most efficient sorting algorithm?",
        )])
        .max_tokens(2000)
        .reasoning_max_tokens(1000)
        .build()?;

    let response = client.send_chat_completion(&request).await?;
    validate_chat_response(&response)?;

    let reasoning = response.choices[0].reasoning();
    assert!(reasoning.is_some(), "Reasoning should be present");

    println!(
        "Max tokens test passed, reasoning length: {}",
        reasoning.unwrap().len()
    );
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_excluded_reasoning() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let request = ChatCompletionRequestBuilder::default()
        .model("deepseek/deepseek-r1")
        .messages(vec![Message::new(
            Role::User,
            "Explain quantum computing in simple terms.",
        )])
        .max_tokens(300)
        .exclude_reasoning()
        .build()?;

    let response = client.send_chat_completion(&request).await?;
    validate_chat_response(&response)?;

    let reasoning = response.choices[0].reasoning();
    // When excluded, reasoning should be None or empty
    assert!(
        reasoning.is_none() || reasoning.unwrap().is_empty(),
        "Reasoning should be excluded from response"
    );

    println!("Excluded reasoning test passed");
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_legacy_include_reasoning() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let request = ChatCompletionRequestBuilder::default()
        .model(REASONING_MODEL)
        .messages(vec![Message::new(Role::User, "What is 2+2?")])
        .max_tokens(100)
        .include_reasoning(true)
        .build()?;

    let response = client.send_chat_completion(&request).await?;
    validate_chat_response(&response)?;

    // With legacy include_reasoning: true, reasoning should be present
    let reasoning = response.choices[0].reasoning();
    assert!(
        reasoning.is_some(),
        "Reasoning should be present with include_reasoning: true"
    );

    println!("Legacy include_reasoning test passed");
    Ok(())
}
