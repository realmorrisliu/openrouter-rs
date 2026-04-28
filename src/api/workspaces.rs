use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

#[derive(Serialize)]
struct ListWorkspacesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_text_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_image_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider_sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub io_logging_api_key_ids: Option<Vec<u64>>,
    pub io_logging_sampling_rate: f64,
    pub is_observability_io_logging_enabled: bool,
    pub is_observability_broadcast_enabled: bool,
    pub is_data_discount_logging_enabled: bool,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WorkspaceListResponse {
    pub data: Vec<Workspace>,
    pub total_count: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateWorkspaceRequest {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_text_model: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_image_model: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider_sort: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub io_logging_api_key_ids: Option<Vec<u64>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub io_logging_sampling_rate: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_data_discount_logging_enabled: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_observability_broadcast_enabled: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_observability_io_logging_enabled: Option<bool>,
}

impl CreateWorkspaceRequest {
    pub fn builder() -> CreateWorkspaceRequestBuilder {
        CreateWorkspaceRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateWorkspaceRequest {
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_text_model: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_image_model: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider_sort: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub io_logging_api_key_ids: Option<Vec<u64>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub io_logging_sampling_rate: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_data_discount_logging_enabled: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_observability_broadcast_enabled: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_observability_io_logging_enabled: Option<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct UpdateWorkspaceRequestWithClearedIoLoggingApiKeyIds<'a> {
    request: &'a UpdateWorkspaceRequest,
}

impl Serialize for UpdateWorkspaceRequestWithClearedIoLoggingApiKeyIds<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(value) = &self.request.name {
            map.serialize_entry("name", value)?;
        }
        if let Some(value) = &self.request.slug {
            map.serialize_entry("slug", value)?;
        }
        if let Some(value) = &self.request.description {
            map.serialize_entry("description", value)?;
        }
        if let Some(value) = &self.request.default_text_model {
            map.serialize_entry("default_text_model", value)?;
        }
        if let Some(value) = &self.request.default_image_model {
            map.serialize_entry("default_image_model", value)?;
        }
        if let Some(value) = &self.request.default_provider_sort {
            map.serialize_entry("default_provider_sort", value)?;
        }
        map.serialize_entry("io_logging_api_key_ids", &Option::<Vec<u64>>::None)?;
        if let Some(value) = &self.request.io_logging_sampling_rate {
            map.serialize_entry("io_logging_sampling_rate", value)?;
        }
        if let Some(value) = &self.request.is_data_discount_logging_enabled {
            map.serialize_entry("is_data_discount_logging_enabled", value)?;
        }
        if let Some(value) = &self.request.is_observability_broadcast_enabled {
            map.serialize_entry("is_observability_broadcast_enabled", value)?;
        }
        if let Some(value) = &self.request.is_observability_io_logging_enabled {
            map.serialize_entry("is_observability_io_logging_enabled", value)?;
        }
        map.end()
    }
}

impl UpdateWorkspaceRequest {
    pub fn builder() -> UpdateWorkspaceRequestBuilder {
        UpdateWorkspaceRequestBuilder::default()
    }

    pub fn with_cleared_io_logging_api_key_ids(
        &self,
    ) -> UpdateWorkspaceRequestWithClearedIoLoggingApiKeyIds<'_> {
        UpdateWorkspaceRequestWithClearedIoLoggingApiKeyIds { request: self }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DeleteWorkspaceResponse {
    deleted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WorkspaceMember {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct WorkspaceMembersRequest {
    pub user_ids: Vec<String>,
}

impl WorkspaceMembersRequest {
    pub fn builder() -> WorkspaceMembersRequestBuilder {
        WorkspaceMembersRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WorkspaceMembersAddResponse {
    pub added_count: f64,
    pub data: Vec<WorkspaceMember>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WorkspaceMembersRemoveResponse {
    pub removed_count: f64,
}

pub async fn list_workspaces(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<WorkspaceListResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_workspaces_with_client(&http_client, base_url, management_key, pagination).await
}

pub(crate) async fn list_workspaces_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<WorkspaceListResponse, OpenRouterError> {
    let url = format!("{base_url}/workspaces");
    let query = ListWorkspacesQuery {
        offset: pagination.and_then(|p| p.offset),
        limit: pagination.and_then(|p| p.limit),
    };
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    let response = if query.offset.is_none() && query.limit.is_none() {
        req.send().await?
    } else {
        req.query(&query).send().await?
    };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "workspace list").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn create_workspace(
    base_url: &str,
    management_key: &str,
    request: &CreateWorkspaceRequest,
) -> Result<Workspace, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_workspace_with_client(&http_client, base_url, management_key, request).await
}

pub(crate) async fn create_workspace_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    request: &CreateWorkspaceRequest,
) -> Result<Workspace, OpenRouterError> {
    let url = format!("{base_url}/workspaces");
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<Workspace> =
            transport_response::parse_json_response(response, "workspace creation").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn get_workspace(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<Workspace, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_workspace_with_client(&http_client, base_url, management_key, id).await
}

pub(crate) async fn get_workspace_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<Workspace, OpenRouterError> {
    let url = format!("{base_url}/workspaces/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<Workspace> =
            transport_response::parse_json_response(response, "workspace lookup").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn update_workspace(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateWorkspaceRequest,
) -> Result<Workspace, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    update_workspace_with_client(&http_client, base_url, management_key, id, request).await
}

pub async fn update_workspace_with_cleared_io_logging_api_key_ids(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateWorkspaceRequest,
) -> Result<Workspace, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    update_workspace_with_cleared_io_logging_api_key_ids_with_client(
        &http_client,
        base_url,
        management_key,
        id,
        request,
    )
    .await
}

pub(crate) async fn update_workspace_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateWorkspaceRequest,
) -> Result<Workspace, OpenRouterError> {
    update_workspace_payload_with_client(http_client, base_url, management_key, id, request).await
}

pub(crate) async fn update_workspace_with_cleared_io_logging_api_key_ids_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateWorkspaceRequest,
) -> Result<Workspace, OpenRouterError> {
    let request = request.with_cleared_io_logging_api_key_ids();
    update_workspace_payload_with_client(http_client, base_url, management_key, id, &request).await
}

async fn update_workspace_payload_with_client<T: Serialize + ?Sized>(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &T,
) -> Result<Workspace, OpenRouterError> {
    let url = format!("{base_url}/workspaces/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::patch(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<Workspace> =
            transport_response::parse_json_response(response, "workspace update").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn delete_workspace(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_workspace_with_client(&http_client, base_url, management_key, id).await
}

pub(crate) async fn delete_workspace_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let url = format!("{base_url}/workspaces/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::delete(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: DeleteWorkspaceResponse =
            transport_response::parse_json_response(response, "workspace deletion").await?;
        Ok(payload.deleted)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn add_workspace_members(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &WorkspaceMembersRequest,
) -> Result<WorkspaceMembersAddResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    add_workspace_members_with_client(&http_client, base_url, management_key, id, request).await
}

pub(crate) async fn add_workspace_members_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &WorkspaceMembersRequest,
) -> Result<WorkspaceMembersAddResponse, OpenRouterError> {
    let url = format!("{base_url}/workspaces/{}/members/add", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "workspace member bulk add").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn remove_workspace_members(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &WorkspaceMembersRequest,
) -> Result<WorkspaceMembersRemoveResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    remove_workspace_members_with_client(&http_client, base_url, management_key, id, request).await
}

pub(crate) async fn remove_workspace_members_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &WorkspaceMembersRequest,
) -> Result<WorkspaceMembersRemoveResponse, OpenRouterError> {
    let url = format!("{base_url}/workspaces/{}/members/remove", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "workspace member bulk removal").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
