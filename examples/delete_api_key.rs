use openrouter_rs::OpenRouterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");
    let client = OpenRouterClient::new(api_key);

    let delete_result = client.delete_api_key("your_exposed_key_hash").await?;
    println!("API key deleted: {}", delete_result);

    Ok(())
}
