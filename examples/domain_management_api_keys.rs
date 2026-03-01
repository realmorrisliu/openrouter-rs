use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");

    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let keys = client
        .management()
        .list_api_keys(Some(0.0), Some(false))
        .await?;
    println!("{keys:?}");

    Ok(())
}
