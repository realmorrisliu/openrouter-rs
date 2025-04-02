use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::new(api_key);

    let endpoints = client
        .list_model_endpoints("deepseek", "deepseek-chat:free")
        .await?;
    println!("{:?}", endpoints);

    Ok(())
}
