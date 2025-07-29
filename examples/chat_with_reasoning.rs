//! # Chat Completion with Reasoning Tokens
//!
//! This example demonstrates how to use OpenRouter's reasoning tokens feature
//! for chain-of-thought processing with various models and configurations.
//!
//! ## Features Demonstrated
//!
//! - Basic chat completion without reasoning
//! - High-effort reasoning with OpenAI o3-mini
//! - Reasoning token limits with Anthropic models
//! - Excluded reasoning (internal-only processing)
//!
//! ## Models Used
//!
//! - `deepseek/deepseek-chat-v3-0324:free` - Free model baseline
//! - `openai/o3-mini` - High-effort reasoning model
//! - `anthropic/claude-3.7-sonnet:thinking` - Anthropic reasoning model
//! - `deepseek/deepseek-r1` - DeepSeek reasoning model
//!
//! ## Usage
//!
//! Set your OpenRouter API key in a `.env` file:
//! ```bash
//! OPENROUTER_API_KEY=your_key_here
//! ```
//!
//! Then run:
//! ```bash
//! cargo run --example chat_with_reasoning
//! ```

use dotenvy_macro::dotenv;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::{Effort, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs")
        .build()?;

    println!("=== Example 1: Basic Reasoning with Default Settings ===");
    // Test with a simple model, without enabling reasoning
    let chat_request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3-0324:free")
        .messages(vec![Message::new(
            Role::User,
            "Which is bigger: 9.11 or 9.9? Please think this through step by step.",
        )])
        .max_tokens(1000)
        .build()?;

    let chat_response = client.send_chat_completion(&chat_request).await?;
    let content = chat_response.choices[0].content().unwrap_or("");
    let reasoning = chat_response.choices[0].reasoning().unwrap_or("");

    println!("Content: {content}");
    println!("Reasoning: {reasoning}");

    println!("\n=== Example 2: High Effort Reasoning ===");
    let chat_request = ChatCompletionRequest::builder()
        .model("openai/o3-mini")
        .messages(vec![Message::new(
            Role::User,
            "How would you build the world's tallest skyscraper?",
        )])
        .max_tokens(2000)
        .reasoning_effort(Effort::High)
        .build()?;

    let chat_response = client.send_chat_completion(&chat_request).await?;
    let content = chat_response.choices[0].content().unwrap_or("");
    let reasoning = chat_response.choices[0].reasoning().unwrap_or("");

    println!("Content: {content}");
    println!(
        "Reasoning (first 200 chars): {}",
        if reasoning.len() > 200 {
            &reasoning[..200]
        } else {
            reasoning
        }
    );

    println!("\n=== Example 3: Max Tokens Reasoning ===");
    let chat_request = ChatCompletionRequest::builder()
        .model("anthropic/claude-3.7-sonnet:thinking")
        .messages(vec![Message::new(
            Role::User,
            "What's the most efficient algorithm for sorting a large dataset?",
        )])
        .max_tokens(3000)
        .reasoning_max_tokens(2000)
        .build()?;

    let chat_response = client.send_chat_completion(&chat_request).await?;
    let content = chat_response.choices[0].content().unwrap_or("");
    let reasoning = chat_response.choices[0].reasoning().unwrap_or("");

    println!("Content: {content}");
    println!(
        "Reasoning (first 200 chars): {}",
        if reasoning.len() > 200 {
            &reasoning[..200]
        } else {
            reasoning
        }
    );

    println!("\n=== Example 4: Excluded Reasoning (Internal Only) ===");
    let chat_request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-r1")
        .messages(vec![Message::new(
            Role::User,
            "Explain quantum computing in simple terms.",
        )])
        .max_tokens(500)
        .exclude_reasoning()
        .build()?;

    let chat_response = client.send_chat_completion(&chat_request).await?;
    let content = chat_response.choices[0].content().unwrap_or("");
    let reasoning = chat_response.choices[0].reasoning();

    println!("Content: {content}");
    println!("Reasoning (should be None): {reasoning:?}");

    Ok(())
}
