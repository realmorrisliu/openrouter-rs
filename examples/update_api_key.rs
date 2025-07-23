use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provisioning_key = dotenv!("OPENROUTER_PROVISIONING_KEY");
    let client = OpenRouterClient::builder()
        .provisioning_key(provisioning_key)
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
