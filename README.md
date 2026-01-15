# OpenRouter Rust SDK

<div align="center">

A **type-safe**, **async** Rust SDK for the [OpenRouter API](https://openrouter.ai/) - Access 200+ AI models with ease

[![Crates.io](https://img.shields.io/crates/v/openrouter-rs)](https://crates.io/crates/openrouter-rs)
[![Documentation](https://docs.rs/openrouter-rs/badge.svg)](https://docs.rs/openrouter-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[📚 Documentation](https://docs.rs/openrouter-rs) | [🎯 Examples](https://github.com/realmorrisliu/openrouter-rs/tree/main/examples) | [📦 Crates.io](https://crates.io/crates/openrouter-rs)

</div>

## 🌟 What makes this special?

- **🔒 Type Safety**: Leverages Rust's type system for compile-time error prevention
- **⚡ Async/Await**: Built on `tokio` for high-performance concurrent operations
- **🧠 Reasoning Tokens**: Industry-leading chain-of-thought reasoning support
- **📡 Streaming**: Real-time response streaming with `futures`
- **🏗️ Builder Pattern**: Ergonomic client and request construction
- **⚙️ Smart Presets**: Curated model groups for programming, reasoning, and free tiers
- **🎯 Complete Coverage**: All OpenRouter API endpoints supported

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
openrouter-rs = "0.4.7"
tokio = { version = "1", features = ["full"] }
```

### 30-Second Example

```rust
use openrouter_rs::{OpenRouterClient, api::chat::*, types::Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = OpenRouterClient::builder()
        .api_key("your_api_key")
        .build()?;

    // Send chat completion
    let request = ChatCompletionRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .messages(vec![
            Message::new(Role::User, "Explain Rust ownership in simple terms")
        ])
        .build()?;

    let response = client.send_chat_completion(&request).await?;
    println!("{}", response.choices[0].content().unwrap_or(""));

    Ok(())
}
```

## ✨ Key Features

### 🧠 Advanced Reasoning Support

Leverage chain-of-thought processing with reasoning tokens:

```rust
use openrouter_rs::types::Effort;

let request = ChatCompletionRequest::builder()
    .model("deepseek/deepseek-r1")
    .messages(vec![Message::new(Role::User, "What's bigger: 9.9 or 9.11?")])
    .reasoning_effort(Effort::High)    // Enable deep reasoning
    .reasoning_max_tokens(2000)        // Control reasoning depth
    .build()?;

let response = client.send_chat_completion(&request).await?;

// Access both reasoning and final answer
println!("🧠 Reasoning: {}", response.choices[0].reasoning().unwrap_or(""));
println!("💡 Answer: {}", response.choices[0].content().unwrap_or(""));
```

### 📡 Real-time Streaming

Process responses as they arrive:

```rust
use futures_util::StreamExt;

let stream = client.stream_chat_completion(&request).await?;

stream
    .filter_map(|event| async { event.ok() })
    .for_each(|chunk| async move {
        if let Some(content) = chunk.choices[0].content() {
            print!("{}", content);  // Print as it arrives
        }
    })
    .await;
```

### ⚙️ Smart Model Presets

Use curated model collections:

```rust
use openrouter_rs::config::OpenRouterConfig;

let config = OpenRouterConfig::default();

// Three built-in presets:
// • programming: Code generation and development
// • reasoning: Advanced problem-solving models  
// • free: Free-tier models for experimentation

println!("Available models: {:?}", config.get_resolved_models());
```

### 🛡️ Comprehensive Error Handling

```rust
use openrouter_rs::error::OpenRouterError;

match client.send_chat_completion(&request).await {
    Ok(response) => println!("Success!"),
    Err(OpenRouterError::ModerationError { reasons, .. }) => {
        eprintln!("Content flagged: {:?}", reasons);
    }
    Err(OpenRouterError::ApiError { code, message }) => {
        eprintln!("API error {}: {}", code, message);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## 📊 API Coverage

| Feature | Status | Module |
|---------|---------|---------|
| Chat Completions | ✅ | [`api::chat`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/chat/) |
| Text Completions | ✅ | [`api::completion`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/completion/) |
| **Reasoning Tokens** | ✅ | [`api::chat`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/chat/) |
| Streaming Responses | ✅ | [`api::chat`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/chat/) |
| Model Information | ✅ | [`api::models`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/models/) |
| API Key Management | ✅ | [`api::api_keys`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/api_keys/) |
| Credit Management | ✅ | [`api::credits`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/credits/) |
| Authentication | ✅ | [`api::auth`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/auth/) |

## 🎯 More Examples

### Filter Models by Category

```rust
use openrouter_rs::types::ModelCategory;

let models = client
    .list_models_by_category(ModelCategory::Programming)
    .await?;

println!("Found {} programming models", models.len());
```

### Advanced Client Configuration

```rust
let client = OpenRouterClient::builder()
    .api_key("your_key")
    .http_referer("https://yourapp.com")
    .x_title("My AI App")
    .base_url("https://openrouter.ai/api/v1")  // Custom endpoint
    .build()?;
```

### Streaming with Reasoning

```rust
let stream = client.stream_chat_completion(
    &ChatCompletionRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .messages(vec![Message::new(Role::User, "Solve this step by step: 2x + 5 = 13")])
        .reasoning_effort(Effort::High)
        .build()?
).await?;

let mut reasoning_buffer = String::new();
let mut content_buffer = String::new();

stream.filter_map(|event| async { event.ok() })
    .for_each(|chunk| async {
        if let Some(reasoning) = chunk.choices[0].reasoning() {
            reasoning_buffer.push_str(reasoning);
            print!("🧠");  // Show reasoning progress
        }
        if let Some(content) = chunk.choices[0].content() {
            content_buffer.push_str(content);
            print!("💬");  // Show content progress
        }
    }).await;

println!("\n🧠 Reasoning: {}", reasoning_buffer);
println!("💡 Answer: {}", content_buffer);
```

## 📚 Documentation & Resources

- **[📖 API Documentation](https://docs.rs/openrouter-rs)** - Complete API reference
- **[🎯 Examples Repository](https://github.com/realmorrisliu/openrouter-rs/tree/main/examples)** - Comprehensive usage examples
- **[🔧 Configuration Guide](https://docs.rs/openrouter-rs/latest/openrouter_rs/config/)** - Model presets and configuration
- **[⚡ OpenRouter API Docs](https://openrouter.ai/docs)** - Official OpenRouter documentation

### Run Examples Locally

```bash
# Set your API key
export OPENROUTER_API_KEY="your_key_here"

# Basic chat completion
cargo run --example send_chat_completion

# Reasoning tokens demo
cargo run --example chat_with_reasoning

# Streaming responses
cargo run --example stream_chat_completion

# Run with reasoning
cargo run --example stream_chat_with_reasoning
```

## 🤝 Community & Support

### 🐛 Found a Bug?

Please [open an issue](https://github.com/realmorrisliu/openrouter-rs/issues/new) with:
- Your Rust version (`rustc --version`)
- SDK version you're using
- Minimal code example
- Expected vs actual behavior

### 💡 Feature Requests

We love hearing your ideas! [Start a discussion](https://github.com/realmorrisliu/openrouter-rs/discussions) to:
- Suggest new features
- Share use cases
- Get help with implementation

### 🛠️ Contributing

Contributions are welcome! Please see our [contributing guidelines](CONTRIBUTING.md):

1. **Fork** the repository
2. **Create** a feature branch
3. **Add** tests for new functionality
4. **Follow** the existing code style
5. **Submit** a pull request

### ⭐ Show Your Support

If this SDK helps your project, consider:
- ⭐ **Starring** the repository
- 🐦 **Sharing** on social media
- 📝 **Writing** about your experience
- 🤝 **Contributing** improvements

## 📋 Requirements

- **Rust**: 1.85+ (2024 edition)
- **Tokio**: 1.0+ (for async runtime)
- **OpenRouter API Key**: [Get yours here](https://openrouter.ai/keys)

## 🗺️ Roadmap

- [ ] **WebSocket Support** - Real-time bidirectional communication
- [ ] **Retry Strategies** - Automatic retry with exponential backoff
- [ ] **Caching Layer** - Response caching for improved performance
- [ ] **CLI Tool** - Command-line interface for quick testing
- [ ] **Middleware System** - Request/response interceptors

## 📜 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This is a **third-party SDK** not officially affiliated with OpenRouter. Use at your own discretion.

---

## 📈 Release History

### Version 0.4.7 *(Latest)*

- ✨ **Added**: Gemini 3 model support

### Version 0.4.6

- 🐛 **Fixed**: Grok model deserialization error (Issue #6)
- ➕ **Added**: `index` and `logprobs` fields to Choice structs
- 🧪 **Added**: Grok model integration test and unit tests for response parsing

- 🧠 **New**: Complete reasoning tokens implementation with chain-of-thought support
- ⚙️ **Updated**: Model presets restructured to `programming`/`reasoning`/`free` categories
- 📚 **Enhanced**: Professional-grade documentation with comprehensive examples
- 🏗️ **Improved**: Configuration system with better model management

### Version 0.4.5

- Added: Support for listing models by supported parameters
- Note: OpenRouter API limitations on simultaneous category and parameter filtering

### Version 0.4.4

- Added: Support for listing models by category
- Thanks to OpenRouter team for the API enhancement!

---

<div align="center">

**Made with ❤️ for the Rust community**

[⭐ Star us on GitHub](https://github.com/realmorrisliu/openrouter-rs) | [📦 Find us on Crates.io](https://crates.io/crates/openrouter-rs) | [📚 Read the Docs](https://docs.rs/openrouter-rs)

</div>
