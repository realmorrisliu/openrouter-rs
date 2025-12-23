use dotenvy_macro::dotenv;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::Role,
};

/// Test example for Grok model (Issue #6)
/// This verifies that Grok model responses can be properly deserialized
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs-grok-test")
        .build()?;

    println!("Testing Grok model: x-ai/grok-code-fast-1");
    println!("=========================================\n");

    let chat_request = ChatCompletionRequest::builder()
        .model("x-ai/grok-code-fast-1")
        .messages(vec![Message::new(
            Role::User,
            "Say 'Hello from Rust!' in exactly those words.",
        )])
        .max_tokens(50)
        .temperature(0.1)
        .build()?;

    let response = client.send_chat_completion(&chat_request).await?;

    println!("Response ID: {}", response.id);
    println!("Model: {}", response.model);
    println!("Choices count: {}", response.choices.len());

    if let Some(choice) = response.choices.first() {
        println!("\n--- Choice Details ---");
        println!("Index: {:?}", choice.index());
        println!("Finish reason: {:?}", choice.finish_reason());
        println!("Native finish reason: {:?}", choice.native_finish_reason());
        println!("Content: {:?}", choice.content());
        println!("Reasoning: {:?}", choice.reasoning());
        println!("Reasoning details: {:?}", choice.reasoning_details());
        println!("Logprobs: {:?}", choice.logprobs());
    }

    if let Some(usage) = &response.usage {
        println!("\n--- Usage ---");
        println!("Prompt tokens: {}", usage.prompt_tokens);
        println!("Completion tokens: {}", usage.completion_tokens);
        println!("Total tokens: {}", usage.total_tokens);
    }

    println!("\n=========================================");
    println!("Grok model test completed successfully!");

    Ok(())
}
