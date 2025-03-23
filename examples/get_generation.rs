use openrouter_rs::OpenRouterClient;
use openrouter_rs::api::generation::GenerationRequest;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");
    let client = OpenRouterClient::new(api_key);

    let generation_request = GenerationRequest::new("your_generation_id");
    let generation_data = client.get_generation(&generation_request).await?;
    println!("{:?}", generation_data);

    Ok(())
}
