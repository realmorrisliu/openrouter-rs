use http::StatusCode;
use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::{
    api::errors::{parse_api_error, try_parse_api_error, unreadable_error_response},
    error::OpenRouterError,
};

fn response_request_id(response: &Response) -> Option<String> {
    response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
        .or_else(|| {
            response
                .headers()
                .get("request-id")
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned)
        })
}

pub(crate) fn response_deserialization_error(
    context: &str,
    status: StatusCode,
    error: &serde_json::Error,
) -> OpenRouterError {
    OpenRouterError::Unknown(format!(
        "Failed to deserialize {context} response (status {status}): {:?} error at line {} column {}",
        error.classify(),
        error.line(),
        error.column()
    ))
}

pub(crate) async fn parse_json_response<T: DeserializeOwned>(
    response: Response,
    context: &str,
) -> Result<T, OpenRouterError> {
    let status = response.status();
    let request_id = response_request_id(&response);
    let body_text = response.text().await?;

    match serde_json::from_str(&body_text) {
        Ok(parsed) => Ok(parsed),
        Err(error) => {
            if let Some(api_error) = try_parse_api_error(status, request_id, &body_text) {
                Err(api_error)
            } else {
                Err(response_deserialization_error(context, status, &error))
            }
        }
    }
}

pub(crate) async fn error_from_response(response: Response) -> OpenRouterError {
    let status = response.status();
    let request_id = response_request_id(&response);
    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => return unreadable_error_response(status, request_id, &error.to_string()),
    };

    parse_api_error(status, request_id, &text)
}

pub(crate) async fn handle_error(response: Response) -> Result<(), OpenRouterError> {
    Err(error_from_response(response).await)
}

/// Credential-producing endpoints must never fall back to echoing an error body.
pub(crate) async fn parse_credential_response<T: DeserializeOwned>(
    response: Response,
    context: &str,
) -> Result<T, OpenRouterError> {
    if response.status().is_success() {
        return parse_json_response(response, context).await;
    }
    let status = response.status();
    let request_id = response_request_id(&response);
    let body = response.text().await?;
    Err(
        try_parse_api_error(status, request_id.clone(), &body).unwrap_or_else(|| {
            parse_api_error(
                status,
                request_id,
                "Invalid credential error response (body omitted)",
            )
        }),
    )
}
