use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct ApiKey {
    pub name: Option<String>,
    pub label: Option<String>,
    pub limit: Option<f64>,
    pub disabled: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub hash: Option<String>,
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

#[derive(Serialize, Debug)]
#[non_exhaustive]
pub struct ApiKeyDetails {
    pub label: String,
    pub usage: f64,
    pub is_free_tier: bool,
    pub is_management_key: bool,
    pub rate_limit: RateLimit,
    pub limit: Option<f64>,
    pub limit_remaining: Option<f64>,
}

#[derive(Deserialize)]
struct ApiKeyDetailsWire {
    label: String,
    usage: f64,
    is_free_tier: bool,
    #[serde(default)]
    is_management_key: Option<bool>,
    #[serde(default)]
    is_provisioning_key: Option<bool>,
    rate_limit: RateLimit,
    limit: Option<f64>,
    limit_remaining: Option<f64>,
}

impl<'de> Deserialize<'de> for ApiKeyDetails {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ApiKeyDetailsWire::deserialize(deserializer)?;
        Ok(Self {
            label: wire.label,
            usage: wire.usage,
            is_free_tier: wire.is_free_tier,
            is_management_key: wire
                .is_management_key
                .or(wire.is_provisioning_key)
                .unwrap_or(false),
            rate_limit: wire.rate_limit,
            limit: wire.limit,
            limit_remaining: wire.limit_remaining,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct RateLimit {
    pub requests: f64,
    pub interval: String,
}

#[derive(Serialize)]
struct CreateApiKeyRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
}

#[derive(Serialize)]
struct UpdateApiKeyRequest {
    name: Option<String>,
    disabled: Option<bool>,
    limit: Option<f64>,
}

#[derive(Serialize)]
struct ListApiKeysQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
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
    let http_client = crate::transport::new_client()?;
    get_current_api_key_with_client(&http_client, base_url, api_key).await
}

pub(crate) async fn get_current_api_key_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
) -> Result<ApiKeyDetails, OpenRouterError> {
    let url = format!("{base_url}/key");

    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let api_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "current API key").await?;
        Ok(api_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Returns a list of all API keys associated with the account. Requires a management API key.
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `management_key` - The management API key for authentication.
/// * `pagination` - Optional pagination options for the API keys list.
/// * `include_disabled` - Optional flag to include disabled API keys.
///
/// # Returns
///
/// * `Result<Vec<ApiKey>, OpenRouterError>` - A list of API keys.
pub async fn list_api_keys(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
    include_disabled: Option<bool>,
) -> Result<Vec<ApiKey>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_api_keys_in_workspace_with_client(
        &http_client,
        base_url,
        management_key,
        pagination,
        include_disabled,
        None,
    )
    .await
}

pub(crate) async fn list_api_keys_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
    include_disabled: Option<bool>,
) -> Result<Vec<ApiKey>, OpenRouterError> {
    list_api_keys_in_workspace_with_client(
        http_client,
        base_url,
        management_key,
        pagination,
        include_disabled,
        None,
    )
    .await
}

pub async fn list_api_keys_in_workspace(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
    include_disabled: Option<bool>,
    workspace_id: Option<&str>,
) -> Result<Vec<ApiKey>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_api_keys_in_workspace_with_client(
        &http_client,
        base_url,
        management_key,
        pagination,
        include_disabled,
        workspace_id,
    )
    .await
}

