use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenRouterError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("API error: {code}, message: {message}")]
    ApiError { code: u16, message: String },
    #[error(
        "Moderation error: {message}, reasons: {reasons:?}, flagged input: {flagged_input}, provider: {provider_name}, model: {model_slug}"
    )]
    ModerationError {
        code: u16,
        message: String,
        reasons: Vec<String>,
        flagged_input: String,
        provider_name: String,
        model_slug: String,
    },
    #[error("Provider error: {message}, provider: {provider_name}, raw: {raw:?}")]
    ProviderError {
        code: u16,
        message: String,
        provider_name: String,
        raw: Value,
    },
    #[error("API error with metadata: {message}, metadata: {metadata:?}")]
    ApiErrorWithMetadata {
        code: u16,
        message: String,
        metadata: Value,
    },
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Unknown error: {0}")]
    Unknown(String),
}
