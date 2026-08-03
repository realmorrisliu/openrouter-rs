use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenRouterClient::builder()
        .management_key(std::env::var("OPENROUTER_MANAGEMENT_KEY")?)
        .build()?;

    for group in client.management().list_scim_groups(None).await?.data {
        println!("{}\t{}", group.id, group.display_name);
    }

    Ok(())
}
