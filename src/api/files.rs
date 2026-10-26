use std::collections::HashMap;

use crate::transport::multipart;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::transport::RequestContext;
use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
};

#[derive(Serialize)]
struct FileWorkspaceQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
}

#[derive(Serialize)]
struct ListFilesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
}

/// Query parameters shared by provider-backed file operations.
#[derive(Serialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"), default)]
#[non_exhaustive]
pub struct FileQuery {
    #[builder(setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[builder(setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

impl FileQuery {
    pub fn builder() -> FileQueryBuilder {
        FileQueryBuilder::default()
    }
}

/// Query parameters for provider-aware file listing.
#[derive(Serialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"), default)]
#[non_exhaustive]
pub struct ListFilesParams {
    #[builder(setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[builder(setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[builder(setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[builder(setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[builder(setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[builder(setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<String>,
    #[builder(setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,
    #[builder(setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

impl ListFilesParams {
    pub fn builder() -> ListFilesParamsBuilder {
        ListFilesParamsBuilder::default()
    }
}

/// Metadata describing a stored file.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FileMetadata {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub downloadable: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A paginated page of files.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FileListResponse {
    pub data: Vec<FileMetadata>,
    pub has_more: bool,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
    pub cursor: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Confirmation that a file was deleted.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FileDeleteResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// File metadata in the shape negotiated with a backing provider.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ProviderFileMetadata {
    #[serde(rename = "_shape")]
    pub shape: String,
    pub id: String,
    pub filename: String,
    #[serde(default, rename = "type")]
    pub object_type: Option<String>,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub bytes: Option<u64>,
    pub created_at: serde_json::Value,
    #[serde(default)]
    pub downloadable: Option<bool>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A provider-shaped page of files.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ProviderFileListResponse {
    #[serde(rename = "_shape")]
    pub shape: String,
    pub data: Vec<ProviderFileMetadata>,
    pub has_more: bool,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Provider-shaped confirmation that a file was deleted.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ProviderFileDeleteResponse {
    #[serde(rename = "_shape")]
    pub shape: String,
    pub id: String,
    #[serde(default, rename = "type")]
    pub object_type: Option<String>,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub deleted: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// File upload payload for `POST /files`.
#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UploadFileRequest {
    #[builder(setter(into))]
    pub filename: String,
    #[builder(setter(into))]
    pub content: Vec<u8>,
    #[builder(setter(into, strip_option), default)]
    pub mime_type: Option<String>,
}

impl UploadFileRequest {
    pub fn builder() -> UploadFileRequestBuilder {
        UploadFileRequestBuilder::default()
    }
}

fn workspace_query(workspace_id: Option<&str>) -> FileWorkspaceQuery {
    FileWorkspaceQuery {
        workspace_id: workspace_id.map(ToOwned::to_owned),
    }
}

fn apply_workspace_query(
    req: crate::transport::RequestBuilder,
    workspace_id: Option<&str>,
) -> crate::transport::RequestBuilder {
    let query = workspace_query(workspace_id);
    if query.workspace_id.is_none() {
        req
    } else {
        req.query(&query)
    }
}

pub(crate) async fn list_files_with_client(
    ctx: &RequestContext<'_>,
    limit: Option<u32>,
    cursor: Option<&str>,
    workspace_id: Option<&str>,
) -> Result<FileListResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/files");
    let query = ListFilesQuery {
        limit,
        cursor: cursor.map(ToOwned::to_owned),
        workspace_id: workspace_id.map(ToOwned::to_owned),
    };
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let req = if query.limit.is_none() && query.cursor.is_none() && query.workspace_id.is_none() {
        req
    } else {
        req.query(&query)
    };
    transport_response::send_json(req, "file list").await
}

pub(crate) async fn list_provider_files_with_client(
    ctx: &RequestContext<'_>,
    params: &ListFilesParams,
) -> Result<ProviderFileListResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/files");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .query(params);
    transport_response::send_json(req, "provider file list").await
}

pub(crate) async fn upload_file_with_client(
    ctx: &RequestContext<'_>,
    request: &UploadFileRequest,
    workspace_id: Option<&str>,
) -> Result<FileMetadata, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/files");
    let mut part =
        multipart::Part::bytes(request.content.clone()).file_name(request.filename.clone());
    if let Some(mime_type) = &request.mime_type {
        part = part
            .mime_str(mime_type)
            .map_err(|error| OpenRouterError::ConfigError(error.to_string()))?;
    }
    let form = multipart::Form::new().part("file", part);
    let req =
        transport_request::with_bearer_auth(transport_request::post(http_client, &url), api_key);
    let req = apply_workspace_query(req, workspace_id).multipart(form);
    transport_response::send_json(req, "file upload").await
}

pub(crate) async fn upload_provider_file_with_client(
    ctx: &RequestContext<'_>,
    request: &UploadFileRequest,
    query: &FileQuery,
) -> Result<ProviderFileMetadata, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/files");
    let mut part =
        multipart::Part::bytes(request.content.clone()).file_name(request.filename.clone());
    if let Some(mime_type) = &request.mime_type {
        part = part
            .mime_str(mime_type)
            .map_err(|error| OpenRouterError::ConfigError(error.to_string()))?;
    }
    let form = multipart::Form::new().part("file", part);
    let req =
        transport_request::with_bearer_auth(transport_request::post(http_client, &url), api_key)
            .query(query)
            .multipart(form);
    transport_response::send_json(req, "provider file upload").await
}

