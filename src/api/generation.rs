use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;

use crate::{error::OpenRouterError, types::ApiResponse, utils::handle_error};

#[derive(Serialize, Deserialize, Debug)]
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
    pub num_media_prompt: Option<u32>,
    pub num_media_completion: Option<u32>,
    pub num_search_results: Option<u32>,
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
    let id = id.into();
    let url = format!("{base_url}/generation?id={id}");

    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .await?;

    if response.status().is_success() {
        let generation_response: ApiResponse<_> = response.body_json().await?;
        Ok(generation_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
