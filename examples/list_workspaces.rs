use openrouter_rs::{OpenRouterClient, types::PaginationOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");

    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let workspaces = client
        .management()
        .list_workspaces(Some(PaginationOptions::with_offset_and_limit(0, 25)))
        .await?;

    println!("workspace count: {}", workspaces.total_count);
    for workspace in workspaces.data {
        println!("{} ({})", workspace.name, workspace.slug);
    }

    Ok(())
}
