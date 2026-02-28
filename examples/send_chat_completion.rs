//! # Basic Chat Completion
//!
//! This is the simplest example of using the OpenRouter SDK for chat completions.
//! It demonstrates the fundamental building blocks: client creation, request building,
//! and response handling.
//!
//! ## Features Demonstrated
//!
//! - Client configuration with builder pattern
//! - Basic chat completion request
//! - Response parsing and display
//! - Error handling with `?` operator
//!
//! ## Perfect for
//!
//! - Getting started with the SDK
//! - Understanding the basic request/response flow
//! - Testing your API key configuration
//! - Learning the builder pattern usage
//!
//! ## Usage
//!
//! ```bash
//! OPENROUTER_API_KEY=your_key cargo run --example send_chat_completion
//! ```

use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::Role,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs")
        .build()?;

    let chat_request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3-0324:free")
        .messages(vec![Message::new(
            Role::User,
            "What is the meaning of life?",
        )])
        .max_tokens(100)
        .temperature(0.7)
        .build()?;

    let chat_response = client.send_chat_completion(&chat_request).await?;
    let content = chat_response.choices[0].content().unwrap();
    println!("{content:?}");

    Ok(())
}
