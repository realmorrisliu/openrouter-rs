use std::time::Duration;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::Role,
};

/// Example: inject a custom `reqwest::Client` into `OpenRouterClient`.
///
/// Useful when you need to:
///   - route through an HTTP/SOCKS proxy (geo-restricted routes, corporate egress)
///   - tighten or relax the default request timeouts
///   - attach middleware (retries, tracing, metrics)
///   - configure mTLS or custom root certificates
///
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");

    // Build a custom reqwest::Client. Replace these knobs with whatever your
    // deployment needs — proxy, middleware, retry policy, custom TLS, etc.
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .pool_max_idle_per_host(64)
        .pool_idle_timeout(Duration::from_secs(90))
        // Example: route all HTTPS traffic through a forward proxy.
        // .proxy(reqwest::Proxy::https("http://user:pass@proxy.example:6999")?)
        .build()?;

    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs-custom-http-client-example")
        .http_client(http_client)
        .build()?;

    println!("Testing custom reqwest::Client injection");
    println!("=========================================\n");

    let chat_request = ChatCompletionRequest::builder()
        .model("openai/gpt-4o-mini")
        .messages(vec![Message::new(
            Role::User,
            "Reply with the single word: ready",
        )])
        .max_tokens(10)
        .temperature(0.0)
        .build()?;

    let response = client.chat().create(&chat_request).await?;

    println!("Response ID: {}", response.id);
    println!("Model:       {}", response.model);

    if let Some(choice) = response.choices.first() {
        println!("Content:     {:?}", choice.content());
    }

    if let Some(usage) = &response.usage {
        println!("\n--- Usage ---");
        println!("Prompt tokens:     {}", usage.prompt_tokens);
        println!("Completion tokens: {}", usage.completion_tokens);
        println!("Total tokens:      {}", usage.total_tokens);
    }

    println!("\n=========================================");
    println!("Custom HTTP client example completed successfully!");

    Ok(())
}
