use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");
    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let new_api_key = client.create_api_key("My New API Key", Some(100.0)).await?;
    println!("{new_api_key:?}");

    Ok(())
}
