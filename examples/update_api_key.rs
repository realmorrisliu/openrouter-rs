use openrouter_rs::OpenRouterClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");
    let client = OpenRouterClient::new(api_key);

    let updated_api_key = client
        .update_api_key(
            "your_key_hash",
            Some("New API Key Name".to_string()),
            Some(false),
            Some(200.0),
        )
        .await?;
    println!("{:?}", updated_api_key);

    Ok(())
}
