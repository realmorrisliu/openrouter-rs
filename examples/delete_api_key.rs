use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");
    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let delete_result = client.delete_api_key("your_exposed_key_hash").await?;
    println!("API key deleted: {delete_result}");

    Ok(())
}
