use std::time::Duration;

use http::StatusCode;
use serde::de::DeserializeOwned;

use super::{RequestBuilder, Response};
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

/// Parses the delta-seconds form of `Retry-After`. The HTTP-date form is
/// reported as absent rather than guessed.
fn retry_after(response: &Response) -> Option<Duration> {
    response
        .headers()
        .get(http::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

fn attach_retry_after(error: OpenRouterError, delay: Option<Duration>) -> OpenRouterError {
    match (error, delay) {
        (OpenRouterError::Api(mut context), Some(delay)) => {
            context.retry_after.get_or_insert(delay);
            OpenRouterError::Api(context)
        }
        (error, _) => error,
    }
}

pub(crate) async fn error_from_response(response: Response) -> OpenRouterError {
    let status = response.status();
    let request_id = response_request_id(&response);
    let delay = retry_after(&response);
    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => {
            return attach_retry_after(
                unreadable_error_response(status, request_id, &error.to_string()),
                delay,
            );
        }
    };

    attach_retry_after(parse_api_error(status, request_id, &text), delay)
}

/// Sends a prepared request and decodes the success body, normalizing any
/// non-success response into an [`OpenRouterError`].
pub(crate) async fn send_json<T: DeserializeOwned>(
    request: RequestBuilder,
    context: &str,
) -> Result<T, OpenRouterError> {
    parse_or_error(request.send().await?, context).await
}

/// Sends a prepared request whose success response carries no body.
pub(crate) async fn send_unit(request: RequestBuilder) -> Result<(), OpenRouterError> {
    let response = request.send().await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(error_from_response(response).await)
    }
}

/// Decodes a success body, or normalizes a failed response into an API error.
pub(crate) async fn parse_or_error<T: DeserializeOwned>(
    response: Response,
    context: &str,
) -> Result<T, OpenRouterError> {
    if response.status().is_success() {
        parse_json_response(response, context).await
    } else {
        Err(error_from_response(response).await)
    }
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
    let body = match response.text().await {
        Ok(body) => body,
        Err(error) => {
            return Err(unreadable_error_response(
                status,
                request_id,
                &error.to_string(),
            ));
        }
    };
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
