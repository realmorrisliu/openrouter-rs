use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::new(api_key);

    let new_api_key = client.create_api_key("My New API Key", Some(100.0)).await?;
    println!("{:?}", new_api_key);

    Ok(())
}
