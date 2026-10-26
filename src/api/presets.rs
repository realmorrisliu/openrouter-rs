use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

use crate::transport::RequestContext;
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

pub(crate) async fn list_presets_with_client(
    ctx: &RequestContext<'_>,
    pagination: Option<PaginationOptions>,
) -> Result<ListPresetsResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = with_pagination(format!("{base_url}/presets"), pagination);
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    transport_response::send_json(req, "preset list").await
}

pub(crate) async fn get_preset_with_client(
    ctx: &RequestContext<'_>,
    slug: &str,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let encoded_slug = encode(slug);
    let url = format!("{base_url}/presets/{encoded_slug}");
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    let payload: ApiResponse<PresetWithDesignatedVersion> =
        transport_response::send_json(req, "preset").await?;
    Ok(payload.data)
}

pub(crate) async fn list_preset_versions_with_client(
    ctx: &RequestContext<'_>,
    slug: &str,
    pagination: Option<PaginationOptions>,
) -> Result<ListPresetVersionsResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let encoded_slug = encode(slug);
    let url = with_pagination(
        format!("{base_url}/presets/{encoded_slug}/versions"),
        pagination,
    );
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    transport_response::send_json(req, "preset version list").await
}

pub(crate) async fn get_preset_version_with_client(
    ctx: &RequestContext<'_>,
    slug: &str,
    version: &str,
) -> Result<PresetDesignatedVersion, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let encoded_slug = encode(slug);
    let encoded_version = encode(version);
    let url = format!("{base_url}/presets/{encoded_slug}/versions/{encoded_version}");
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    let payload: ApiResponse<PresetDesignatedVersion> =
        transport_response::send_json(req, "preset version").await?;
    Ok(payload.data)
}

async fn create_preset_with_client<T: Serialize + ?Sized>(
    ctx: &RequestContext<'_>,
    slug: &str,
    suffix: &str,
    request: &T,
    context: &str,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = preset_url(base_url, slug, suffix);
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request);
    let parsed: ApiResponse<PresetWithDesignatedVersion> =
        transport_response::send_json(req, context).await?;
    Ok(parsed.data)
}

pub(crate) async fn create_chat_completion_preset_with_client(
    ctx: &RequestContext<'_>,
    slug: &str,
    request: &ChatCompletionRequest,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    create_preset_with_client(
        ctx,
        slug,
        "chat/completions",
        request,
        "chat completion preset",
    )
    .await
}

pub(crate) async fn create_response_preset_with_client(
    ctx: &RequestContext<'_>,
    slug: &str,
    request: &ResponsesRequest,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    create_preset_with_client(ctx, slug, "responses", request, "response preset").await
}

pub(crate) async fn create_message_preset_with_client(
    ctx: &RequestContext<'_>,
    slug: &str,
    request: &AnthropicMessagesRequest,
) -> Result<PresetWithDesignatedVersion, OpenRouterError> {
    create_preset_with_client(ctx, slug, "messages", request, "message preset").await
}
