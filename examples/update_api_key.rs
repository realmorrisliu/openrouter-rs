use openrouter_rs::{OpenRouterClient, api::api_keys::UpdateApiKeyRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");
    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let request = UpdateApiKeyRequest::builder()
        .name("New API Key Name")
        .disabled(false)
        .limit(200.0) // Use clear_limit() instead to remove the spending limit.
        .build()?;
    let updated_api_key = client
        .management()
        .update_api_key_with_request("your_key_hash", &request)
        .await?;
    println!("{updated_api_key:?}");

    Ok(())
}
