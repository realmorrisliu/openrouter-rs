use openrouter_rs::{
    OpenRouterClient,
    api::chat::*,
    types::{OpenRouterExperimentalMetadata, Role},
};

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
        .experimental_metadata(OpenRouterExperimentalMetadata::Enabled)
        .build()?;

    let response = client.chat().create(&request).await?;
    println!("{response:?}");
    if let Some(details) = response
        .usage
        .as_ref()
        .and_then(|usage| usage.prompt_tokens_details.as_ref())
    {
        println!(
            "Cached prompt tokens: {:?}; cache writes: {:?}",
            details.cached_tokens, details.cache_write_tokens
        );
    }

    Ok(())
}
