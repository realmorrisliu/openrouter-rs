use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provisioning_key = std::env::var("OPENROUTER_PROVISIONING_KEY")
        .expect("OPENROUTER_PROVISIONING_KEY must be set");
    let client = OpenRouterClient::builder()
        .provisioning_key(provisioning_key)
        .build()?;

    let new_api_key = client.create_api_key("My New API Key", Some(100.0)).await?;
    println!("{new_api_key:?}");

    Ok(())
}
