use openrouter_rs::{OpenRouterClient, api::embeddings::EmbeddingRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = EmbeddingRequest::builder()
        .model("openai/text-embedding-3-large")
        .input("OpenRouter Rust SDK embeddings example")
        .build()?;

    let response = client.create_embedding(&request).await?;
    println!("model: {}", response.model);
    println!("vectors: {}", response.data.len());

    Ok(())
}
