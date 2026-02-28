use futures_util::StreamExt;
use openrouter_rs::{OpenRouterClient, api::responses::ResponsesRequest};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = ResponsesRequest::builder()
        .model("openai/gpt-5")
        .input(json!([{
            "role": "user",
            "content": "Write a short haiku about Rust."
        }]))
        .build()?;

    let stream = client.stream_response(&request).await?;

    stream
        .filter_map(|event| async { event.ok() })
        .for_each(|event| async move {
            println!("event: {}", event.event_type);
        })
        .await;

    Ok(())
}
