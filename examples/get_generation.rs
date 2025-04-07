use dotenvy_macro::dotenv;
use openrouter_rs::{OpenRouterClient, api::completion::CompletionRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder(api_key).build();

    let completion_request = CompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3-0324:free")
        .prompt("Once upon a time")
        .max_tokens(100)
        .temperature(0.7)
        .build()?;

    let completion_response = client.send_completion_request(&completion_request).await?;

    println!("{:?}", completion_response);

    // Wait for the completion to finish
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    println!("Wait for completion");

    let generation_id = completion_response.id;
    let generation_data = client.get_generation(&generation_id).await?;
    println!("{:?}", generation_data);

    Ok(())
}
