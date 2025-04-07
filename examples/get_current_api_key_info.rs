use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder(api_key).build();

    let api_key_info = client.get_current_api_key_info().await?;
    println!("{:?}", api_key_info);

    Ok(())
}
