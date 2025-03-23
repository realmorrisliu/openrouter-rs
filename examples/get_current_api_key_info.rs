use openrouter_rs::OpenRouterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");
    let client = OpenRouterClient::new(api_key);

    let api_key_info = client.get_current_api_key_info().await?;
    println!("{:?}", api_key_info);

    Ok(())
}
