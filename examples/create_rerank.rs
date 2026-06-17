use openrouter_rs::{OpenRouterClient, api::rerank::RerankRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = RerankRequest::builder()
        .model("cohere/rerank-v3.5")
        .query("Which city is the capital of France?")
        .documents(vec![
            "Berlin is the capital of Germany.".to_string(),
            "Paris is the capital of France.".to_string(),
            "Tokyo is the capital of Japan.".to_string(),
        ])
        .top_n(2)
        .build()?;

    let response = client.rerank().create(&request).await?;
    for result in response.results {
        println!(
            "index={} score={} text={} image={}",
            result.index,
            result.relevance_score,
            result.document.text.as_deref().unwrap_or("-"),
            result.document.image.as_deref().unwrap_or("-")
        );
    }

    Ok(())
}
