use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provisioning_key = std::env::var("OPENROUTER_PROVISIONING_KEY")
        .expect("OPENROUTER_PROVISIONING_KEY must be set");
    let client = OpenRouterClient::builder()
        .provisioning_key(provisioning_key)
        .build()?;

    let api_keys = client.list_api_keys(Some(0.0), Some(true)).await?;
    println!("{api_keys:?}");

    Ok(())
}
