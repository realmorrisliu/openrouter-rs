use openrouter_rs::OpenRouterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");
    let client = OpenRouterClient::new(api_key);

    let api_keys = client.list_api_keys(Some(0.0), Some(true)).await?;
    println!("{:?}", api_keys);

    Ok(())
}
