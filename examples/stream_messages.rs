use futures_util::StreamExt;
use openrouter_rs::{
    OpenRouterClient,
    api::messages::{AnthropicMessage, AnthropicMessagesRequest, AnthropicMessagesStreamEvent},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = AnthropicMessagesRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .max_tokens(256)
        .messages(vec![AnthropicMessage::user(
            "Write a short haiku about Rust.",
        )])
        .build()?;

    let mut stream = client.stream_messages(&request).await?;

    while let Some(item) = stream.next().await {
        let event = item?;
        match event.data {
            AnthropicMessagesStreamEvent::ContentBlockDelta { delta, .. } => {
                if delta["type"] == "text_delta" {
                    if let Some(text) = delta["text"].as_str() {
                        print!("{text}");
                    }
                }
            }
            AnthropicMessagesStreamEvent::MessageStop => {
                println!();
                println!("done");
            }
            _ => {}
        }
    }

    Ok(())
}
