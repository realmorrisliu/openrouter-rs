use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::transport::RequestContext;
use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ApiResponse,
};

/// An external-user registration. Active state does not authorize or block inference.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ExternalUser {
    pub user: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ListExternalUsersResponse {
    pub data: Vec<ExternalUser>,
    pub total_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateExternalUserRequest {
    #[builder(setter(into))]
    pub user: String,
}

impl CreateExternalUserRequest {
    pub fn builder() -> CreateExternalUserRequestBuilder {
        CreateExternalUserRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateExternalUserRequest {
    pub is_active: bool,
}

impl UpdateExternalUserRequest {
    pub fn builder() -> UpdateExternalUserRequestBuilder {
        UpdateExternalUserRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ListExternalUsersParams {
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

impl ListExternalUsersParams {
    pub fn builder() -> ListExternalUsersParamsBuilder {
        ListExternalUsersParamsBuilder::default()
    }
}

pub(crate) async fn list_external_users_with_client(
    ctx: &RequestContext<'_>,
    params: &ListExternalUsersParams,
) -> Result<ListExternalUsersResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/external-users");
    let req = transport_request::with_client_request_headers(
        transport_request::get(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .query(params);
    transport_response::send_json(req, "list_external_users").await
}

pub(crate) async fn create_external_user_with_client(
    ctx: &RequestContext<'_>,
    request: &CreateExternalUserRequest,
) -> Result<ExternalUser, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/external-users");
    let req = transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(request);
    let payload: ApiResponse<ExternalUser> =
        transport_response::send_json(req, "create_external_user").await?;
    Ok(payload.data)
}

pub(crate) async fn get_external_user_with_client(
    ctx: &RequestContext<'_>,
    user: &str,
) -> Result<ExternalUser, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/external-users/{}", encode(user));
    let req = transport_request::with_client_request_headers(
        transport_request::get(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?;
    let payload: ApiResponse<ExternalUser> =
        transport_response::send_json(req, "get_external_user").await?;
    Ok(payload.data)
}

pub(crate) async fn update_external_user_with_client(
    ctx: &RequestContext<'_>,
    user: &str,
    request: &UpdateExternalUserRequest,
) -> Result<ExternalUser, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/external-users/{}", encode(user));
    let req = transport_request::with_client_request_headers(
        transport_request::patch(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(request);
    let payload: ApiResponse<ExternalUser> =
        transport_response::send_json(req, "update_external_user").await?;
    Ok(payload.data)
}

pub(crate) async fn delete_external_user_with_client(
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
    let url = format!("{base_url}/external-users/{}", encode(user));
    transport_response::send_unit(transport_request::with_client_request_headers(
        transport_request::delete(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?)
    .await
}
