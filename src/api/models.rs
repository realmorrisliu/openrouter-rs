use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    types::{ApiResponse, ModelCategory, SupportedParameters},
    utils::handle_error,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub created: f64,
    pub description: String,
    pub context_length: f64,
    pub architecture: Architecture,
    pub top_provider: TopProvider,
    pub pricing: Pricing,
    pub per_request_limits: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Architecture {
    pub modality: String,
    pub tokenizer: String,
    pub instruct_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopProvider {
    pub context_length: Option<f64>,
    pub max_completion_tokens: Option<f64>,
    pub is_moderated: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pricing {
    pub prompt: String,
    pub completion: String,
    pub image: Option<String>,
    pub request: Option<String>,
    pub input_cache_read: Option<String>,
    pub input_cache_write: Option<String>,
    pub web_search: Option<String>,
    pub internal_reasoning: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Endpoint {
    pub name: String,
    pub context_length: f64,
    pub pricing: EndpointPricing,
    pub provider_name: String,
    pub supported_parameters: Vec<String>,
    pub quantization: Option<String>,
    pub max_completion_tokens: Option<f64>,
    pub max_prompt_tokens: Option<f64>,
    pub status: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EndpointPricing {
    pub request: String,
    pub image: String,
    pub prompt: String,
    pub completion: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EndpointData {
    pub id: String,
    pub name: String,
    pub created: f64,
    pub description: String,
    pub architecture: EndpointArchitecture,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EndpointArchitecture {
    pub tokenizer: Option<String>,
    pub instruct_type: Option<String>,
    pub modality: Option<String>,
}

/// Returns a list of models available through the API
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `category` - The category of the models.
///
/// # Returns
///
/// * `Result<Vec<Model>, OpenRouterError>` - A list of models or an error.
pub async fn list_models(
    base_url: &str,
    api_key: &str,
    category: Option<ModelCategory>,
    supported_parameters: Option<SupportedParameters>,
) -> Result<Vec<Model>, OpenRouterError> {
    let url = match (category, supported_parameters) {
        (Some(category), None) => {
            format!("{base_url}/models?category={category}")
        }
        (None, Some(supported_parameters)) => {
            format!("{base_url}/models?supported_parameters={supported_parameters}")
        }
        _ => {
            format!("{base_url}/models")
        }
    };

    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
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
    let encoded_author = encode(author);
    let encoded_slug = encode(slug);
    let url = format!("{base_url}/models/{encoded_author}/{encoded_slug}/endpoints");
    println!("URL: {url}");

    let mut response = surf::get(&url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .await?;

    if response.status().is_success() {
        let endpoint_list_response: ApiResponse<_> = response.body_json().await?;
        Ok(endpoint_list_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
