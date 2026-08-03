use std::collections::HashMap;

use derive_builder::Builder;
use reqwest::{Client as HttpClient, multipart};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

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
    req: reqwest::RequestBuilder,
    workspace_id: Option<&str>,
) -> reqwest::RequestBuilder {
    let query = workspace_query(workspace_id);
    if query.workspace_id.is_none() {
        req
    } else {
        req.query(&query)
    }
}

/// List files in the default or selected workspace (`GET /files`).
pub async fn list_files(
    base_url: &str,
    api_key: &str,
    limit: Option<u32>,
    cursor: Option<&str>,
    workspace_id: Option<&str>,
) -> Result<FileListResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_files_with_client(&http_client, base_url, api_key, limit, cursor, workspace_id).await
}

pub(crate) async fn list_files_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    limit: Option<u32>,
    cursor: Option<&str>,
    workspace_id: Option<&str>,
) -> Result<FileListResponse, OpenRouterError> {
    let url = format!("{base_url}/files");
    let query = ListFilesQuery {
        limit,
        cursor: cursor.map(ToOwned::to_owned),
        workspace_id: workspace_id.map(ToOwned::to_owned),
    };
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response =
        if query.limit.is_none() && query.cursor.is_none() && query.workspace_id.is_none() {
            req.send().await?
        } else {
            req.query(&query).send().await?
        };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "file list").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// List files using a provider-native response shape (`GET /files`).
pub async fn list_provider_files(
    base_url: &str,
    api_key: &str,
    params: &ListFilesParams,
) -> Result<ProviderFileListResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_provider_files_with_client(&http_client, base_url, api_key, params).await
}

pub(crate) async fn list_provider_files_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    params: &ListFilesParams,
) -> Result<ProviderFileListResponse, OpenRouterError> {
    let url = format!("{base_url}/files");
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .query(params)
            .send()
            .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "provider file list").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Upload a file into the default or selected workspace (`POST /files`).
pub async fn upload_file(
    base_url: &str,
    api_key: &str,
    request: &UploadFileRequest,
    workspace_id: Option<&str>,
) -> Result<FileMetadata, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    upload_file_with_client(&http_client, base_url, api_key, request, workspace_id).await
}

pub(crate) async fn upload_file_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    request: &UploadFileRequest,
    workspace_id: Option<&str>,
) -> Result<FileMetadata, OpenRouterError> {
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
    let response = apply_workspace_query(req, workspace_id)
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "file upload").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Upload a file using a provider-native response shape (`POST /files`).
pub async fn upload_provider_file(
    base_url: &str,
    api_key: &str,
    request: &UploadFileRequest,
    query: &FileQuery,
) -> Result<ProviderFileMetadata, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    upload_provider_file_with_client(&http_client, base_url, api_key, request, query).await
}

pub(crate) async fn upload_provider_file_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    request: &UploadFileRequest,
    query: &FileQuery,
) -> Result<ProviderFileMetadata, OpenRouterError> {
    let url = format!("{base_url}/files");
    let mut part =
        multipart::Part::bytes(request.content.clone()).file_name(request.filename.clone());
    if let Some(mime_type) = &request.mime_type {
        part = part
            .mime_str(mime_type)
            .map_err(|error| OpenRouterError::ConfigError(error.to_string()))?;
    }
    let form = multipart::Form::new().part("file", part);
    let response =
        transport_request::with_bearer_auth(transport_request::post(http_client, &url), api_key)
            .query(query)
            .multipart(form)
            .send()
            .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "provider file upload").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Get metadata for one file (`GET /files/{file_id}`).
pub async fn get_file_metadata(
    base_url: &str,
    api_key: &str,
    file_id: &str,
    workspace_id: Option<&str>,
) -> Result<FileMetadata, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_file_metadata_with_client(&http_client, base_url, api_key, file_id, workspace_id).await
}

