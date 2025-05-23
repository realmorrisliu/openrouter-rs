# OpenRouter Rust SDK

`openrouter-rs` is a third-party Rust SDK that helps you interact with the OpenRouter API. It wraps various endpoints of the OpenRouter API, making it easier to use in Rust projects. By taking advantage of Rust's strengths like type safety, memory safety, and concurrency without data races, `openrouter-rs` ensures a solid and reliable integration with the OpenRouter API.

## Release Notes

### Version 0.4.4

- Added: Support for listing models by supported parameters (Please note: The OpenRouter API currently does not support filtering by both `category` and `supported_parameters` at the same time. Additionally, you can only pass in a single `category` or `supported_parameters` value—not an array.)

### Version 0.4.3

- Added: Support for listing models by category (Thanks OpenRouter team! [Details](https://github.com/zed-industries/zed/discussions/16576#discussioncomment-12952507))

## Current Status

This SDK is currently in active development and supports both simple and advanced usage patterns. I've implemented basic integration tests covering:
- Get API key information
- Model listing
- Chat completions
- Response validation

If you encounter any issues while using it, please open an issue to help us improve.

Notice: I'm trying to simplify the codebase and remove some unnecessary features in version 0.5.0, including:
- Remove commands and keyring
- Simplify configuration loading
- Remove unnecessary dependencies

## TODO

- [ ] Testing
  - [x] Core integration tests
  - [ ] Complete API coverage
- [ ] Features
  - [ ] Advanced model management capabilities
  - [ ] Use cargo feature flags to enable/disable features
  - [ ] Add cli tools for easy usage

If you have any suggestions or feedback, feel free to open an issue or submit a pull request.

## Features

- ✅ Builder pattern for client configuration and complex requests
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
openrouter-rs = "0.4.4"
```

## Quick Start

### Using Client Builder Pattern (Recommended)

```rust
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Using the builder pattern for client configuration
    let client = OpenRouterClient::builder()
        .api_key("your_api_key")
        .base_url("https://openrouter.ai/api/v1") // optional
        .http_referer("your_referer") // optional
        .x_title("your_app") // optional
        .build()?;

    // Builder pattern for requests
    let request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3-0324:free")
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

### Simple Client Constructor

```rust
// Simple client creation with just API key
let client = OpenRouterClient::builder()
    .api_key("your_api_key")
    .build()?;
```

## Key Features Explained

### 1. Builder Pattern for Client Configuration

The client now uses a builder pattern for more flexible configuration:

```rust
let client = OpenRouterClient::builder()
    .api_key("your_api_key")
    .http_referer("https://yourdomain.com")
    .x_title("Your App Name")
    .build()?;
```

**Benefits:**
- Immutable client configuration
- Clear, chainable configuration
- Default values for optional parameters

### 2. Streaming Responses

```rust
use futures_util::StreamExt;

let client = OpenRouterClient::builder()
    .api_key("your_api_key")
    .build()?;
let request = ChatCompletionRequest::builder()
    .model("deepseek/deepseek-chat-v3-0324:free")
    .messages(vec![Message::new(Role::User, "Tell me a joke.")])
    .build()?;
let response = client.stream_chat_completion(&request).await?;
response
    .filter_map(|event| async { event.ok() })
    .for_each(|event| async move {
        event.choices.into_iter().for_each(|choice| {
            if let Choice::Streaming(c) = choice {
                if let Some(content) = c.delta.content {
                    println!("{}", content);
                }
            }
        });
    })
    .await;
```

### 3. Error Handling

Comprehensive error types:

```rust
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
cargo run --example send_chat_completion

# Simple usage
cargo run --example send_completion_request

# Streaming
cargo run --example stream_chat_completion

# Run integration tests (requires API key)
OPENROUTER_API_KEY=your_key cargo test --test integration -- --nocapture
```

## Best Practices

1. **Client Configuration**: Use `OpenRouterClient::builder()` for flexible setup
2. **Request Building**: Use builders (`Request::builder()`) for complex requests
3. **Error Handling**: Match on `OpenRouterError` variants
4. **Reuse Clients**: Create one `OpenRouterClient` per application

## Migration Guide

If upgrading from older versions:

| Old Style | New Recommended Style |
|-----------|-----------------------|
| `OpenRouterClient::new(key)` | `OpenRouterClient::builder().api_key(key).build()` |
| `client.base_url("url")` | `OpenRouterClient::builder().api_key(key).base_url("url").build()` |
| `ChatCompletionRequest::new()` | `ChatCompletionRequest::builder().build()` |

## Risk Disclaimer

This is a third-party SDK not affiliated with OpenRouter. Use at your own risk.

## Contributing

Contributions welcome! Please:
1. Follow the builder pattern for new features
2. Include documentation examples
3. Add tests for new features

## License

MIT - See [LICENSE](LICENSE)
