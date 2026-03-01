use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");
    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let updated_api_key = client
        .update_api_key(
            "your_key_hash",
            Some("New API Key Name".to_string()),
            Some(false),
            Some(200.0),
        )
        .await?;
    println!("{updated_api_key:?}");

    Ok(())
}
