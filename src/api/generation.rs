use crate::{error::OpenRouterError, types::ApiResponse, utils::handle_error};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerationRequest {
    id: String,
}

impl GenerationRequest {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerationData {
    id: String,
    total_cost: f64,
    created_at: String,
    model: String,
    origin: String,
    usage: f64,
    is_byok: bool,
    upstream_id: Option<String>,
    cache_discount: Option<f64>,
    app_id: Option<u32>,
    streamed: Option<bool>,
    cancelled: Option<bool>,
    provider_name: Option<String>,
    latency: Option<u32>,
    moderation_latency: Option<u32>,
    generation_time: Option<u32>,
    finish_reason: Option<String>,
    native_finish_reason: Option<String>,
    tokens_prompt: Option<u32>,
    tokens_completion: Option<u32>,
    native_tokens_prompt: Option<u32>,
    native_tokens_completion: Option<u32>,
    native_tokens_reasoning: Option<u32>,
    num_media_prompt: Option<u32>,
    num_media_completion: Option<u32>,
    num_search_results: Option<u32>,
}

/// Returns metadata about a specific generation request
///
/// # Arguments
///
/// * `client` - The HTTP client to use for the request
/// * `api_key` - The API key for authentication
/// * `request` - The GenerationRequest containing the ID of the generation request
///
/// # Returns
///
/// * `Result<GenerationData, OpenRouterError>` - The metadata of the generation request or an error
pub async fn get_generation(
    client: &Client,
    api_key: &str,
    request: &GenerationRequest,
) -> Result<GenerationData, OpenRouterError> {
    let url = "https://openrouter.ai/api/v1/generation";

    let response = client
        .get(url)
        .bearer_auth(api_key)
        .query(&[("id", &request.id)])
        .send()
        .await?;

    if response.status().is_success() {
        let generation_response = response.json::<ApiResponse<GenerationData>>().await?;
        Ok(generation_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
