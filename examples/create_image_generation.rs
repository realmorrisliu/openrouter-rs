use openrouter_rs::{OpenRouterClient, api::images::ImageGenerationRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = ImageGenerationRequest::builder()
        .model("bytedance-seed/seedream-4.5")
        .prompt("A red panda astronaut floating in space, studio lighting")
        .aspect_ratio("16:9")
        .resolution("2K")
        .build()?;

    let response = client.images().create(&request).await?;
    println!("created: {}", response.created);
    println!("images: {}", response.data.len());
    if let Some(first) = response.data.first() {
        println!("first image base64 bytes: {}", first.b64_json.len());
        println!(
            "first image media type: {}",
            first.media_type.as_deref().unwrap_or("image/png")
        );
    }

    Ok(())
}
