use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provisioning_key = std::env::var("OPENROUTER_PROVISIONING_KEY")
        .expect("OPENROUTER_PROVISIONING_KEY must be set");
    let client = OpenRouterClient::builder()
        .provisioning_key(provisioning_key)
        .build()?;

    let delete_result = client.delete_api_key("your_exposed_key_hash").await?;
    println!("API key deleted: {delete_result}");

    Ok(())
}
