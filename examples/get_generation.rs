use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;
use openrouter_rs::api::generation::GenerationRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::new(api_key);

    let generation_request = GenerationRequest::new("your_generation_id");
    let generation_data = client.get_generation(&generation_request).await?;
    println!("{:?}", generation_data);

    Ok(())
}
