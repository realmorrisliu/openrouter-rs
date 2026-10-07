use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ApiResponse,
};

/// An end-user registration. Active state does not authorize or block inference.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct EndUser {
    pub user: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ListEndUsersResponse {
    pub data: Vec<EndUser>,
    pub total_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateEndUserRequest {
    #[builder(setter(into))]
    pub user: String,
}

impl CreateEndUserRequest {
    pub fn builder() -> CreateEndUserRequestBuilder {
        CreateEndUserRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateEndUserRequest {
    pub is_active: bool,
}

impl UpdateEndUserRequest {
    pub fn builder() -> UpdateEndUserRequestBuilder {
        UpdateEndUserRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ListEndUsersParams {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_inactive: Option<bool>,
}

impl ListEndUsersParams {
    pub fn builder() -> ListEndUsersParamsBuilder {
        ListEndUsersParamsBuilder::default()
    }
}

/// GET `/end-users`.
pub async fn list_end_users(
    base_url: &str,
    management_key: &str,
    params: &ListEndUsersParams,
) -> Result<ListEndUsersResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_end_users_with_client(
        &http_client,
        base_url,
        management_key,
        (&None, &None, &None),
        params,
    )
    .await
}

pub(crate) async fn list_end_users_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    params: &ListEndUsersParams,
) -> Result<ListEndUsersResponse, OpenRouterError> {
    let url = format!("{base_url}/end-users");
    let response = transport_request::with_client_request_headers(
        transport_request::get(http_client, &url),
        management_key,
        metadata.0,
        metadata.1,
        metadata.2,
    )?
    .query(params)
    .send()
    .await?;
    if response.status().is_success() {
        transport_response::parse_json_response(response, "list_end_users").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// POST `/end-users`.
pub async fn create_end_user(
    base_url: &str,
    management_key: &str,
    request: &CreateEndUserRequest,
) -> Result<EndUser, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_end_user_with_client(
        &http_client,
        base_url,
        management_key,
        (&None, &None, &None),
        request,
    )
    .await
}

pub(crate) async fn create_end_user_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    request: &CreateEndUserRequest,
) -> Result<EndUser, OpenRouterError> {
    let url = format!("{base_url}/end-users");
    let response = transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        management_key,
        metadata.0,
        metadata.1,
        metadata.2,
    )?
    .json(request)
    .send()
    .await?;
    if response.status().is_success() {
        let payload: ApiResponse<EndUser> =
            transport_response::parse_json_response(response, "create_end_user").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// GET `/end-users/{user}`.
pub async fn get_end_user(
    base_url: &str,
    management_key: &str,
    user: &str,
) -> Result<EndUser, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_end_user_with_client(
        &http_client,
        base_url,
        management_key,
        (&None, &None, &None),
        user,
    )
    .await
}

pub(crate) async fn get_end_user_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    user: &str,
) -> Result<EndUser, OpenRouterError> {
    let url = format!("{base_url}/end-users/{}", encode(user));
    let response = transport_request::with_client_request_headers(
        transport_request::get(http_client, &url),
        management_key,
        metadata.0,
        metadata.1,
        metadata.2,
    )?
    .send()
    .await?;
    if response.status().is_success() {
        let payload: ApiResponse<EndUser> =
            transport_response::parse_json_response(response, "get_end_user").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// PATCH `/end-users/{user}`.
pub async fn update_end_user(
    base_url: &str,
    management_key: &str,
    user: &str,
    request: &UpdateEndUserRequest,
) -> Result<EndUser, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    update_end_user_with_client(
        &http_client,
        base_url,
        management_key,
        (&None, &None, &None),
        user,
        request,
    )
    .await
}

pub(crate) async fn update_end_user_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    user: &str,
    request: &UpdateEndUserRequest,
) -> Result<EndUser, OpenRouterError> {
    let url = format!("{base_url}/end-users/{}", encode(user));
    let response = transport_request::with_client_request_headers(
        transport_request::patch(http_client, &url),
        management_key,
        metadata.0,
        metadata.1,
        metadata.2,
    )?
    .json(request)
    .send()
    .await?;
    if response.status().is_success() {
        let payload: ApiResponse<EndUser> =
            transport_response::parse_json_response(response, "update_end_user").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// DELETE `/end-users/{user}`. Soft-deactivates registration; does not block inference.
pub async fn delete_end_user(
    base_url: &str,
    management_key: &str,
    user: &str,
) -> Result<(), OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_end_user_with_client(
        &http_client,
        base_url,
        management_key,
        (&None, &None, &None),
        user,
    )
    .await
}

pub(crate) async fn delete_end_user_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    user: &str,
) -> Result<(), OpenRouterError> {
    let url = format!("{base_url}/end-users/{}", encode(user));
    let response = transport_request::with_client_request_headers(
        transport_request::delete(http_client, &url),
        management_key,
        metadata.0,
        metadata.1,
        metadata.2,
    )?
    .send()
    .await?;
    if response.status().is_success() {
        Ok(())
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
