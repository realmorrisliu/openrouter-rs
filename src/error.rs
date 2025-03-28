use serde_json::Value;
use surf::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenRouterError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(surf::Error),
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
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<surf::Error> for OpenRouterError {
    fn from(err: surf::Error) -> Self {
        OpenRouterError::HttpRequest(err)
    }
}
