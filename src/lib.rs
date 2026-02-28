//! # OpenRouter Rust SDK
//!
//! `openrouter-rs` is a type-safe, async Rust SDK for the [OpenRouter API](https://openrouter.ai/),
//! providing easy access to 200+ AI models from providers like OpenAI, Anthropic, Google, and more.
//!
//! ## ✨ Key Features
//!
//! - **🔒 Type Safety**: Leverages Rust's type system for compile-time error prevention
//! - **⚡ Async/Await**: Built on `tokio` for high-performance async operations  
//! - **🏗️ Builder Pattern**: Ergonomic client and request construction
//! - **📡 Streaming Support**: Real-time response streaming with `futures`
//! - **🧠 Reasoning Tokens**: Advanced support for chain-of-thought reasoning
//! - **⚙️ Model Presets**: Pre-configured model groups for different use cases
//! - **🎯 Full API Coverage**: Complete OpenRouter API endpoint support
//!
//! ## 🚀 Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! openrouter-rs = "0.5.1"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! ### Basic Chat Completion
//!
//! ```rust
//! use openrouter_rs::{
//!     OpenRouterClient,
//!     api::chat::{ChatCompletionRequest, Message},
//!     types::Role,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client with builder pattern
//!     let client = OpenRouterClient::builder()
//!         .api_key("your_api_key")
//!         .http_referer("https://yourapp.com")
//!         .x_title("My App")
//!         .build()?;
//!
//!     // Build chat request
//!     let request = ChatCompletionRequest::builder()
//!         .model("anthropic/claude-sonnet-4")
//!         .messages(vec![
//!             Message::new(Role::System, "You are a helpful assistant"),
//!             Message::new(Role::User, "Explain Rust ownership in simple terms"),
//!         ])
//!         .temperature(0.7)
//!         .max_tokens(500)
//!         .build()?;
//!
//!     // Send request and get response
//!     let response = client.send_chat_completion(&request).await?;
//!     println!("Response: {}", response.choices[0].content().unwrap_or(""));
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Streaming Responses
//!
//! ```rust
//! use futures_util::StreamExt;
//! use openrouter_rs::{OpenRouterClient, api::chat::*};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenRouterClient::builder()
//!     .api_key("your_api_key")
//!     .build()?;
//!
//! let request = ChatCompletionRequest::builder()
//!     .model("google/gemini-2.5-flash")
//!     .messages(vec![Message::new(Role::User, "Write a haiku about Rust")])
//!     .build()?;
//!
//! let mut stream = client.stream_chat_completion(&request).await?;
//!
//! while let Some(result) = stream.next().await {
//!     if let Ok(response) = result {
//!         if let Some(content) = response.choices[0].content() {
//!             print!("{}", content);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Reasoning Tokens (Chain-of-Thought)
//!
//! ```rust
//! use openrouter_rs::{OpenRouterClient, api::chat::*, types::{Role, Effort}};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenRouterClient::builder()
//!     .api_key("your_api_key")
//!     .build()?;
//!
//! let request = ChatCompletionRequest::builder()
//!     .model("deepseek/deepseek-r1")
//!     .messages(vec![Message::new(Role::User, "What's bigger: 9.9 or 9.11?")])
//!     .reasoning_effort(Effort::High)  // Enable high-effort reasoning
//!     .reasoning_max_tokens(1000)      // Limit reasoning tokens
//!     .build()?;
//!
//! let response = client.send_chat_completion(&request).await?;
//!
//! println!("Reasoning: {}", response.choices[0].reasoning().unwrap_or(""));
//! println!("Answer: {}", response.choices[0].content().unwrap_or(""));
//! # Ok(())
//! # }
//! ```
//!
//! ## 📚 Core Modules
//!
//! - [`client`] - Client configuration and HTTP operations
//! - [`api`] - OpenRouter API endpoints (chat, models, credits, etc.)
//! - [`types`] - Request/response types and enums
//! - [`config`] - Configuration management and model presets
//! - [`error`] - Error types and handling
//!
//! ## 🎯 Model Presets
//!
//! The SDK includes curated model presets for different use cases:
//!
//! - **`programming`**: Code generation and software development
//! - **`reasoning`**: Advanced reasoning and problem-solving  
//! - **`free`**: Free-tier models for experimentation
//!
//! ```rust
//! use openrouter_rs::config::OpenRouterConfig;
//!
//! let config = OpenRouterConfig::default();
//! println!("Available models: {:?}", config.get_resolved_models());
//! ```
//!
//! ## 🔗 API Coverage
//!
//! | Feature | Status | Module |
//! |---------|--------|---------|
//! | Chat Completions | ✅ | [`api::chat`] |
//! | Text Completions | ✅ | [`api::completion`] |
//! | Model Information | ✅ | [`api::models`] |
//! | Streaming | ✅ | [`api::chat`] |
//! | Reasoning Tokens | ✅ | [`api::chat`] |
//! | API Key Management | ✅ | [`api::api_keys`] |
//! | Credit Management | ✅ | [`api::credits`] |
//! | Generation Data | ✅ | [`api::generation`] |
//! | Authentication | ✅ | [`api::auth`] |
//! | Guardrails | ✅ | [`api::guardrails`] |
//!
//! ## 📖 Examples
//!
//! Check out the [`examples/`](https://github.com/realmorrisliu/openrouter-rs/tree/main/examples)
//! directory for comprehensive usage examples:
//!
//! - Basic chat completion
//! - Streaming responses  
//! - Reasoning tokens
//! - Model management
//! - Error handling
//! - Advanced configurations
//!
//! ## 🤝 Contributing
//!
//! Contributions are welcome! Please see our
//! [GitHub repository](https://github.com/realmorrisliu/openrouter-rs) for issues and pull requests.

pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod types;
pub mod utils;

pub use api::chat::{Content, ContentPart, ImageUrl, Message};
pub use api::models::Model;
pub use client::OpenRouterClient;
