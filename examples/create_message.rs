use openrouter_rs::{
    OpenRouterClient,
    api::messages::{AnthropicMessage, AnthropicMessagesRequest},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = AnthropicMessagesRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .max_tokens(256)
        .messages(vec![AnthropicMessage::user("Say hello in one sentence.")])
        .build()?;

    let response = client.create_message(&request).await?;
    println!("message id: {:?}", response.id);
    println!("stop reason: {:?}", response.stop_reason);
    println!("content blocks: {}", response.content.len());

    Ok(())
}
