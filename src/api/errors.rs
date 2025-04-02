use serde::{Deserialize, Serialize};
use surf::StatusCode;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiErrorResponse {
    error: ApiError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    code: StatusCode,
    message: String,
    metadata: Option<ApiErrorMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiErrorMetadata {
    ModerationError(ModerationErrorMetadata),
    ProviderError(ProviderErrorMetadata),
    Raw(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModerationErrorMetadata {
    reasons: Vec<String>,
    flagged_input: String,
    provider_name: String,
    model_slug: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProviderErrorMetadata {
    provider_name: String,
    raw: serde_json::Value,
}

impl From<ApiErrorResponse> for crate::error::OpenRouterError {
    fn from(api_error_response: ApiErrorResponse) -> Self {
        let ApiError {
            code,
            message,
            metadata,
        } = api_error_response.error;

        match metadata {
            Some(ApiErrorMetadata::ModerationError(moderation_error)) => {
                crate::error::OpenRouterError::ModerationError {
                    code,
                    message,
                    reasons: moderation_error.reasons,
                    flagged_input: moderation_error.flagged_input,
                    provider_name: moderation_error.provider_name,
                    model_slug: moderation_error.model_slug,
                }
            }
            Some(ApiErrorMetadata::ProviderError(provider_error)) => {
                crate::error::OpenRouterError::ProviderError {
                    code,
                    message,
                    provider_name: provider_error.provider_name,
                    raw: provider_error.raw,
                }
            }
            Some(ApiErrorMetadata::Raw(raw_metadata)) => {
                crate::error::OpenRouterError::ApiErrorWithMetadata {
                    code,
                    message,
                    metadata: raw_metadata,
                }
            }
            None => crate::error::OpenRouterError::ApiError { code, message },
        }
    }
}
