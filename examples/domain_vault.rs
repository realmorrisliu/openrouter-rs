use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenRouterClient::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY")?)
        .build()?;
    let intern_id = std::env::var("OPENROUTER_INTERN_ID")?;

    for connection in client
        .vault()
        .list_connections_for_intern(&intern_id)
        .await?
        .data
    {
        println!("{}", connection.key);
    }

    Ok(())
}
