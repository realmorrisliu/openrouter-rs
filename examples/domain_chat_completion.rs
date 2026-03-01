use openrouter_rs::{OpenRouterClient, api::chat::*, types::Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");

    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3-0324:free")
        .messages(vec![Message::new(
            Role::User,
            "Reply with one short sentence about Rust.",
        )])
        .max_tokens(64)
        .temperature(0.2)
        .build()?;

    let response = client.chat().create(&request).await?;
    println!("{response:?}");

    Ok(())
}
