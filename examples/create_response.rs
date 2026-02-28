use openrouter_rs::{OpenRouterClient, api::responses::ResponsesRequest};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = ResponsesRequest::builder()
        .model("openai/gpt-5")
        .input(json!([{
            "role": "user",
            "content": "Say hello in one sentence."
        }]))
        .build()?;

    let response = client.create_response(&request).await?;
    println!("response id: {:?}", response.id);
    println!("status: {:?}", response.status);

    Ok(())
}
