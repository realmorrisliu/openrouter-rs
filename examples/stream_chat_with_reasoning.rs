//! # Streaming Chat with Reasoning
//!
//! This example shows how to handle streaming responses when using reasoning
//! tokens. It demonstrates collecting all streaming events and then processing
//! them to extract both reasoning and content portions.
//!
//! ## Features Demonstrated
//!
//! - Streaming chat completion with reasoning enabled
//! - Collecting streaming events for batch processing
//! - Separating reasoning content from response content
//! - High-effort reasoning configuration
//!
//! ## Key Concepts
//!
//! - **Streaming**: Responses arrive in real-time chunks
//! - **Event Collection**: Gather all events before processing
//! - **Content Separation**: Reasoning vs. final answer content
//! - **Error Filtering**: Handle stream errors gracefully
//!
//! ## Usage
//!
//! ```bash
//! OPENROUTER_API_KEY=your_key cargo run --example stream_chat_with_reasoning
//! ```

use futures_util::StreamExt;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::{Effort, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs")
        .build()?;

    println!("=== Streaming Chat with Reasoning ===");
    let chat_request = ChatCompletionRequest::builder()
        .model("openai/o3-mini")
        .messages(vec![Message::new(
            Role::User,
            "What's bigger, 9.9 or 9.11? Think about this step by step.",
        )])
        .max_tokens(1000)
        .reasoning_effort(Effort::High)
        .build()?;

    let stream = client.stream_chat_completion(&chat_request).await?;

    // Method 1: Collect all events then process
    println!("Collecting streaming data...");
    let events: Vec<_> = stream
        .filter_map(|event| async { event.ok() })
        .collect()
        .await;

    let mut content_buffer = String::new();
    let mut reasoning_buffer = String::new();

    for event in events {
        if let Some(reasoning) = event.choices[0].reasoning() {
            reasoning_buffer.push_str(reasoning);
        }

        if let Some(content) = event.choices[0].content() {
            content_buffer.push_str(content);
        }
    }

    println!("\n=== Final Results (Method 1) ===");
    println!("Complete Content: {content_buffer}");
    println!("Complete Reasoning: {reasoning_buffer}");

    Ok(())
}
