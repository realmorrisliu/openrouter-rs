use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    strip_option_vec_setter,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

#[derive(Serialize)]
struct ListByokKeysQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
}

/// Bring-your-own-key provider credential returned by `/byok`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ByokKey {
    pub id: String,
    pub provider: String,
    pub workspace_id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub disabled: bool,
    pub is_fallback: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_api_key_hashes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_user_ids: Option<Vec<String>>,
    pub sort_order: i64,
    pub created_at: String,
}

/// Paginated BYOK credential list response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ByokKeyListResponse {
    pub data: Vec<ByokKey>,
    pub total_count: u64,
}

/// Request payload for creating a BYOK credential (`POST /byok`).
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateByokKeyRequest {
    #[builder(setter(into))]
    pub provider: String,
    #[builder(setter(into))]
    pub key: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_models: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_user_ids: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_fallback: Option<bool>,
}

impl CreateByokKeyRequest {
    pub fn builder() -> CreateByokKeyRequestBuilder {
        CreateByokKeyRequestBuilder::default()
    }
}

impl CreateByokKeyRequestBuilder {
    strip_option_vec_setter!(allowed_models, String);
    strip_option_vec_setter!(allowed_user_ids, String);
}

/// Request payload for updating a BYOK credential (`PATCH /byok/{id}`).
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateByokKeyRequest {
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_models: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_user_ids: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_fallback: Option<bool>,
}

impl UpdateByokKeyRequest {
    pub fn builder() -> UpdateByokKeyRequestBuilder {
        UpdateByokKeyRequestBuilder::default()
    }
}

impl UpdateByokKeyRequestBuilder {
    strip_option_vec_setter!(allowed_models, String);
    strip_option_vec_setter!(allowed_user_ids, String);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DeleteByokKeyResponse {
    deleted: bool,
}

/// List BYOK provider credentials (`GET /byok`). Requires a management key.
pub async fn list_byok_keys(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
    workspace_id: Option<&str>,
    provider: Option<&str>,
) -> Result<ByokKeyListResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_byok_keys_with_client(
        &http_client,
        base_url,
        management_key,
        pagination,
        workspace_id,
        provider,
    )
    .await
}

pub(crate) async fn list_byok_keys_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
    workspace_id: Option<&str>,
    provider: Option<&str>,
) -> Result<ByokKeyListResponse, OpenRouterError> {
    let url = format!("{base_url}/byok");
    let query = ListByokKeysQuery {
        offset: pagination.and_then(|p| p.offset),
        limit: pagination.and_then(|p| p.limit),
        workspace_id: workspace_id.map(ToOwned::to_owned),
        provider: provider.map(ToOwned::to_owned),
    };
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    let response = if query.offset.is_none()
        && query.limit.is_none()
        && query.workspace_id.is_none()
        && query.provider.is_none()
    {
        req.send().await?
    } else {
        req.query(&query).send().await?
    };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "BYOK key list").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Create a BYOK provider credential (`POST /byok`). Requires a management key.
pub async fn create_byok_key(
    base_url: &str,
    management_key: &str,
    request: &CreateByokKeyRequest,
) -> Result<ByokKey, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_byok_key_with_client(&http_client, base_url, management_key, request).await
}

pub(crate) async fn create_byok_key_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    request: &CreateByokKeyRequest,
) -> Result<ByokKey, OpenRouterError> {
    let url = format!("{base_url}/byok");
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<ByokKey> =
            transport_response::parse_json_response(response, "BYOK key creation").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Get a BYOK provider credential (`GET /byok/{id}`). Requires a management key.
pub async fn get_byok_key(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<ByokKey, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_byok_key_with_client(&http_client, base_url, management_key, id).await
}

pub(crate) async fn get_byok_key_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<ByokKey, OpenRouterError> {
    let url = format!("{base_url}/byok/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<ByokKey> =
            transport_response::parse_json_response(response, "BYOK key lookup").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Update a BYOK provider credential (`PATCH /byok/{id}`). Requires a management key.
pub async fn update_byok_key(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateByokKeyRequest,
) -> Result<ByokKey, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    update_byok_key_with_client(&http_client, base_url, management_key, id, request).await
}

pub(crate) async fn update_byok_key_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateByokKeyRequest,
) -> Result<ByokKey, OpenRouterError> {
    let url = format!("{base_url}/byok/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::patch(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<ByokKey> =
            transport_response::parse_json_response(response, "BYOK key update").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Delete a BYOK provider credential (`DELETE /byok/{id}`). Requires a management key.
pub async fn delete_byok_key(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_byok_key_with_client(&http_client, base_url, management_key, id).await
}

pub(crate) async fn delete_byok_key_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let url = format!("{base_url}/byok/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::delete(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: DeleteByokKeyResponse =
            transport_response::parse_json_response(response, "BYOK key deletion").await?;
        Ok(payload.deleted)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
