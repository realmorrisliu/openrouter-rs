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
    types::ApiResponse,
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
