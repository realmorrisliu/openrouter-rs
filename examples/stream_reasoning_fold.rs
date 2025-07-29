use dotenvy_macro::dotenv;
use futures_util::StreamExt;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::{Effort, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs")
        .build()?;

    println!("=== Streaming Chat with Reasoning (Fold Method) ===");
    let chat_request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-r1")
        .messages(vec![Message::new(
            Role::User,
            "What's bigger, 9.9 or 9.11? Think about this step by step.",
        )])
        .max_tokens(1000)
        .reasoning_effort(Effort::High)
        .build()?;

    let stream = client.stream_chat_completion(&chat_request).await?;

    // Method 2: Use fold to accumulate data
    println!("Processing streaming data with fold...");
    let (content_buffer, reasoning_buffer) = stream
        .filter_map(|event| async { event.ok() })
        .fold(
            (String::new(), String::new()),
            |(mut content, mut reasoning), event| async move {
                if let Some(r) = event.choices[0].reasoning() {
                    reasoning.push_str(r);
                }

                if let Some(c) = event.choices[0].content() {
                    content.push_str(c);
                }

                (content, reasoning)
            },
        )
        .await;

    println!("\n=== Final Results (Fold Method) ===");
    println!("Complete Content: {content_buffer}");
    println!("Complete Reasoning: {reasoning_buffer}");

    println!("\n=== Statistics ===");
    println!("Content length: {} characters", content_buffer.len());
    println!("Reasoning length: {} characters", reasoning_buffer.len());

    Ok(())
}
