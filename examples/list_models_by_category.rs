use dotenvy_macro::dotenv;
use openrouter_rs::{OpenRouterClient, types::ModelCategory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let models = client
        .list_models_by_category(ModelCategory::Programming)
        .await?;
    println!("{:?}", models);

    Ok(())
}
