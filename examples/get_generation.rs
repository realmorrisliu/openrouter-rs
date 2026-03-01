use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::Role,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3-0324:free")
        .messages(vec![Message::new(Role::User, "Once upon a time")])
        .max_tokens(100)
        .temperature(0.7)
        .build()?;

    let response = client.chat().create(&request).await?;

    println!("{response:?}");

    // Wait for the completion to finish
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    println!("Wait for completion");

    let generation_id = response.id;
    let generation_data = client.get_generation(&generation_id).await?;
    println!("{generation_data:?}");

    Ok(())
}
