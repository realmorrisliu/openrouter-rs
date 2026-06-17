use std::collections::HashMap;

use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

use crate::{
    api::{
        chat::ChatCompletionRequest, messages::AnthropicMessagesRequest,
        responses::ResponsesRequest,
    },
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

/// A specific version of a preset, containing persisted config and optional system prompt.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PresetDesignatedVersion {
    pub id: String,
    pub preset_id: String,
    pub creator_id: String,
    pub version: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    pub config: HashMap<String, Value>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Preset metadata without version details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Preset {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub designated_version_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_updated_at: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Paginated preset list response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ListPresetsResponse {
    pub data: Vec<Preset>,
    pub total_count: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Paginated preset-version list response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ListPresetVersionsResponse {
    pub data: Vec<PresetDesignatedVersion>,
    pub total_count: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Preset metadata returned after creating or updating a preset from an inference request body.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PresetWithDesignatedVersion {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub designated_version_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub designated_version: Option<PresetDesignatedVersion>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

fn preset_url(base_url: &str, slug: &str, suffix: &str) -> String {
    let encoded_slug = encode(slug);
    format!("{base_url}/presets/{encoded_slug}/{suffix}")
}

fn with_pagination(url: String, pagination: Option<PaginationOptions>) -> String {
    let Some(pagination) = pagination else {
        return url;
    };

    let query = pagination.to_query_pairs();
    if query.is_empty() {
        return url;
    }

    let query = query
        .into_iter()
        .map(|(key, value)| format!("{key}={}", encode(&value)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{url}?{query}")
}

/// List presets (`GET /presets`).
pub async fn list_presets(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<ListPresetsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_presets_with_client(&http_client, base_url, management_key, pagination).await
}

pub(crate) async fn list_presets_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<ListPresetsResponse, OpenRouterError> {
    let url = with_pagination(format!("{base_url}/presets"), pagination);
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "preset list").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Get a preset with its designated version (`GET /presets/{slug}`).
pub async fn get_preset(
    base_url: &str,
    management_key: &str,
    slug: &str,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_preset_with_client(&http_client, base_url, management_key, slug).await
}

pub(crate) async fn get_preset_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    slug: &str,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    let encoded_slug = encode(slug);
    let url = format!("{base_url}/presets/{encoded_slug}");
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<PresetWithDesignatedVersion> =
            transport_response::parse_json_response(response, "preset").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// List versions for a preset (`GET /presets/{slug}/versions`).
pub async fn list_preset_versions(
    base_url: &str,
    management_key: &str,
    slug: &str,
    pagination: Option<PaginationOptions>,
) -> Result<ListPresetVersionsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_preset_versions_with_client(&http_client, base_url, management_key, slug, pagination).await
}

pub(crate) async fn list_preset_versions_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    slug: &str,
    pagination: Option<PaginationOptions>,
) -> Result<ListPresetVersionsResponse, OpenRouterError> {
    let encoded_slug = encode(slug);
    let url = with_pagination(
        format!("{base_url}/presets/{encoded_slug}/versions"),
        pagination,
    );
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "preset version list").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Get a specific preset version (`GET /presets/{slug}/versions/{version}`).
pub async fn get_preset_version(
    base_url: &str,
    management_key: &str,
    slug: &str,
    version: &str,
) -> Result<PresetDesignatedVersion, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_preset_version_with_client(&http_client, base_url, management_key, slug, version).await
}

pub(crate) async fn get_preset_version_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    slug: &str,
    version: &str,
) -> Result<PresetDesignatedVersion, OpenRouterError> {
    let encoded_slug = encode(slug);
    let encoded_version = encode(version);
    let url = format!("{base_url}/presets/{encoded_slug}/versions/{encoded_version}");
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<PresetDesignatedVersion> =
            transport_response::parse_json_response(response, "preset version").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

async fn create_preset_with_client<T: Serialize + ?Sized>(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    slug: &str,
    suffix: &str,
    request: &T,
    context: &str,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    let url = preset_url(base_url, slug, suffix);
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<PresetWithDesignatedVersion> =
            transport_response::parse_json_response(response, context).await?;
        Ok(parsed.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Create or update a preset from a chat-completions request body.
pub async fn create_chat_completion_preset(
    base_url: &str,
    management_key: &str,
    slug: &str,
    request: &ChatCompletionRequest,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_chat_completion_preset_with_client(&http_client, base_url, management_key, slug, request)
        .await
}

pub(crate) async fn create_chat_completion_preset_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    slug: &str,
    request: &ChatCompletionRequest,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    create_preset_with_client(
        http_client,
        base_url,
        management_key,
        slug,
        "chat/completions",
        request,
        "chat completion preset",
    )
    .await
}

/// Create or update a preset from a Responses API request body.
pub async fn create_response_preset(
    base_url: &str,
    management_key: &str,
    slug: &str,
    request: &ResponsesRequest,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_response_preset_with_client(&http_client, base_url, management_key, slug, request).await
}

pub(crate) async fn create_response_preset_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    slug: &str,
    request: &ResponsesRequest,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    create_preset_with_client(
        http_client,
        base_url,
        management_key,
        slug,
        "responses",
        request,
        "response preset",
    )
    .await
}

/// Create or update a preset from an Anthropic-compatible Messages request body.
pub async fn create_message_preset(
    base_url: &str,
    management_key: &str,
    slug: &str,
    request: &AnthropicMessagesRequest,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_message_preset_with_client(&http_client, base_url, management_key, slug, request).await
}

pub(crate) async fn create_message_preset_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    slug: &str,
    request: &AnthropicMessagesRequest,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    create_preset_with_client(
        http_client,
        base_url,
        management_key,
        slug,
        "messages",
        request,
        "message preset",
    )
    .await
}
