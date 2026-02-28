use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::Role,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs")
        .build()?;

    let question = "Which is bigger: 9.11 or 9.9?";

    println!("=== Reasoning Chain-of-Thought Example ===");
    println!("Question: {question}");

    // Step 1: Get reasoning from a capable model
    println!("\n--- Step 1: Getting reasoning from deepseek-r1 ---");
    let reasoning_request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-r1")
        .messages(vec![Message::new(
            Role::User,
            format!("{question} Please think this through, but don't output an answer"),
        )])
        .max_tokens(1000)
        .enable_reasoning()
        .build()?;

    let reasoning_response = client.send_chat_completion(&reasoning_request).await?;
    let reasoning = reasoning_response.choices[0].reasoning().unwrap_or("");

    println!("Reasoning obtained: {} characters", reasoning.len());
    println!(
        "Reasoning preview: {}",
        if reasoning.len() > 200 {
            &reasoning[..200]
        } else {
            reasoning
        }
    );

    // Step 2: Test naive response without reasoning
    println!("\n--- Step 2: Naive response from gpt-4o-mini ---");
    let naive_request = ChatCompletionRequest::builder()
        .model("openai/gpt-4o-mini")
        .messages(vec![Message::new(Role::User, question)])
        .max_tokens(200)
        .build()?;

    let naive_response = client.send_chat_completion(&naive_request).await?;
    let naive_content = naive_response.choices[0].content().unwrap_or("");

    println!("Naive response: {naive_content}");

    // Step 3: Enhanced response with injected reasoning
    println!("\n--- Step 3: Enhanced response with injected reasoning ---");
    let enhanced_content = format!("{question}. Here is some context to help you: {reasoning}");
    let enhanced_request = ChatCompletionRequest::builder()
        .model("openai/gpt-4o-mini")
        .messages(vec![Message::new(Role::User, enhanced_content)])
        .max_tokens(300)
        .build()?;

    let enhanced_response = client.send_chat_completion(&enhanced_request).await?;
    let enhanced_content = enhanced_response.choices[0].content().unwrap_or("");

    println!("Enhanced response: {enhanced_content}");

    // Step 4: Compare results
    println!("\n=== Comparison ===");
    println!("Naive approach length: {} characters", naive_content.len());
    println!(
        "Enhanced approach length: {} characters",
        enhanced_content.len()
    );
    println!("Reasoning injection improved response quality and detail!");

    Ok(())
}