pub(crate) async fn get_file_metadata_with_client(
    ctx: &RequestContext<'_>,
    file_id: &str,
    workspace_id: Option<&str>,
) -> Result<FileMetadata, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let encoded_id = encode(file_id);
    let url = format!("{base_url}/files/{encoded_id}");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let req = apply_workspace_query(req, workspace_id);
    transport_response::send_json(req, "file metadata").await
}

pub(crate) async fn get_provider_file_metadata_with_client(
    ctx: &RequestContext<'_>,
    file_id: &str,
    query: &FileQuery,
) -> Result<ProviderFileMetadata, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/files/{}", encode(file_id));
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .query(query);
    transport_response::send_json(req, "provider file metadata").await
}

pub(crate) async fn download_file_content_with_client(
    ctx: &RequestContext<'_>,
    file_id: &str,
    workspace_id: Option<&str>,
) -> Result<Vec<u8>, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let encoded_id = encode(file_id);
    let url = format!("{base_url}/files/{encoded_id}/content");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response = apply_workspace_query(req, workspace_id).send().await?;

    if response.status().is_success() {
        Ok(response.bytes().await?.to_vec())
    } else {
        Err(transport_response::error_from_response(response).await)
    }
}

pub(crate) async fn download_provider_file_content_with_client(
    ctx: &RequestContext<'_>,
    file_id: &str,
    query: &FileQuery,
) -> Result<Vec<u8>, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/files/{}/content", encode(file_id));
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .query(query)
            .send()
            .await?;

    if response.status().is_success() {
        Ok(response.bytes().await?.to_vec())
    } else {
        Err(transport_response::error_from_response(response).await)
    }
}

pub(crate) async fn delete_file_with_client(
    ctx: &RequestContext<'_>,
    file_id: &str,
    workspace_id: Option<&str>,
) -> Result<FileDeleteResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let encoded_id = encode(file_id);
    let url = format!("{base_url}/files/{encoded_id}");
    let req =
        transport_request::with_bearer_auth(transport_request::delete(http_client, &url), api_key);
    let req = apply_workspace_query(req, workspace_id);
    transport_response::send_json(req, "file deletion").await
}

pub(crate) async fn delete_provider_file_with_client(
    ctx: &RequestContext<'_>,
    file_id: &str,
    query: &FileQuery,
) -> Result<ProviderFileDeleteResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/files/{}", encode(file_id));
    let req =
        transport_request::with_bearer_auth(transport_request::delete(http_client, &url), api_key)
            .query(query);
    transport_response::send_json(req, "provider file deletion").await
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ContainerFile {
    pub id: String,
    pub object: String,
    pub container_id: String,
    pub created_at: i64,
    pub bytes: u64,
    pub path: String,
    pub source: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ContainerFileListResponse {
    pub object: String,
    pub data: Vec<ContainerFile>,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
    pub has_more: bool,
}

pub(crate) async fn list_container_files_with_client(
    ctx: &RequestContext<'_>,
    container_id: &str,
    after: Option<&str>,
    limit: Option<u32>,
) -> Result<ContainerFileListResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/containers/{}/files", encode(container_id));
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let req = if let Some(after) = after {
        req.query(&[("after", after)])
    } else {
        req
    };
    let req = if let Some(limit) = limit {
        req.query(&[("limit", limit)])
    } else {
        req
    };

    let req = req;
    transport_response::send_json(req, "list_container_files").await
}

pub(crate) async fn get_container_file_with_client(
    ctx: &RequestContext<'_>,
    container_id: &str,
    file_id: &str,
) -> Result<ContainerFile, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!(
        "{base_url}/containers/{}/files/{}",
        encode(container_id),
        encode(file_id)
    );
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);

    let req = req;
    transport_response::send_json(req, "get_container_file").await
}

pub(crate) async fn download_container_file_with_client(
    ctx: &RequestContext<'_>,
    container_id: &str,
    file_id: &str,
) -> Result<Vec<u8>, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!(
        "{base_url}/containers/{}/files/{}/content",
        encode(container_id),
        encode(file_id)
    );
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);

    let response = req.send().await?;
    if response.status().is_success() {
        Ok(response.bytes().await?.to_vec())
    } else {
        Err(transport_response::error_from_response(response).await)
    }
}

pub(crate) async fn promote_container_file_with_client(
    ctx: &RequestContext<'_>,
    container_id: &str,
    file_id: &str,
) -> Result<ProviderFileMetadata, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!(
        "{base_url}/containers/{}/files/{}/promote",
        encode(container_id),
        encode(file_id)
    );
    let req =
        transport_request::with_bearer_auth(transport_request::post(http_client, &url), api_key);

    let req = req;
    transport_response::send_json(req, "promote_container_file").await
}
