use openrouter_rs::{OpenRouterClient, api::interns::ListInternsParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenRouterClient::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY")?)
        .build()?;
    let params = ListInternsParams::builder().limit(50).build()?;
    let page = client.interns().list(&params).await?;
    for intern in page.data {
        println!("{}: {}", intern.name, intern.status);
    }
    Ok(())
}
