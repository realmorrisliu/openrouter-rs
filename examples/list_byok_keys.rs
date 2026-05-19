use openrouter_rs::{OpenRouterClient, types::PaginationOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");

    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let keys = client
        .management()
        .list_byok_keys(
            Some(PaginationOptions::with_offset_and_limit(0, 25)),
            None,
            None,
        )
        .await?;

    println!("BYOK key count: {}", keys.total_count);
    for key in keys.data {
        println!("{} {} ({})", key.provider, key.label, key.id);
    }

    Ok(())
}
