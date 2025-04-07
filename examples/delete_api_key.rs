use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder(api_key).build();

    let delete_result = client.delete_api_key("your_exposed_key_hash").await?;
    println!("API key deleted: {}", delete_result);

    Ok(())
}
