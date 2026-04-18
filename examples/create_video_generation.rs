use openrouter_rs::{OpenRouterClient, api::videos::VideoGenerationRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = VideoGenerationRequest::builder()
        .model("google/veo-3.1")
        .prompt("A cinematic flyover of snowy mountains at sunrise")
        .aspect_ratio("16:9")
        .duration(8)
        .resolution("720p")
        .build()?;

    let response = client.videos().create(&request).await?;
    println!("job id: {}", response.id);
    println!("status: {}", response.status);
    println!("polling url: {}", response.polling_url);

    Ok(())
}
