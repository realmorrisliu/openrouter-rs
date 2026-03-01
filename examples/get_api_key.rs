use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");
    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let specific_api_key = client.get_api_key("your_key_hash").await?;
    println!("{specific_api_key:?}");

    Ok(())
}
