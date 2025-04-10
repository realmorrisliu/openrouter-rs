use std::path::PathBuf;

use serde_json::Value;
use surf::StatusCode;
use thiserror::Error;

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
    #[error("Validation error: {0}")]
    Validation(String),

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
