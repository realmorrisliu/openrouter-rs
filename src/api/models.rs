use crate::{error::OpenRouterError, types::ApiResponse, utils::handle_error};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    id: String,
    name: String,
    created: u64,
    description: String,
    context_length: u32,
    architecture: Architecture,
    top_provider: TopProvider,
    pricing: Pricing,
    per_request_limits: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Architecture {
    modality: String,
    tokenizer: String,
    instruct_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopProvider {
    context_length: f64,
    max_completion_tokens: f64,
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
/// * `client` - The HTTP client to use for the request.
/// * `api_key` - The API key for authentication.
///
/// # Returns
///
/// * `Result<Vec<Model>, OpenRouterError>` - A list of models or an error.
pub async fn list_models(client: &Client, api_key: &str) -> Result<Vec<Model>, OpenRouterError> {
    let url = "https://openrouter.ai/api/v1/models";

    let response = client.get(url).bearer_auth(api_key).send().await?;

    if response.status().is_success() {
        let model_list_response = response.json::<ApiResponse<Vec<Model>>>().await?;
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
/// * `client` - The HTTP client to use for the request.
/// * `api_key` - The API key for authentication.
/// * `author` - The author of the model.
/// * `slug` - The slug identifier for the model.
///
/// # Returns
///
/// * `Result<EndpointData, OpenRouterError>` - The endpoint data or an error.
pub async fn list_model_endpoints(
    client: &Client,
    api_key: &str,
    author: &str,
    slug: &str,
) -> Result<EndpointData, OpenRouterError> {
    let url = format!("https://openrouter.ai/api/v1/models/{}/{}", author, slug);

    let response = client.get(&url).bearer_auth(api_key).send().await?;

    if response.status().is_success() {
        let endpoint_list_response = response.json::<ApiResponse<EndpointData>>().await?;
        Ok(endpoint_list_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
