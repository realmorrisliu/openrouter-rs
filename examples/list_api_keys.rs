use openrouter_rs::{OpenRouterClient, types::PaginationOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");
    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let api_keys = client
        .list_api_keys(Some(PaginationOptions::with_offset(0)), Some(true))
        .await?;
    println!("{api_keys:?}");

    Ok(())
}
