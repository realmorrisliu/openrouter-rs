use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let api_key_info = client.get_current_api_key_info().await?;
    println!("{api_key_info:?}");

    Ok(())
}
