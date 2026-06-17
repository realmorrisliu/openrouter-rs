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
//! - `client.rerank()` -> [`rerank`]
//! - `client.audio().speech()` / `client.audio().transcriptions()` -> [`audio`]
//! - `client.videos()` -> [`videos`]
//! - `client.models()` -> [`models`], [`embeddings`], [`discovery`]
//! - `client.management()` -> [`api_keys`], [`auth`], [`byok`], [`credits`], [`generation`], [`guardrails`], [`observability`], [`organization`], [`presets`], [`workspaces`]
//! - `client.legacy()` -> [`legacy`] (feature `legacy-completions`)
//!
//! Endpoint families currently implemented here:
//!
//! - chat completions and multimodal content
//! - Responses API
//! - Anthropic-compatible Messages API
//! - rerank
//! - audio speech generation and transcription
//! - video generation and polling
//! - model discovery, providers, user model filters, model counts, rankings, and ZDR endpoints
//! - embeddings
//! - API-key and auth-code flows
//! - credits, Coinbase charge creation, generation lookup/content, and activity
//! - BYOK provider credential management
//! - observability destination management
//! - preset creation from inference request bodies
//! - guardrails and guardrail assignments
//! - organization member listing
//! - workspace CRUD and membership management
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

pub mod analytics;
pub mod api_keys;
pub mod audio;
pub mod auth;
pub mod byok;
pub mod chat;
pub mod credits;
pub mod discovery;
pub mod embeddings;
pub mod errors;
pub mod files;
pub mod generation;
pub mod guardrails;
pub mod messages;
pub mod models;
pub mod observability;
pub mod organization;
pub mod presets;
pub mod rerank;
pub mod responses;
#[deprecated(note = "use api::audio for the canonical /audio/speech surface")]
pub mod tts;
pub mod videos;
pub mod workspaces;

#[cfg(feature = "legacy-completions")]
pub mod legacy;
