//! # OpenRouter API Endpoints
//!
//! This module contains the typed request/response implementations behind the
//! domain clients exposed from [`crate::client::OpenRouterClient`].
//!
//! Canonical domain mapping:
//!
//! - `client.chat()` -> [`chat`]
//! - `client.responses()` -> [`responses`]
//! - `client.messages()` -> [`messages`]
//! - `client.models()` -> [`models`], [`embeddings`], [`discovery`]
//! - `client.management()` -> [`api_keys`], [`auth`], [`credits`], [`generation`], [`guardrails`]
//! - `client.legacy()` -> [`legacy`] (feature `legacy-completions`)
//!
//! Endpoint families currently implemented here:
//!
//! - chat completions and multimodal content
//! - Responses API
//! - Anthropic-compatible Messages API
//! - model discovery, providers, user model filters, model counts, and ZDR endpoints
//! - embeddings
//! - API-key and auth-code flows
//! - credits, Coinbase charge creation, generation lookup, and activity
//! - guardrails and guardrail assignments
//! - structured API error payloads
//!
//! ## Quick Examples
//!
//! ### Chat
//! ```no_run
//! use openrouter_rs::{
//!     OpenRouterClient,
//!     api::chat::{ChatCompletionRequest, Message},
//!     types::Role,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenRouterClient::builder().api_key("your_key").build()?;
//! let request = ChatCompletionRequest::builder()
//!     .model("google/gemini-2.5-flash")
//!     .messages(vec![Message::new(Role::User, "Hello!")])
//!     .build()?;
//! let response = client.chat().create(&request).await?;
//! println!("{:?}", response.choices[0].content());
//! # Ok(())
//! # }
//! ```
//!
//! ### Responses
//! ```no_run
//! use openrouter_rs::{OpenRouterClient, api::responses::ResponsesRequest};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenRouterClient::builder().api_key("your_key").build()?;
//! let request = ResponsesRequest::builder()
//!     .model("openai/gpt-5")
//!     .input(json!([{ "role": "user", "content": "Say hello." }]))
//!     .build()?;
//! let response = client.responses().create(&request).await?;
//! println!("{:?}", response.id);
//! # Ok(())
//! # }
//! ```
//!
//! ### Discovery
//! ```no_run
//! use openrouter_rs::{OpenRouterClient, types::ModelCategory};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenRouterClient::builder().api_key("your_key").build()?;
//! let models = client.models().list_by_category(ModelCategory::Programming).await?;
//! println!("Found {} models", models.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All endpoint methods return `Result<_, OpenRouterError>`:
//!
//! ```no_run
//! use openrouter_rs::{
//!     OpenRouterClient,
//!     api::chat::{ChatCompletionRequest, Message},
//!     error::OpenRouterError,
//!     types::Role,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenRouterClient::builder().api_key("your_key").build()?;
//! let request = ChatCompletionRequest::builder()
//!     .model("google/gemini-2.5-flash")
//!     .messages(vec![Message::new(Role::User, "Hello!")])
//!     .build()?;
//!
//! match client.chat().create(&request).await {
//!     Ok(response) => println!("Success: {:?}", response),
//!     Err(OpenRouterError::Api(api_error)) if api_error.is_retryable() => {
//!         println!("Retryable API error: {}", api_error.message);
//!     }
//!     Err(err) => println!("Other error: {}", err),
//! }
//! # Ok(())
//! # }
//! ```

pub mod api_keys;
pub mod auth;
pub mod chat;
pub mod credits;
pub mod discovery;
pub mod embeddings;
pub mod errors;
pub mod generation;
pub mod guardrails;
pub mod messages;
pub mod models;
pub mod responses;

#[cfg(feature = "legacy-completions")]
pub mod legacy;
