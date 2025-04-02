# OpenRouter Rust SDK

`openrouter-rs` is a third-party Rust SDK that helps you interact with the OpenRouter API. It wraps various endpoints of the OpenRouter API, making it easier to use in Rust projects. By taking advantage of Rust's strengths like type safety, memory safety, and concurrency without data races, `openrouter-rs` ensures a solid and reliable integration with the OpenRouter API.

## Current Status

This SDK is currently being used in production and supports both simple and advanced usage patterns. If you encounter any issues while using it, please open an issue.

## Features

- ✅ Builder pattern for complex requests (chat/completion/credits/generation)
- ✅ Simple constructors for basic usage
- ✅ Full API coverage including:
  - API key management
  - Chat and text completions
  - Streaming responses
  - Credit management
  - Model information

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
openrouter-rs = "0.3.0"
```

## Quick Start

### Using Builder Pattern (Recommended)

```rust
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenRouterClient::new("your_api_key");

    // Builder pattern for full control
    let request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat:free")
        .messages(vec![
            Message::new(Role::System, "You are a helpful assistant"),
            Message::new(Role::User, "Explain Rust in simple terms")
        ])
        .temperature(0.7)
        .max_tokens(200)
        .build()?;

    let response = client.send_chat_completion(&request).await?;
    println!("Response: {:?}", response);

    Ok(())
}
```

### Simple Constructor

```rust
use openrouter_rs::api::chat::{ChatCompletionRequest, Message, Role};

// Simple one-off requests
let request = ChatCompletionRequest::new(
    "deepseek/deepseek-chat:free",
    vec![Message::new(Role::User, "Hello world!")]
);
```

## Key Features Explained

### 1. Builder Pattern

For complex requests, we recommend using the builder pattern:

```rust
use openrouter_rs::api::completion::CompletionRequest;

let request = CompletionRequest::builder()
    .model("deepseek/deepseek-chat:free")
    .prompt("Write a poem about Rust")
    .temperature(0.8)
    .top_p(0.9)
    .max_tokens(150)
    .build()?;
```

**Benefits:**
- Compile-time safety
- Auto-completion friendly
- Clear parameter validation

### 2. Streaming Responses

```rust
use openrouter_rs::api::chat::{ChatCompletionRequest, Message, Role};
use futures_util::StreamExt;

let client = OpenRouterClient::new("your_api_key");
let request = ChatCompletionRequest::builder()
    .model("deepseek/deepseek-chat:free")
    .messages(vec![Message::new(Role::User, "Tell me a joke.")])
    .max_tokens(50)
    .temperature(0.5)
    .build()?;
let mut stream = client.stream_chat_completion(&request).await?;
while let Some(event) = stream.next().await {
    match event {
        Ok(event) => print!("{}", event.choices[0].delta.content.unwrap_or_default()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### 3. Error Handling

Comprehensive error types:

```rust
use openrouter_rs::error::OpenRouterError;

match client.send_chat_completion(&request).await {
    Ok(response) => { /* handle success */ },
    Err(OpenRouterError::ModerationError { reasons, .. }) => {
        eprintln!("Content flagged: {:?}", reasons);
    },
    Err(e) => { /* handle other errors */ }
}
```

## Examples

Run examples with:

```sh
# Builder pattern example
cargo run --example chat_completion

# Simple usage
cargo run --example quick_start

# Streaming
cargo run --example streaming_chat
```

## Best Practices

1. **For complex requests**: Use builders (`Request::builder()`)
2. **For simple cases**: Use direct constructors (`Request::new()`)
3. **Always handle errors**: Match on `OpenRouterError` variants
4. **Reuse clients**: Create one `OpenRouterClient` per application

## Migration Guide

If upgrading from older versions:

| Old Style | New Recommended Style |
|-----------|-----------------------|
| `ChatCompletionRequest::new().max_tokens(100)` | `ChatCompletionRequest::builder().max_tokens(100).build()` |
| `CoinbaseChargeRequest::new(1.0, "addr", 1)` | `CoinbaseChargeRequest::builder().amount(1.0).sender("addr").chain_id(1).build()` |

## Risk Disclaimer

This is a third-party SDK not affiliated with OpenRouter. Use at your own risk.

## Contributing

Contributions welcome! Please:
1. Use builders for new request types
2. Include documentation examples
3. Add tests for new features

## License

MIT - See [LICENSE](LICENSE)
