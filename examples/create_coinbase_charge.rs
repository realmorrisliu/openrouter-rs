use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;
use openrouter_rs::api::credits::CoinbaseChargeRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let coinbase_request = CoinbaseChargeRequest::new(1.1, "your_ethereum_address", 1);
    let coinbase_response = client.create_coinbase_charge(&coinbase_request).await?;
    println!("{coinbase_response:?}");

    Ok(())
}
