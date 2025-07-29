//! # Error Handling
//!
//! This module provides comprehensive error types for the OpenRouter SDK.
//! All errors implement the standard library's `Error` trait and can be
//! used with any error handling framework.
//!
//! ## 🎯 Error Categories
//!
//! ### HTTP Request Errors
//! - **`HttpRequest`**: Network-level failures (connectivity, timeouts)
//! - **`ApiError`**: HTTP status code errors from the API
//!
//! ### OpenRouter API Errors  
//! - **`ModerationError`**: Content moderation violations
//! - **`ProviderError`**: Issues with specific model providers
//! - **`ApiErrorWithMetadata`**: API errors with additional context
//!
//! ### Configuration Errors
//! - **`ConfigError`**: Configuration parsing or validation issues
//! - **`ConfigNotFound`**: Missing configuration files
//! - **`KeyNotConfigured`**: Missing or invalid API keys
//!
//! ### Data Processing Errors
//! - **`UninitializedFieldError`**: Builder pattern validation failures
//! - **`Serialization`**: JSON serialization/deserialization errors
//!
//! ### System Errors
//! - **`Io`**: File system and I/O operations
//! - **`Unknown`**: Unexpected errors
//!
//! ## 🚀 Usage Examples
//!
//! ### Basic Error Handling
//!
//! ```rust
//! use openrouter_rs::{OpenRouterClient, error::OpenRouterError};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenRouterClient::builder()
//!     .api_key("invalid_key")
//!     .build()?;
//!
//! match client.list_models().await {
//!     Ok(models) => println!("Found {} models", models.len()),
//!     Err(OpenRouterError::ApiError { code, message }) => {
//!         eprintln!("API error {}: {}", code, message);
//!     }
//!     Err(OpenRouterError::HttpRequest(e)) => {
//!         eprintln!("Network error: {}", e);
//!     }
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Rate Limiting Handling
//!
//! ```rust
//! use openrouter_rs::error::OpenRouterError;
//! use surf::StatusCode;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! match client.send_chat_completion(&request).await {
//!     Err(OpenRouterError::ApiError { code: StatusCode::TooManyRequests, .. }) => {
//!         println!("Rate limited, retrying after delay...");
//!         tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
//!         // Retry logic here
//!     }
//!     Ok(response) => println!("Success!"),
//!     Err(e) => return Err(e.into()),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Configuration Error Handling
//!
//! ```rust
//! use openrouter_rs::{config::load_config, error::OpenRouterError};
//!
//! match load_config("./config.toml") {
//!     Ok(config) => println!("Config loaded successfully"),
//!     Err(OpenRouterError::ConfigNotFound(path)) => {
//!         println!("Config file not found at: {}", path.display());
//!         // Use default configuration
//!     }
//!     Err(OpenRouterError::ConfigError(msg)) => {
//!         eprintln!("Invalid configuration: {}", msg);
//!     }
//!     Err(e) => eprintln!("Unexpected error: {}", e),
//! }
//! ```
//!
//! ## 🔄 Error Conversion
//!
//! The SDK automatically converts common error types:
//!
//! - `surf::Error` → `OpenRouterError::HttpRequest`
//! - `serde_json::Error` → `OpenRouterError::Serialization`
//! - `std::io::Error` → `OpenRouterError::Io`
//! - `derive_builder::UninitializedFieldError` → `OpenRouterError::UninitializedFieldError`

use std::path::PathBuf;

use serde_json::Value;
use surf::StatusCode;
use thiserror::Error;

/// Comprehensive error type for OpenRouter SDK operations
///
/// This enum covers all possible error conditions that can occur when
/// using the OpenRouter SDK, from network issues to API-specific errors.
/// All variants implement `std::error::Error` and provide detailed
/// context information.
///
/// # Examples
///
/// ```rust
/// use openrouter_rs::error::OpenRouterError;
/// use surf::StatusCode;
///
/// // Pattern matching on specific error types
/// fn handle_error(error: OpenRouterError) {
///     match error {
///         OpenRouterError::ApiError { code: StatusCode::Unauthorized, .. } => {
///             println!("Check your API key");
///         }
///         OpenRouterError::ModerationError { reasons, .. } => {
///             println!("Content flagged for: {:?}", reasons);
///         }
///         OpenRouterError::ConfigError(msg) => {
///             println!("Configuration issue: {}", msg);
///         }
///         _ => println!("Other error: {}", error),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum OpenRouterError {
    // HTTP request errors
    #[error("HTTP request failed: {0}")]
    HttpRequest(surf::Error),

    // API response errors
    #[error("API error: {code}, message: {message}")]
    ApiError { code: StatusCode, message: String },

    #[error(
        "Moderation error: {message}, reasons: {reasons:?}, flagged input: {flagged_input}, provider: {provider_name}, model: {model_slug}"
    )]
    ModerationError {
        code: StatusCode,
        message: String,
        reasons: Vec<String>,
        flagged_input: String,
        provider_name: String,
        model_slug: String,
    },

    #[error("Provider error: {message}, provider: {provider_name}, raw: {raw:?}")]
    ProviderError {
        code: StatusCode,
        message: String,
        provider_name: String,
        raw: Value,
    },

    #[error("API error with metadata: {message}, metadata: {metadata:?}")]
    ApiErrorWithMetadata {
        code: StatusCode,
        message: String,
        metadata: Value,
    },

    // Configuration errors
    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Config file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("API key not configured")]
    KeyNotConfigured,

    // Data processing errors
    #[error("Uninitialized field error: {0}")]
    UninitializedFieldError(#[from] derive_builder::UninitializedFieldError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    // System IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // Uncategorized errors
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<surf::Error> for OpenRouterError {
    fn from(err: surf::Error) -> Self {
        OpenRouterError::HttpRequest(err)
    }
}
