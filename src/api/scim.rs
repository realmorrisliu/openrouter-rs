use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::transport::RequestContext;
use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

#[derive(Serialize)]
struct PaginationQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
}

#[derive(Serialize)]
struct KeepMembersQuery {
    keep_members: bool,
}

#[derive(Deserialize)]
struct DeleteScimGroupMappingResponse {
    deleted: bool,
}

/// Pagination and exact-name filters for SCIM group discovery.
#[derive(Serialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ListScimGroupsParams {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
}

impl ListScimGroupsParams {
    pub fn builder() -> ListScimGroupsParamsBuilder {
        ListScimGroupsParamsBuilder::default()
    }
}

/// One SCIM group synchronized into the organization.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ScimGroup {
    pub id: String,
    pub organization_id: String,
    pub external_id: Option<String>,
    pub display_name: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Paginated SCIM group response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ListScimGroupsResponse {
    pub data: Vec<ScimGroup>,
    pub total_count: u64,
}

/// One SCIM group-to-workspace role mapping.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ScimGroupMapping {
    pub id: String,
    pub organization_id: String,
    pub scim_group_id: String,
    pub workspace_id: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Paginated SCIM group mapping response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ListScimGroupMappingsResponse {
    pub data: Vec<ScimGroupMapping>,
    pub total_count: u64,
}

/// Request to map a SCIM group to one workspace role.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateScimGroupMappingRequest {
    #[builder(setter(into))]
    pub scim_group_id: String,
    #[builder(setter(into))]
    pub workspace_id: String,
    #[builder(setter(into))]
    pub role: String,
}

impl CreateScimGroupMappingRequest {
    pub fn builder() -> CreateScimGroupMappingRequestBuilder {
        CreateScimGroupMappingRequestBuilder::default()
    }
}

/// Request to update a SCIM group mapping role.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateScimGroupMappingRequest {
    #[builder(setter(into))]
    pub role: String,
}

impl UpdateScimGroupMappingRequest {
    pub fn builder() -> UpdateScimGroupMappingRequestBuilder {
        UpdateScimGroupMappingRequestBuilder::default()
    }
}

pub(crate) async fn list_scim_groups_with_client(
    ctx: &RequestContext<'_>,
    pagination: Option<PaginationOptions>,
) -> Result<ListScimGroupsResponse, OpenRouterError> {
    let params = ListScimGroupsParams {
        offset: pagination.and_then(|value| value.offset),
        limit: pagination.and_then(|value| value.limit),
        ..Default::default()
    };
    list_scim_groups_with_params_with_client(ctx, &params).await
}

pub(crate) async fn list_scim_groups_with_params_with_client(
    ctx: &RequestContext<'_>,
    params: &ListScimGroupsParams,
) -> Result<ListScimGroupsResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let req = transport_request::with_client_request_headers(
        transport_request::get(http_client, &format!("{base_url}/scim/groups")),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .query(params);
    transport_response::send_json(req, "SCIM group list").await
}

pub(crate) async fn list_scim_group_mappings_with_client(
    ctx: &RequestContext<'_>,
    pagination: Option<PaginationOptions>,
) -> Result<ListScimGroupMappingsResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let query = PaginationQuery {
        offset: pagination.and_then(|value| value.offset),
        limit: pagination.and_then(|value| value.limit),
    };
    let request = transport_request::with_bearer_auth(
        transport_request::get(http_client, &format!("{base_url}/scim/group-mappings")),
        management_key,
    );
    let req = if query.offset.is_none() && query.limit.is_none() {
        request
    } else {
        request.query(&query)
    };
    transport_response::send_json(req, "SCIM group mapping list").await
}

pub(crate) async fn create_scim_group_mapping_with_client(
    ctx: &RequestContext<'_>,
    request: &CreateScimGroupMappingRequest,
) -> Result<ScimGroupMapping, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &format!("{base_url}/scim/group-mappings")),
        management_key,
    )
    .json(request);
    let response: ApiResponse<ScimGroupMapping> =
        transport_response::send_json(req, "SCIM group mapping creation").await?;
    Ok(response.data)
}

pub(crate) async fn get_scim_group_mapping_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
) -> Result<ScimGroupMapping, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let req = transport_request::with_bearer_auth(
        transport_request::get(
            http_client,
            &format!("{base_url}/scim/group-mappings/{}", encode(id)),
        ),
        management_key,
    );
    let response: ApiResponse<ScimGroupMapping> =
        transport_response::send_json(req, "SCIM group mapping").await?;
    Ok(response.data)
}

pub(crate) async fn update_scim_group_mapping_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    request: &UpdateScimGroupMappingRequest,
) -> Result<ScimGroupMapping, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let req = transport_request::with_bearer_auth(
        transport_request::patch(
            http_client,
            &format!("{base_url}/scim/group-mappings/{}", encode(id)),
        ),
        management_key,
    )
    .json(request);
    let response: ApiResponse<ScimGroupMapping> =
        transport_response::send_json(req, "SCIM group mapping update").await?;
    Ok(response.data)
}

pub(crate) async fn delete_scim_group_mapping_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    keep_members: bool,
) -> Result<bool, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let req = transport_request::with_bearer_auth(
        transport_request::delete(
            http_client,
            &format!("{base_url}/scim/group-mappings/{}", encode(id)),
        ),
        management_key,
    )
    .query(&KeepMembersQuery { keep_members });
    let response: DeleteScimGroupMappingResponse =
        transport_response::send_json(req, "SCIM group mapping deletion").await?;
    Ok(response.deleted)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ScimSyncJob {
    pub id: String,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error_message: Option<String>,
    pub synced_groups: Option<u64>,
    pub deleted_groups: Option<u64>,
}

pub(crate) async fn create_scim_sync_job_with_client(
    ctx: &RequestContext<'_>,
) -> Result<ScimSyncJob, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/scim/sync-jobs");
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    );

    let req = req;
    let payload: ApiResponse<ScimSyncJob> =
        transport_response::send_json(req, "create_scim_sync_job").await?;
    Ok(payload.data)
}

pub(crate) async fn get_scim_sync_job_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
) -> Result<ScimSyncJob, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/scim/sync-jobs/{}", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );

    let req = req;
    let payload: ApiResponse<ScimSyncJob> =
        transport_response::send_json(req, "get_scim_sync_job").await?;
    Ok(payload.data)
}
