use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

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

/// List SCIM groups (`GET /scim/groups`).
pub async fn list_scim_groups(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<ListScimGroupsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_scim_groups_with_client(&http_client, base_url, management_key, pagination).await
}

pub(crate) async fn list_scim_groups_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<ListScimGroupsResponse, OpenRouterError> {
    let query = PaginationQuery {
        offset: pagination.and_then(|value| value.offset),
        limit: pagination.and_then(|value| value.limit),
    };
    let request = transport_request::with_bearer_auth(
        transport_request::get(http_client, &format!("{base_url}/scim/groups")),
        management_key,
    );
    let response = if query.offset.is_none() && query.limit.is_none() {
        request.send().await?
    } else {
        request.query(&query).send().await?
    };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "SCIM group list").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// List SCIM group mappings (`GET /scim/group-mappings`).
pub async fn list_scim_group_mappings(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<ListScimGroupMappingsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_scim_group_mappings_with_client(&http_client, base_url, management_key, pagination).await
}

pub(crate) async fn list_scim_group_mappings_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<ListScimGroupMappingsResponse, OpenRouterError> {
    let query = PaginationQuery {
        offset: pagination.and_then(|value| value.offset),
        limit: pagination.and_then(|value| value.limit),
    };
    let request = transport_request::with_bearer_auth(
        transport_request::get(http_client, &format!("{base_url}/scim/group-mappings")),
        management_key,
    );
    let response = if query.offset.is_none() && query.limit.is_none() {
        request.send().await?
    } else {
        request.query(&query).send().await?
    };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "SCIM group mapping list").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Create a SCIM group mapping (`POST /scim/group-mappings`).
pub async fn create_scim_group_mapping(
    base_url: &str,
    management_key: &str,
    request: &CreateScimGroupMappingRequest,
) -> Result<ScimGroupMapping, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_scim_group_mapping_with_client(&http_client, base_url, management_key, request).await
}

pub(crate) async fn create_scim_group_mapping_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    request: &CreateScimGroupMappingRequest,
) -> Result<ScimGroupMapping, OpenRouterError> {
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &format!("{base_url}/scim/group-mappings")),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let response: ApiResponse<ScimGroupMapping> =
            transport_response::parse_json_response(response, "SCIM group mapping creation")
                .await?;
        Ok(response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Get one SCIM group mapping (`GET /scim/group-mappings/{id}`).
pub async fn get_scim_group_mapping(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<ScimGroupMapping, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_scim_group_mapping_with_client(&http_client, base_url, management_key, id).await
}

pub(crate) async fn get_scim_group_mapping_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<ScimGroupMapping, OpenRouterError> {
    let response = transport_request::with_bearer_auth(
        transport_request::get(
            http_client,
            &format!("{base_url}/scim/group-mappings/{}", encode(id)),
        ),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let response: ApiResponse<ScimGroupMapping> =
            transport_response::parse_json_response(response, "SCIM group mapping").await?;
        Ok(response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Update one SCIM group mapping (`PATCH /scim/group-mappings/{id}`).
pub async fn update_scim_group_mapping(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateScimGroupMappingRequest,
) -> Result<ScimGroupMapping, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    update_scim_group_mapping_with_client(&http_client, base_url, management_key, id, request).await
}

pub(crate) async fn update_scim_group_mapping_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateScimGroupMappingRequest,
) -> Result<ScimGroupMapping, OpenRouterError> {
    let response = transport_request::with_bearer_auth(
        transport_request::patch(
            http_client,
            &format!("{base_url}/scim/group-mappings/{}", encode(id)),
        ),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let response: ApiResponse<ScimGroupMapping> =
            transport_response::parse_json_response(response, "SCIM group mapping update").await?;
        Ok(response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Delete one SCIM group mapping (`DELETE /scim/group-mappings/{id}`).
pub async fn delete_scim_group_mapping(
    base_url: &str,
    management_key: &str,
    id: &str,
    keep_members: bool,
) -> Result<bool, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_scim_group_mapping_with_client(&http_client, base_url, management_key, id, keep_members)
        .await
}

pub(crate) async fn delete_scim_group_mapping_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    keep_members: bool,
) -> Result<bool, OpenRouterError> {
    let response = transport_request::with_bearer_auth(
        transport_request::delete(
            http_client,
            &format!("{base_url}/scim/group-mappings/{}", encode(id)),
        ),
        management_key,
    )
    .query(&KeepMembersQuery { keep_members })
    .send()
    .await?;

    if response.status().is_success() {
        let response: DeleteScimGroupMappingResponse =
            transport_response::parse_json_response(response, "SCIM group mapping deletion")
                .await?;
        Ok(response.deleted)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
