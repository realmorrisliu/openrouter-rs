use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;

use crate::{error::OpenRouterError, types::ApiResponse, utils::handle_error};

#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    id: String,
    name: String,
    created: f64,
    description: String,
    context_length: f64,
    architecture: Architecture,
    top_provider: TopProvider,
    pricing: Pricing,
    per_request_limits: Option<std::collections::HashMap<String, String>>,
}

impl Model {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }
    pub fn display_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn max_token_count(&self) -> u32 {
        self.context_length as u32
    }
    pub fn max_output_tokens(&self) -> Option<u32> {
        self.top_provider
            .max_completion_tokens
            .map(|max_completion_tokens| max_completion_tokens as u32)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Architecture {
    modality: String,
    tokenizer: String,
    instruct_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopProvider {
    context_length: Option<f64>,
    max_completion_tokens: Option<f64>,
    is_moderated: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pricing {
    prompt: String,
    completion: String,
    image: String,
    request: String,
    input_cache_read: String,
    input_cache_write: String,
    web_search: String,
    internal_reasoning: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Endpoint {
    name: String,
    context_length: f64,
    pricing: EndpointPricing,
    provider_name: String,
    supported_parameters: Vec<String>,
    quantization: Option<String>,
    max_completion_tokens: Option<f64>,
    max_prompt_tokens: Option<f64>,
    status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EndpointPricing {
    request: String,
    image: String,
    prompt: String,
    completion: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EndpointData {
    id: String,
    name: String,
    created: f64,
    description: String,
    architecture: EndpointArchitecture,
    endpoints: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EndpointArchitecture {
    tokenizer: Option<String>,
    instruct_type: Option<String>,
    modality: Option<String>,
}

/// Returns a list of models available through the API
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
///
/// # Returns
///
/// * `Result<Vec<Model>, OpenRouterError>` - A list of models or an error.
pub async fn list_models(base_url: &str, api_key: &str) -> Result<Vec<Model>, OpenRouterError> {
    let url = format!("{}/models", base_url);

    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .await?;

    if response.status().is_success() {
        let model_list_response: ApiResponse<_> = response.body_json().await?;
        Ok(model_list_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Returns details about the endpoints for a specific model
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `author` - The author of the model.
/// * `slug` - The slug identifier for the model.
///
/// # Returns
///
/// * `Result<EndpointData, OpenRouterError>` - The endpoint data or an error.
pub async fn list_model_endpoints(
    base_url: &str,
    api_key: &str,
    author: &str,
    slug: &str,
) -> Result<EndpointData, OpenRouterError> {
    let url = format!("{}/models/{}/{}", base_url, author, slug);

    let mut response = surf::get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .await?;

    if response.status().is_success() {
        let endpoint_list_response: ApiResponse<_> = response.body_json().await?;
        Ok(endpoint_list_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