pub(crate) async fn list_api_keys_in_workspace_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
    include_disabled: Option<bool>,
    workspace_id: Option<&str>,
) -> Result<Vec<ApiKey>, OpenRouterError> {
    let url = format!("{base_url}/keys");
    let query = ListApiKeysQuery {
        offset: pagination.and_then(|p| p.offset),
        include_disabled,
        workspace_id: workspace_id.map(ToOwned::to_owned),
    };
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    let response = if query.offset.is_none()
        && query.include_disabled.is_none()
        && query.workspace_id.is_none()
    {
        req.send().await?
    } else {
        req.query(&query).send().await?
    };

    if response.status().is_success() {
        let api_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "API key list").await?;
        Ok(api_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Creates a new API key. Requires a management API key.
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `management_key` - The management API key for authentication.
/// * `name` - The display name for the new API key.
/// * `limit` - Optional credit limit for the new API key.
///
/// # Returns
///
/// * `Result<ApiKey, OpenRouterError>` - The created API key.
pub async fn create_api_key(
    base_url: &str,
    management_key: &str,
    name: &str,
    limit: Option<f64>,
) -> Result<ApiKey, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_api_key_in_workspace_with_client(
        &http_client,
        base_url,
        management_key,
        name,
        limit,
        None,
    )
    .await
}

pub(crate) async fn create_api_key_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    name: &str,
    limit: Option<f64>,
) -> Result<ApiKey, OpenRouterError> {
    create_api_key_in_workspace_with_client(
        http_client,
        base_url,
        management_key,
        name,
        limit,
        None,
    )
    .await
}

pub async fn create_api_key_in_workspace(
    base_url: &str,
    management_key: &str,
    name: &str,
    limit: Option<f64>,
    workspace_id: Option<&str>,
) -> Result<ApiKey, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_api_key_in_workspace_with_client(
        &http_client,
        base_url,
        management_key,
        name,
        limit,
        workspace_id,
    )
    .await
}

pub(crate) async fn create_api_key_in_workspace_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    name: &str,
    limit: Option<f64>,
    workspace_id: Option<&str>,
) -> Result<ApiKey, OpenRouterError> {
    let url = format!("{base_url}/keys");
    let request = CreateApiKeyRequest {
        name: name.to_string(),
        limit,
        workspace_id: workspace_id.map(ToOwned::to_owned),
    };

    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(&request)
    .send()
    .await?;

    if response.status().is_success() {
        let api_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "API key creation").await?;
        Ok(api_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Returns details about a specific API key. Requires a management API key.
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `management_key` - The management API key for authentication.
/// * `hash` - The hash of the API key to retrieve.
///
/// # Returns
///
/// * `Result<ApiKey, OpenRouterError>` - The details of the specified API key.
pub async fn get_api_key(
    base_url: &str,
    management_key: &str,
    hash: &str,
) -> Result<ApiKey, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_api_key_with_client(&http_client, base_url, management_key, hash).await
}

pub(crate) async fn get_api_key_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    hash: &str,
) -> Result<ApiKey, OpenRouterError> {
    let url = format!("{base_url}/keys/{hash}");

    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let api_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "API key").await?;
        Ok(api_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Deletes an API key. Requires a management API key.
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `management_key` - The management API key for authentication.
/// * `hash` - The hash of the API key to delete.
///
/// # Returns
///
/// * `Result<bool, OpenRouterError>` - A boolean indicating whether the deletion was successful.
pub async fn delete_api_key(
    base_url: &str,
    management_key: &str,
    hash: &str,
) -> Result<bool, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_api_key_with_client(&http_client, base_url, management_key, hash).await
}

pub(crate) async fn delete_api_key_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    hash: &str,
) -> Result<bool, OpenRouterError> {
    let url = format!("{base_url}/keys/{hash}");

    let response = transport_request::with_bearer_auth(
        transport_request::delete(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        Ok(true)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Updates an existing API key. Requires a management API key.
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `management_key` - The management API key for authentication.
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
    management_key: &str,
    hash: &str,
    name: Option<String>,
    disabled: Option<bool>,
    limit: Option<f64>,
) -> Result<ApiKey, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    update_api_key_with_client(
        &http_client,
        base_url,
        management_key,
        hash,
        name,
        disabled,
        limit,
    )
    .await
}

pub(crate) async fn update_api_key_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
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

    let response = transport_request::with_bearer_auth(
        transport_request::patch(http_client, &url),
        management_key,
    )
    .json(&request)
    .send()
    .await?;

    if response.status().is_success() {
        let api_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "API key update").await?;
        Ok(api_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
