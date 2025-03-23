use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::Role,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");
    let client = OpenRouterClient::new(api_key);

    let chat_request = ChatCompletionRequest::new(
        "deepseek/deepseek-chat:free",
        vec![Message::new(Role::User, "What is the meaning of life?")],
    )
    .max_tokens(100)
    .temperature(0.7);

    let chat_response = client.send_chat_completion(&chat_request).await?;
    println!("{:?}", chat_response);

    Ok(())
}
