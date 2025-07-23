use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provisioning_key = dotenv!("OPENROUTER_PROVISIONING_KEY");
    let client = OpenRouterClient::builder()
        .provisioning_key(provisioning_key)
        .build()?;

    let specific_api_key = client.get_api_key("your_key_hash").await?;
    println!("{specific_api_key:?}");

    Ok(())
}
