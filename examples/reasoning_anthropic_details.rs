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
        .build()?;

    println!("=== Example: Anthropic Reasoning Details with Tool Use ===");

    // First API call with tools and reasoning
    let chat_request = ChatCompletionRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .messages(vec![Message::new(
            Role::User,
            "What's the weather like in Boston? Then recommend what to wear.",
        )])
        .max_tokens(2000)
        .reasoning_max_tokens(1000)
        .build()?;

    let response = client.send_chat_completion(&chat_request).await?;
    let choice = &response.choices[0];

    println!("Content: {}", choice.content().unwrap_or(""));
    println!("Reasoning: {}", choice.reasoning().unwrap_or(""));

    // Extract reasoning details for preserving reasoning blocks
    if let Some(reasoning_details) = choice.reasoning_details() {
        println!(
            "Reasoning Details found: {} blocks",
            reasoning_details.len()
        );

        for (i, detail) in reasoning_details.iter().enumerate() {
            println!(
                "Block {}: Type={}, Content={}",
                i + 1,
                detail.reasoning_type(),
                if detail.content().len() > 100 {
                    &detail.content()[..100]
                } else {
                    detail.content()
                }
            );
        }

        // Example of preserving reasoning blocks for tool continuation
        // (This would typically be done when tool calls are involved)
        println!("\n=== Preserving Reasoning Blocks Example ===");
        println!("When using tools, you would preserve reasoning_details like this:");
        println!("reasoning_details: {reasoning_details:?}");

        // In a real scenario, you would pass these back in subsequent API calls
        // to maintain the model's reasoning continuity during tool use
    } else {
        println!("No reasoning details found (may not be supported by this model)");
    }

    // Example of how reasoning details would be used in a tool calling scenario
    println!("\n=== Tool Use Scenario (Conceptual) ===");
    println!("1. Model generates response with reasoning_details");
    println!("2. Model requests tool call");
    println!("3. Tool results are obtained");
    println!("4. Next request includes preserved reasoning_details");
    println!("5. Model continues reasoning from where it left off");

    Ok(())
}
