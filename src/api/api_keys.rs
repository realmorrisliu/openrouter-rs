use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;

use crate::{error::OpenRouterError, types::ApiResponse, utils::handle_error};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiKey {
    pub name: Option<String>,
    pub label: Option<String>,
    pub limit: Option<f64>,
    pub disabled: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub hash: Option<String>,
    pub key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiKeyDetails {
    pub label: String,
    pub usage: f64,
    pub is_free_tier: bool,
    pub is_provisioning_key: bool,
    pub rate_limit: RateLimit,
    pub limit: Option<f64>,
    pub limit_remaining: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RateLimit {
    pub requests: f64,
    pub interval: String,
}

#[derive(Serialize)]
struct CreateApiKeyRequest {
    name: String,
    limit: Option<f64>,
}

#[derive(Serialize)]
struct UpdateApiKeyRequest {
    name: Option<String>,
    disabled: Option<bool>,
    limit: Option<f64>,
}

/// Get information on the API key associated with the current authentication session
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
///
/// # Returns
///
/// * `Result<ApiKeyDetails, OpenRouterError>` - The details of the current API key.
pub async fn get_current_api_key(
    base_url: &str,
    api_key: &str,
) -> Result<ApiKeyDetails, OpenRouterError> {
    let url = format!("{base_url}/key");

    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .send()
        .await?;

    if response.status().is_success() {
        let api_response: ApiResponse<_> = response.body_json().await?;
        Ok(api_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Returns a list of all API keys associated with the account. Requires a Provisioning API key.
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `offset` - Optional offset for the API keys.
/// * `include_disabled` - Optional flag to include disabled API keys.
///
/// # Returns
///
/// * `Result<Vec<ApiKey>, OpenRouterError>` - A list of API keys.
pub async fn list_api_keys(
    base_url: &str,
    api_key: &str,
    offset: Option<f64>,
    include_disabled: Option<bool>,
) -> Result<Vec<ApiKey>, OpenRouterError> {
    let url = format!("{base_url}/keys");
    let mut query_params = HashMap::new();
    if let Some(offset) = offset {
        query_params.insert("offset", offset.to_string());
    }
    if let Some(include_disabled) = include_disabled {
        query_params.insert("include_disabled", include_disabled.to_string());
    }

    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .query(&query_params)?
        .await?;

    if response.status().is_success() {
        let api_response: ApiResponse<_> = response.body_json().await?;
        Ok(api_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Creates a new API key. Requires a Provisioning API key.
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `name` - The display name for the new API key.
/// * `limit` - Optional credit limit for the new API key.
///
/// # Returns
///
/// * `Result<ApiKey, OpenRouterError>` - The created API key.
pub async fn create_api_key(
    base_url: &str,
    api_key: &str,
    name: &str,
    limit: Option<f64>,
) -> Result<ApiKey, OpenRouterError> {
    let url = format!("{base_url}/keys");
    let request = CreateApiKeyRequest {
        name: name.to_string(),
        limit,
    };

    let mut response = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .body_json(&request)?
        .await?;

    if response.status().is_success() {
        let api_response: ApiResponse<_> = response.body_json().await?;
        Ok(api_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Returns details about a specific API key. Requires a Provisioning API key.
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `hash` - The hash of the API key to retrieve.
///
/// # Returns
///
/// * `Result<ApiKey, OpenRouterError>` - The details of the specified API key.
pub async fn get_api_key(
    base_url: &str,
    api_key: &str,
    hash: &str,
) -> Result<ApiKey, OpenRouterError> {
    let url = format!("{base_url}/keys/{hash}");

    let mut response = surf::get(&url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .await?;

    if response.status().is_success() {
        let api_response: ApiResponse<_> = response.body_json().await?;
        Ok(api_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Deletes an API key. Requires a Provisioning API key.
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `hash` - The hash of the API key to delete.
///
/// # Returns
///
/// * `Result<bool, OpenRouterError>` - A boolean indicating whether the deletion was successful.
pub async fn delete_api_key(
    base_url: &str,
    api_key: &str,
    hash: &str,
) -> Result<bool, OpenRouterError> {
    let url = format!("{base_url}/keys/{hash}");

    let response = surf::delete(&url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .await?;

    if response.status().is_success() {
        Ok(true)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Updates an existing API key. Requires a Provisioning API key.
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `hash` - The hash of the API key to update.
/// * `name` - Optional new display name for the API key.
/// * `disabled` - Optional flag to disable the API key.
/// * `limit` - Optional new credit limit for the API key.
///
/// # Returns
///
/// * `Result<ApiKey, OpenRouterError>` - The updated API key.
pub async fn update_api_key(
    base_url: &str,
    api_key: &str,
    hash: &str,
    name: Option<String>,
    disabled: Option<bool>,
    limit: Option<f64>,
) -> Result<ApiKey, OpenRouterError> {
    let url = format!("{base_url}/keys/{hash}");
    let request = UpdateApiKeyRequest {
        name,
        disabled,
        limit,
    };

    let mut response = surf::patch(&url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .body_json(&request)?
        .await?;

    if response.status().is_success() {
        let api_response: ApiResponse<_> = response.body_json().await?;
        Ok(api_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
