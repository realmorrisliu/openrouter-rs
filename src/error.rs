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
//! ### Validation Errors
//! - **`ConfigError`**: Invalid SDK configuration or request/tool validation issues
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
//! ```no_run
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
//! ```no_run
//! use openrouter_rs::error::OpenRouterError;
//! use openrouter_rs::{OpenRouterClient, api::chat::{ChatCompletionRequest, Message}, types::Role};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenRouterClient::builder()
//!     .api_key("your_api_key")
//!     .build()?;
//! let request = ChatCompletionRequest::builder()
//!     .model("google/gemini-2.5-flash")
//!     .messages(vec![Message::new(Role::User, "Hello!")])
//!     .build()?;
//!
//! let mut backoff = std::time::Duration::from_secs(1);
//! for _ in 0..3 {
//!     match client.chat().create(&request).await {
//!         Err(OpenRouterError::Api(api_error)) if api_error.is_retryable() => {
//!             // `Retry-After` is absent for the HTTP-date form, so fall back to backoff.
//!             backoff = api_error.retry_after.unwrap_or(backoff * 2);
//!             tokio::time::sleep(backoff).await;
//!         }
//!         Ok(response) => break,
//!         Err(e) => return Err(e.into()),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Transport Failure Classification
//!
//! [`HttpRequestError::kind`] separates a timeout from a connection failure so a
//! caller can decide whether a retry is worthwhile without parsing the message.
//! The classification is populated by whichever backend is compiled in.
//!
//! ```rust
//! use openrouter_rs::error::{HttpRequestError, HttpRequestErrorKind};
//!
//! let error = HttpRequestError::classified(HttpRequestErrorKind::Timeout, "timed out");
//! assert!(error.is_timeout());
//! assert!(!error.is_connect());
//! ```
//!
//! ### Validation Error Handling
//!
//! ```rust
//! use openrouter_rs::error::OpenRouterError;
//!
//! fn describe(error: OpenRouterError) {
//!     match error {
//!         OpenRouterError::ConfigError(msg) => {
//!             eprintln!("Invalid SDK configuration: {}", msg);
//!         }
//!         other => eprintln!("Unexpected error: {}", other),
//!     }
//! }
//! ```
//!
//! ## 🔄 Error Conversion
//!
//! The SDK automatically converts common error types:
//!
//! - transport client errors → `OpenRouterError::HttpRequest`
//! - `serde_json::Error` → `OpenRouterError::Serialization`
//! - `std::io::Error` → `OpenRouterError::Io`
//! - `derive_builder::UninitializedFieldError` → `OpenRouterError::UninitializedFieldError`

use http::StatusCode;
use serde_json::Value;
use thiserror::Error;

/// Failure class of an HTTP request error.
///
/// The granularity a caller sees depends on the active backend: the `reqwest`
/// backend reports all variants, while `wasip3` reports the classes its host
/// API distinguishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HttpRequestErrorKind {
    /// The request exceeded the client's configured timeout.
    Timeout,
    /// The request was sent but no connection or response was established.
    Connect,
    /// The request could not be built or sent.
    Request,
    /// Reading the response body failed.
    Body,
    /// Decoding the response body failed.
    Decode,
    /// A failure the backend did not classify.
    Other,
}

/// Stable, backend-neutral request error wrapper used by [`OpenRouterError::HttpRequest`].
///
/// Holds a classification rather than a backend error, so it stays `Clone`,
/// `PartialEq`, and `Eq` whichever backend is compiled in.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct HttpRequestError {
    kind: HttpRequestErrorKind,
    url: Option<String>,
    message: String,
}

impl HttpRequestError {
    /// Builds an unclassified request error, retaining the message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            kind: HttpRequestErrorKind::Other,
            url: None,
            message: message.into(),
        }
    }

    /// Builds a classified request error.
    pub fn classified(kind: HttpRequestErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            url: None,
            message: message.into(),
        }
    }

    /// Records the request URL, when the backend captured one.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Failure class, so callers can decide whether retrying is worthwhile.
    pub fn kind(&self) -> HttpRequestErrorKind {
        self.kind
    }

    /// Request URL, when the transport captured one.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    /// True when the request exceeded the configured timeout.
    pub fn is_timeout(&self) -> bool {
        self.kind == HttpRequestErrorKind::Timeout
    }

    /// True when no response could be obtained from the server.
    pub fn is_connect(&self) -> bool {
        self.kind == HttpRequestErrorKind::Connect
    }
}

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
    /// Server-provided retry delay from the `Retry-After` header.
    ///
    /// The HTTP-date form is reported as `None` rather than guessed.
    pub retry_after: Option<std::time::Duration>,
}

impl ApiErrorContext {
    /// Returns true if the request is typically retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.status,
            StatusCode::REQUEST_TIMEOUT
                | StatusCode::TOO_MANY_REQUESTS
                | StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
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
///         OpenRouterError::Api(api_error) if api_error.status == http::StatusCode::UNAUTHORIZED => {
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
#[non_exhaustive]
pub enum OpenRouterError {
    // HTTP request errors
    #[error("HTTP request failed: {0}")]
    HttpRequest(HttpRequestError),

    // API response errors
    #[error("{0}")]
    Api(Box<ApiErrorContext>),

    // Configuration errors
    #[error("Config error: {0}")]
    ConfigError(String),

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

impl From<HttpRequestError> for OpenRouterError {
    fn from(err: HttpRequestError) -> Self {
        OpenRouterError::HttpRequest(err)
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for OpenRouterError {
    fn from(err: reqwest::Error) -> Self {
        OpenRouterError::HttpRequest(crate::transport::reqwest_error(err))
    }
}
