use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, ContentPart, Message},
    types::Role,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = ChatCompletionRequest::builder()
        .model("openai/gpt-4o")
        .messages(vec![Message::with_parts(
            Role::User,
            vec![
                ContentPart::text("Please transcribe this audio."),
                ContentPart::input_audio("UklGRiQAAABXQVZF...", "wav"),
            ],
        )])
        .build()?;

    let response = client.send_chat_completion(&request).await?;
    println!("{}", response.choices[0].content().unwrap_or(""));

    Ok(())
}
