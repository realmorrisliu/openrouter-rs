use openrouter_rs::{OpenRouterClient, api::interns::ListInternsParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenRouterClient::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY")?)
        .build()?;
    let params = ListInternsParams::builder().limit(50).build()?;
    let page = client.interns().list(&params).await?;
    for intern in &page.data {
        println!("{}: {}", intern.name, intern.status);
    }

    if let Some(intern) = page.data.first()
        && let Some(icon_sha256) = intern.icon_sha256.as_deref()
    {
        let icon = client
            .interns()
            .get_icon(&intern.id, Some(icon_sha256))
            .await?;
        println!(
            "{} icon: {} ({} bytes)",
            intern.name,
            icon.content_type.as_deref().unwrap_or("unknown"),
            icon.bytes.len()
        );
    }

    Ok(())
}
