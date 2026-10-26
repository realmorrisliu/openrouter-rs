use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::transport::RequestContext;
use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

/// One organization member returned by `GET /organization/members`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OrganizationMember {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    pub email: String,
    pub role: String,
}

/// Paginated organization member list.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OrganizationMembersResponse {
    pub data: Vec<OrganizationMember>,
    pub total_count: u64,
}

fn with_pagination(url: String, pagination: Option<PaginationOptions>) -> String {
    let params = pagination
        .map(PaginationOptions::to_query_pairs)
        .unwrap_or_default()
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();

    if params.is_empty() {
        url
    } else {
        format!("{url}?{}", params.join("&"))
    }
}

pub(crate) async fn list_organization_members_with_client(
    ctx: &RequestContext<'_>,
    pagination: Option<PaginationOptions>,
) -> Result<OrganizationMembersResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = with_pagination(format!("{base_url}/organization/members"), pagination);
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    transport_response::send_json(req, "organization members").await
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OrganizationSettings {
    pub id: String,
    pub is_filtered_model_catalog_enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateOrganizationSettingsRequest {
    pub is_filtered_model_catalog_enabled: bool,
}

impl UpdateOrganizationSettingsRequest {
    pub fn builder() -> UpdateOrganizationSettingsRequestBuilder {
        UpdateOrganizationSettingsRequestBuilder::default()
    }
}

pub(crate) async fn get_organization_settings_with_client(
    ctx: &RequestContext<'_>,
) -> Result<OrganizationSettings, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/organization/settings");
    let req = transport_request::with_client_request_headers(
        transport_request::get(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?;
    let payload: ApiResponse<OrganizationSettings> =
        transport_response::send_json(req, "get_organization_settings").await?;
    Ok(payload.data)
}

pub(crate) async fn update_organization_settings_with_client(
    ctx: &RequestContext<'_>,
    request: &UpdateOrganizationSettingsRequest,
) -> Result<OrganizationSettings, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/organization/settings");
    let req = transport_request::with_client_request_headers(
        transport_request::patch(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(request);
    let payload: ApiResponse<OrganizationSettings> =
        transport_response::send_json(req, "update_organization_settings").await?;
    Ok(payload.data)
}
