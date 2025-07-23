use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;
use openrouter_rs::api::completion::CompletionRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let completion_request = CompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3-0324:free")
        .models(["deepseek/deepseek-chat-v3-0324:free"])
        .prompt("Once upon a time")
        .max_tokens(100)
        .temperature(0.7)
        .build()?;

    let completion_response = client.send_completion_request(&completion_request).await?;
    let content = completion_response.choices[0].content().unwrap();
    println!("{content:?}");

    Ok(())
}
