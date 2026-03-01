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
//! - **`Api`**: Normalized API failures from the OpenRouter API
//!
//! ### OpenRouter API Errors
//! - **`ApiErrorContext`**: status/code/message/request-id/metadata envelope
//! - **`ApiErrorKind::Moderation`**: Content moderation violations
//! - **`ApiErrorKind::Provider`**: Provider-specific upstream failures
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
//! match client.models().list().await {
//!     Ok(models) => println!("Found {} models", models.len()),
//!     Err(OpenRouterError::Api(api_error)) => {
//!         eprintln!("API error {}: {}", api_error.status, api_error.message);
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
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! match client.chat().create(&request).await {
//!     Err(OpenRouterError::Api(api_error)) if api_error.is_retryable() => {
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

/// Normalized API error category.
#[derive(Debug, Clone)]
pub enum ApiErrorKind {
    /// Generic API error payload without specialized metadata.
    Generic,
    /// Moderation rejection with structured moderation metadata.
    Moderation {
        reasons: Vec<String>,
        flagged_input: String,
        provider_name: String,
        model_slug: String,
    },
    /// Provider-side failure metadata.
    Provider { provider_name: String, raw: Value },
}

/// Normalized API error payload used across all endpoint modules.
#[derive(Debug, Clone)]
pub struct ApiErrorContext {
    pub status: StatusCode,
    pub api_code: Option<i64>,
    pub message: String,
    pub request_id: Option<String>,
    pub metadata: Option<Value>,
    pub kind: ApiErrorKind,
}

impl ApiErrorContext {
    /// Returns true if the request is typically retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.status,
            StatusCode::RequestTimeout
                | StatusCode::TooManyRequests
                | StatusCode::InternalServerError
                | StatusCode::BadGateway
                | StatusCode::ServiceUnavailable
                | StatusCode::GatewayTimeout
        )
    }

    pub fn is_client_error(&self) -> bool {
        self.status.is_client_error()
    }

    pub fn is_server_error(&self) -> bool {
        self.status.is_server_error()
    }
}

impl std::fmt::Display for ApiErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(api_code) = self.api_code {
            write!(
                f,
                "API error {} (api_code={}): {}",
                self.status, api_code, self.message
            )?;
        } else {
            write!(f, "API error {}: {}", self.status, self.message)?;
        }

        if let Some(request_id) = &self.request_id {
            write!(f, " [request_id={}]", request_id)?;
        }

        Ok(())
    }
}

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
/// use openrouter_rs::error::{ApiErrorKind, OpenRouterError};
///
/// // Pattern matching on specific error types
/// fn handle_error(error: OpenRouterError) {
///     match error {
///         OpenRouterError::Api(api_error) if api_error.status == surf::StatusCode::Unauthorized => {
///             println!("Check your API key");
///         }
///         OpenRouterError::Api(api_error) => match &api_error.kind {
///             ApiErrorKind::Moderation { reasons, .. } => {
///                 println!("Content flagged for: {:?}", reasons);
///             }
///             _ => println!("Other API error: {}", api_error),
///         },
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
    #[error("{0}")]
    Api(Box<ApiErrorContext>),

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
