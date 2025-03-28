use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;
use openrouter_rs::api::completion::CompletionRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::new(api_key);

    let completion_request =
        CompletionRequest::new("deepseek/deepseek-chat:free", "Once upon a time")
            .max_tokens(100)
            .temperature(0.7);

    let completion_response = client.send_completion_request(&completion_request).await?;
    println!("{:?}", completion_response);

    Ok(())
}
