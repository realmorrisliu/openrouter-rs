use dotenvy_macro::dotenv;
use futures_util::StreamExt;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::{Choice, Role},
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

    let response = client.stream_chat_completion(&chat_request).await?;

    response
        .filter_map(|event| async { event.ok() })
        .for_each(|event| async move {
            event
                .choices
                .into_iter()
                .filter_map(|choice| match choice {
                    Choice::Streaming(choice) => {
                        if choice.finish_reason.is_some() {
                            println!("{:?}", choice);
                        }

                        choice.delta.content
                    }
                    _ => None,
                })
                .for_each(|content| println!("{}", content));
        })
        .await;

    Ok(())
}
