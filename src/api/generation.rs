use std::collections::HashMap;

use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ApiResponse,
};

#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct GenerationData {
    pub id: String,
    pub total_cost: f64,
    pub created_at: String,
    pub model: String,
    pub origin: String,
    pub usage: f64,
    pub is_byok: bool,
    pub upstream_id: Option<String>,
    pub cache_discount: Option<f64>,
    pub app_id: Option<u32>,
    pub streamed: Option<bool>,
    pub cancelled: Option<bool>,
    pub provider_name: Option<String>,
    pub latency: Option<u32>,
    pub moderation_latency: Option<u32>,
    pub generation_time: Option<u32>,
    pub finish_reason: Option<String>,
    pub native_finish_reason: Option<String>,
    pub tokens_prompt: Option<u32>,
    pub tokens_completion: Option<u32>,
    pub native_tokens_prompt: Option<u32>,
    pub native_tokens_completion: Option<u32>,
    pub native_tokens_reasoning: Option<u32>,
    pub num_fetches: Option<u32>,
    pub num_media_prompt: Option<u32>,
    pub num_media_completion: Option<u32>,
    pub num_search_results: Option<u32>,
    pub response_cache_source_id: Option<String>,
    pub service_tier: Option<String>,
}

/// Stored prompt/input and completion/output content returned by `GET /generation/content`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GenerationContentData {
    #[serde(default)]
    pub input: HashMap<String, Value>,
    #[serde(default)]
    pub output: HashMap<String, Value>,
}

/// Returns metadata about a specific generation request
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication
/// * `id` - The ID of the generation request
///
/// # Returns
///
/// * `Result<GenerationData, OpenRouterError>` - The metadata of the generation request or an error
pub async fn get_generation(
    base_url: &str,
    api_key: &str,
    id: impl Into<String>,
) -> Result<GenerationData, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_generation_with_client(&http_client, base_url, api_key, id).await
}

pub(crate) async fn get_generation_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    id: impl Into<String>,
) -> Result<GenerationData, OpenRouterError> {
    let id = id.into();
    let url = format!("{base_url}/generation?id={id}");

    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let generation_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "generation").await?;
        Ok(generation_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Returns the stored prompt/input and completion/output content for a specific generation.
pub async fn get_generation_content(
    base_url: &str,
    api_key: &str,
    id: impl Into<String>,
) -> Result<GenerationContentData, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_generation_content_with_client(&http_client, base_url, api_key, id).await
}

pub(crate) async fn get_generation_content_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    id: impl Into<String>,
) -> Result<GenerationContentData, OpenRouterError> {
    let id = id.into();
    let url = format!("{base_url}/generation/content?id={id}");

    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let generation_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "generation content").await?;
        Ok(generation_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
