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

    println!("=== Streaming Chat with Progressive Accumulation ===");
    let chat_request = ChatCompletionRequest::builder()
        .model("openai/o3-mini")
        .messages(vec![Message::new(
            Role::User,
            "What's bigger, 9.9 or 9.11? Think about this step by step.",
        )])
        .max_tokens(1000)
        .reasoning_effort(Effort::High)
        .build()?;

    let stream = client.stream_chat_completion(&chat_request).await?;

    // Method 3: Accumulate while showing progress
    println!("Processing stream with progress indication...\n");
    let (content_buffer, reasoning_buffer, chunk_count) = stream
        .filter_map(|event| async { event.ok() })
        .enumerate()
        .fold(
            (String::new(), String::new(), 0),
            |(mut content, mut reasoning, _), (i, event)| async move {
                let mut has_data = false;

                if let Some(r) = event.choices[0].reasoning() {
                    reasoning.push_str(r);
                    print!("📝"); // Reasoning content indicator
                    has_data = true;
                }

                if let Some(c) = event.choices[0].content() {
                    content.push_str(c);
                    print!("💬"); // Content indicator
                    has_data = true;
                }

                if has_data {
                    // Show progress every 10 chunks
                    if i % 10 == 0 {
                        print!(" [chunk {}]", i + 1);
                    }

                    // Flush output
                    use std::io::{self, Write};
                    io::stdout().flush().unwrap();
                }

                (content, reasoning, i + 1)
            },
        )
        .await;

    println!("\n\n=== Final Results (Progressive Method) ===");
    println!("Processed {chunk_count} chunks");
    println!("Content length: {} characters", content_buffer.len());
    println!("Reasoning length: {} characters", reasoning_buffer.len());

    println!("\n--- Complete Content ---");
    println!("{content_buffer}");

    println!("\n--- Complete Reasoning ---");
    println!(
        "{}",
        if reasoning_buffer.len() > 500 {
            format!("{}...", &reasoning_buffer[..500])
        } else {
            reasoning_buffer
        }
    );

    Ok(())
}
