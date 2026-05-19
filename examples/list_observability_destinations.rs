use openrouter_rs::{OpenRouterClient, types::PaginationOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");

    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let destinations = client
        .management()
        .list_observability_destinations(
            Some(PaginationOptions::with_offset_and_limit(0, 25)),
            None,
        )
        .await?;

    println!("destination count: {}", destinations.total_count);
    for destination in destinations.data {
        println!(
            "{} {} ({})",
            destination.destination_type,
            destination.name.unwrap_or_else(|| "<unnamed>".to_string()),
            destination.id
        );
    }

    Ok(())
}
