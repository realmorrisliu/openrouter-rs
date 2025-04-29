use dotenvy_macro::dotenv;
use futures_util::StreamExt;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::Role,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let chat_request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3-0324:free")
        .messages(vec![Message::new(
            Role::User,
            "What is the meaning of life?",
        )])
        .temperature(0.7)
        .build()?;

    let stream = client.stream_chat_completion(&chat_request).await?;

    stream
        .filter_map(|event| async { event.ok() })
        .for_each(|event| async move {
            println!("{}", event.choices[0].content().unwrap());
        })
        .await;

    Ok(())
}
