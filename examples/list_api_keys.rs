use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::new(api_key);

    let api_keys = client.list_api_keys(Some(0.0), Some(true)).await?;
    println!("{:?}", api_keys);

    Ok(())
}
