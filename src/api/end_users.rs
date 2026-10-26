use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::transport::RequestContext;
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

pub(crate) async fn list_end_users_with_client(
    ctx: &RequestContext<'_>,
    params: &ListEndUsersParams,
) -> Result<ListEndUsersResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/end-users");
    let req = transport_request::with_client_request_headers(
        transport_request::get(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .query(params);
    transport_response::send_json(req, "list_end_users").await
}

pub(crate) async fn create_end_user_with_client(
    ctx: &RequestContext<'_>,
    request: &CreateEndUserRequest,
) -> Result<EndUser, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/end-users");
    let req = transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(request);
    let payload: ApiResponse<EndUser> =
        transport_response::send_json(req, "create_end_user").await?;
    Ok(payload.data)
}

pub(crate) async fn get_end_user_with_client(
    ctx: &RequestContext<'_>,
    user: &str,
) -> Result<EndUser, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/end-users/{}", encode(user));
    let req = transport_request::with_client_request_headers(
        transport_request::get(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?;
    let payload: ApiResponse<EndUser> = transport_response::send_json(req, "get_end_user").await?;
    Ok(payload.data)
}

pub(crate) async fn update_end_user_with_client(
    ctx: &RequestContext<'_>,
    user: &str,
    request: &UpdateEndUserRequest,
) -> Result<EndUser, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/end-users/{}", encode(user));
    let req = transport_request::with_client_request_headers(
        transport_request::patch(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(request);
    let payload: ApiResponse<EndUser> =
        transport_response::send_json(req, "update_end_user").await?;
    Ok(payload.data)
}

pub(crate) async fn delete_end_user_with_client(
    ctx: &RequestContext<'_>,
    user: &str,
) -> Result<(), OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/end-users/{}", encode(user));
    transport_response::send_unit(transport_request::with_client_request_headers(
        transport_request::delete(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?)
    .await
}