pub(crate) async fn get_file_metadata_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    file_id: &str,
    workspace_id: Option<&str>,
) -> Result<FileMetadata, OpenRouterError> {
    let encoded_id = encode(file_id);
    let url = format!("{base_url}/files/{encoded_id}");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response = apply_workspace_query(req, workspace_id).send().await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "file metadata").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Get metadata using a provider-native response shape (`GET /files/{file_id}`).
pub async fn get_provider_file_metadata(
    base_url: &str,
    api_key: &str,
    file_id: &str,
    query: &FileQuery,
) -> Result<ProviderFileMetadata, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_provider_file_metadata_with_client(&http_client, base_url, api_key, file_id, query).await
}

pub(crate) async fn get_provider_file_metadata_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    file_id: &str,
    query: &FileQuery,
) -> Result<ProviderFileMetadata, OpenRouterError> {
    let url = format!("{base_url}/files/{}", encode(file_id));
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .query(query)
            .send()
            .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "provider file metadata").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Download raw file content (`GET /files/{file_id}/content`).
pub async fn download_file_content(
    base_url: &str,
    api_key: &str,
    file_id: &str,
    workspace_id: Option<&str>,
) -> Result<Vec<u8>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    download_file_content_with_client(&http_client, base_url, api_key, file_id, workspace_id).await
}

pub(crate) async fn download_file_content_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    file_id: &str,
    workspace_id: Option<&str>,
) -> Result<Vec<u8>, OpenRouterError> {
    let encoded_id = encode(file_id);
    let url = format!("{base_url}/files/{encoded_id}/content");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response = apply_workspace_query(req, workspace_id).send().await?;

    if response.status().is_success() {
        Ok(response.bytes().await?.to_vec())
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Download raw content from a selected provider (`GET /files/{file_id}/content`).
pub async fn download_provider_file_content(
    base_url: &str,
    api_key: &str,
    file_id: &str,
    query: &FileQuery,
) -> Result<Vec<u8>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    download_provider_file_content_with_client(&http_client, base_url, api_key, file_id, query)
        .await
}

pub(crate) async fn download_provider_file_content_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    file_id: &str,
    query: &FileQuery,
) -> Result<Vec<u8>, OpenRouterError> {
    let url = format!("{base_url}/files/{}/content", encode(file_id));
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .query(query)
            .send()
            .await?;

    if response.status().is_success() {
        Ok(response.bytes().await?.to_vec())
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Delete one file (`DELETE /files/{file_id}`).
pub async fn delete_file(
    base_url: &str,
    api_key: &str,
    file_id: &str,
    workspace_id: Option<&str>,
) -> Result<FileDeleteResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_file_with_client(&http_client, base_url, api_key, file_id, workspace_id).await
}

pub(crate) async fn delete_file_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    file_id: &str,
    workspace_id: Option<&str>,
) -> Result<FileDeleteResponse, OpenRouterError> {
    let encoded_id = encode(file_id);
    let url = format!("{base_url}/files/{encoded_id}");
    let req =
        transport_request::with_bearer_auth(transport_request::delete(http_client, &url), api_key);
    let response = apply_workspace_query(req, workspace_id).send().await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "file deletion").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Delete a file using a provider-native response shape (`DELETE /files/{file_id}`).
pub async fn delete_provider_file(
    base_url: &str,
    api_key: &str,
    file_id: &str,
    query: &FileQuery,
) -> Result<ProviderFileDeleteResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_provider_file_with_client(&http_client, base_url, api_key, file_id, query).await
}

pub(crate) async fn delete_provider_file_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    file_id: &str,
    query: &FileQuery,
) -> Result<ProviderFileDeleteResponse, OpenRouterError> {
    let url = format!("{base_url}/files/{}", encode(file_id));
    let response =
        transport_request::with_bearer_auth(transport_request::delete(http_client, &url), api_key)
            .query(query)
            .send()
            .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "provider file deletion").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
