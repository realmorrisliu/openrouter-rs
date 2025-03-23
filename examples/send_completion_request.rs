use openrouter_rs::OpenRouterClient;
use openrouter_rs::api::completion::CompletionRequest;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");
    let client = OpenRouterClient::new(api_key);

    let completion_request =
        CompletionRequest::new("deepseek/deepseek-chat:free", "Once upon a time")
            .max_tokens(100)
            .temperature(0.7);

    let completion_response = client.send_completion_request(&completion_request).await?;
    println!("{:?}", completion_response);

    Ok(())
}
