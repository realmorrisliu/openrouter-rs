use openrouter_rs::{OpenRouterClient, types::PaginationOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");

    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let members = client
        .management()
        .list_organization_members(Some(PaginationOptions::with_offset_and_limit(0, 25)))
        .await?;

    println!("member count: {}", members.total_count);
    for member in members.data {
        println!(
            "{} {} <{}>",
            member.first_name.unwrap_or_default(),
            member.last_name.unwrap_or_default(),
            member.email
        );
    }

    Ok(())
}
