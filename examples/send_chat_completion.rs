use dotenvy_macro::dotenv;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::Role,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs")
        .build();

    let chat_request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3-0324:free")
        .messages(vec![Message::new(
            Role::User,
            "What is the meaning of life?",
        )])
        .max_tokens(100)
        .temperature(0.7)
        .build()?;

    let chat_response = client.send_chat_completion(&chat_request).await?;
    println!("{:?}", chat_response);

    Ok(())
}
