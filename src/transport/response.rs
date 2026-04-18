#![allow(dead_code)]

use http::StatusCode;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    api::errors::{parse_api_error, unreadable_error_response},
    error::OpenRouterError,
};

fn body_preview(body_text: &str, limit: usize) -> String {
    let normalized = body_text.replace('\r', "\\r").replace('\n', "\\n");
    let mut preview = String::new();
    let mut chars = normalized.chars();

    for _ in 0..limit {
        match chars.next() {
            Some(ch) => preview.push(ch),
            None => return preview,
        }
    }

    if chars.next().is_some() {
        preview.push_str("...");
    }

    preview
}

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

fn body_contains_api_error(body_text: &str) -> bool {
    serde_json::from_str::<Value>(body_text)
        .ok()
        .and_then(|value| value.get("error").cloned())
        .is_some()
}

pub(crate) fn response_deserialization_error(
    context: &str,
    status: StatusCode,
    error: &serde_json::Error,
    body_text: &str,
) -> OpenRouterError {
    OpenRouterError::Unknown(format!(
        "Failed to deserialize {context} response (status {status}): {error}; body preview: {}",
        body_preview(body_text, 240)
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
            if body_contains_api_error(&body_text) {
                Err(parse_api_error(status, request_id, &body_text))
            } else {
                Err(response_deserialization_error(
                    context, status, &error, &body_text,
                ))
            }
        }
    }
}

pub(crate) async fn handle_error(response: Response) -> Result<(), OpenRouterError> {
    let status = response.status();
    let request_id = response_request_id(&response);
    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => {
            return Err(unreadable_error_response(
                status,
                request_id,
                &error.to_string(),
            ));
        }
    };

    Err(parse_api_error(status, request_id, &text))
}
