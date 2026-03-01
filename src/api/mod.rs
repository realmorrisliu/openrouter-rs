//! # OpenRouter API Endpoints
//!
//! This module provides implementations for all OpenRouter API endpoints,
//! organized by functionality. Each submodule contains the request/response
//! types and methods for interacting with specific API endpoints.
//!
//! ## 📡 Available Endpoints
//!
//! ### Chat Completions ([`chat`])
//! Modern chat-based API for conversational AI interactions:
//! - Standard chat completions
//! - Streaming responses
//! - Reasoning tokens support
//! - System/user/assistant message handling
//!
//! ```rust
//! use openrouter_rs::api::chat::{ChatCompletionRequest, Message};
//! use openrouter_rs::types::Role;
//!
//! let request = ChatCompletionRequest::builder()
//!     .model("anthropic/claude-sonnet-4")
//!     .messages(vec![Message::new(Role::User, "Hello, world!")])
//!     .build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Text Completions ([`completion`])
//! Legacy text completion API for prompt-based interactions:
//! - Simple text-in, text-out interface
//! - Backward compatibility with older applications
//!
//! ### Model Information ([`models`])
//! Retrieve information about available models:
//! - List all available models
//! - Filter by category (programming, reasoning, etc.)
//! - Filter by supported parameters
//! - Get detailed model specifications
//!
//! ```rust
//! use openrouter_rs::types::ModelCategory;
//!
//! // Get all models in the programming category
//! let models = client.models().list_by_category(ModelCategory::Programming).await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### API Key Management ([`api_keys`])
//! Manage and validate API keys:
//! - Get current API key information
//! - List all API keys for account
//! - Validate key permissions
//!
//! ### Credit Management ([`credits`])
//! Monitor usage and billing:
//! - Check current credit balance
//! - View usage statistics
//! - Track spending by model
//!
//! ### Generation Data ([`generation`])
//! Access detailed generation metadata:
//! - Token counts and pricing
//! - Model performance metrics
//! - Request/response timestamps
//!
//! ### Authentication ([`auth`])
//! Handle authentication and authorization:
//! - OAuth2 flows
//! - API key validation
//! - Permission management
//!
//! ### Error Handling ([`errors`])
//! Structured error responses from the API:
//! - Rate limiting errors
//! - Authentication failures
//! - Model availability issues
//!
//! ## 🚀 Quick Examples
//!
//! ### Basic Chat
//! ```rust
//! use openrouter_rs::{OpenRouterClient, api::chat::*};
//! use openrouter_rs::types::Role;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenRouterClient::builder()
//!     .api_key("your_key")
//!     .build()?;
//!
//! let request = ChatCompletionRequest::builder()
//!     .model("google/gemini-2.5-flash")
//!     .messages(vec![Message::new(Role::User, "Hello!")])
//!     .build()?;
//!
//! let response = client.chat().create(&request).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Model Discovery
//! ```rust
//! use openrouter_rs::OpenRouterClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenRouterClient::builder()
//!     .api_key("your_key")
//!     .build()?;
//!
//! let models = client.models().list().await?;
//! println!("Found {} models", models.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## ⚠️ Error Handling
//!
//! All API methods return `Result` types that should be handled appropriately:
//!
//! ```rust
//! use openrouter_rs::error::OpenRouterError;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! match client.chat().create(&request).await {
//!     Ok(response) => println!("Success: {:?}", response),
//!     Err(OpenRouterError::RateLimitExceeded) => {
//!         println!("Rate limit hit, retrying later...");
//!     }
//!     Err(OpenRouterError::InvalidApiKey) => {
//!         println!("Check your API key configuration");
//!     }
//!     Err(e) => println!("Other error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```

pub mod api_keys;
pub mod auth;
pub mod chat;
pub mod completion;
pub mod credits;
pub mod discovery;
pub mod embeddings;
pub mod errors;
pub mod generation;
pub mod guardrails;
pub mod messages;
pub mod models;
pub mod responses;
