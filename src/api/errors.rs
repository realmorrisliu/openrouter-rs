use serde::Deserialize;
use serde_json::Value;
use surf::StatusCode;

use crate::error::{ApiErrorContext, ApiErrorKind, OpenRouterError};

#[derive(Deserialize, Debug)]
struct ApiErrorResponse {
    error: ApiError,
}

#[derive(Deserialize, Debug)]
struct ApiError {
    code: Option<i64>,
    message: String,
    metadata: Option<ApiErrorMetadata>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ApiErrorMetadata {
    ModerationError(ModerationErrorMetadata),
    ProviderError(ProviderErrorMetadata),
    Raw(Value),
}

#[derive(Deserialize, Debug)]
struct ModerationErrorMetadata {
    reasons: Vec<String>,
    flagged_input: String,
    provider_name: String,
    model_slug: String,
}

#[derive(Deserialize, Debug)]
struct ProviderErrorMetadata {
    provider_name: String,
    raw: Value,
}

fn normalize_error_status(status: StatusCode, api_code: Option<i64>) -> StatusCode {
    if !status.is_success() {
        return status;
    }

    api_code
        .and_then(|code| u16::try_from(code).ok())
        .and_then(|code| StatusCode::try_from(code).ok())
        .filter(|code| code.is_client_error() || code.is_server_error())
        .unwrap_or(status)
}

fn build_api_error(
    status: StatusCode,
    request_id: Option<String>,
    api_error: ApiError,
) -> OpenRouterError {
    let ApiError {
        code,
        message,
        metadata,
    } = api_error;
    let status = normalize_error_status(status, code);

    let (kind, normalized_metadata) = match metadata {
        Some(ApiErrorMetadata::ModerationError(moderation)) => (
            ApiErrorKind::Moderation {
                reasons: moderation.reasons.clone(),
                flagged_input: moderation.flagged_input.clone(),
                provider_name: moderation.provider_name.clone(),
                model_slug: moderation.model_slug.clone(),
            },
            Some(serde_json::json!({
                "reasons": moderation.reasons,
                "flagged_input": moderation.flagged_input,
                "provider_name": moderation.provider_name,
                "model_slug": moderation.model_slug,
            })),
        ),
        Some(ApiErrorMetadata::ProviderError(provider)) => (
            ApiErrorKind::Provider {
                provider_name: provider.provider_name,
                raw: provider.raw.clone(),
            },
            Some(provider.raw),
        ),
        Some(ApiErrorMetadata::Raw(raw)) => (ApiErrorKind::Generic, Some(raw)),
        None => (ApiErrorKind::Generic, None),
    };

    OpenRouterError::Api(Box::new(ApiErrorContext {
        status,
        api_code: code,
        message,
        request_id,
        metadata: normalized_metadata,
        kind,
    }))
}

pub fn parse_api_error(
    status: StatusCode,
    request_id: Option<String>,
    text: &str,
) -> OpenRouterError {
    match serde_json::from_str::<ApiErrorResponse>(text) {
        Ok(payload) => build_api_error(status, request_id, payload.error),
        Err(_) => OpenRouterError::Api(Box::new(ApiErrorContext {
            status,
            api_code: Some(i64::from(u16::from(status))),
            message: text.to_string(),
            request_id,
            metadata: None,
            kind: ApiErrorKind::Generic,
        })),
    }
}

pub fn unreadable_error_response(
    status: StatusCode,
    request_id: Option<String>,
    error: &str,
) -> OpenRouterError {
    OpenRouterError::Api(Box::new(ApiErrorContext {
        status,
        api_code: Some(i64::from(u16::from(status))),
        message: format!("Failed to read error response body: {error}"),
        request_id,
        metadata: Some(serde_json::json!({
            "body_read_error": error,
        })),
        kind: ApiErrorKind::Generic,
    }))
}
