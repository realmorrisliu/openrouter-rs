use openrouter_rs::{OpenRouterClient, api::generation::GenerationFeedbackRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let client = OpenRouterClient::builder()
        .management_key(std::env::var("OPENROUTER_MANAGEMENT_KEY")?)
        .build()?;
    let request = GenerationFeedbackRequest::builder()
        .generation_id("gen-123")
        .category("incorrect_response")
        .comment("The response repeated the same paragraph.")
        .build()?;

    let response = client
        .management()
        .submit_generation_feedback(&request)
        .await?;
    println!("feedback recorded: {}", response.success);
    Ok(())
}
