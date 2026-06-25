use std::collections::HashMap;

use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, Effort, ModelCategory, SupportedParameters},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Model {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hugging_face_id: Option<String>,
    pub name: String,
    pub created: f64,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_length: Option<f64>,
    pub architecture: Architecture,
    pub top_provider: TopProvider,
    pub pricing: Pricing,
    pub per_request_limits: Option<HashMap<String, String>>,
    #[serde(default)]
    pub supported_parameters: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_voices: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_parameters: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub knowledge_cutoff: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<ModelLinks>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benchmarks: Option<ModelBenchmarks>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ModelReasoning>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Architecture {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruct_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_modalities: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modalities: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TopProvider {
    pub context_length: Option<f64>,
    pub max_completion_tokens: Option<f64>,
    pub is_moderated: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Pricing {
    pub prompt: String,
    pub completion: String,
    pub image: Option<String>,
    pub request: Option<String>,
    pub input_cache_read: Option<String>,
    pub input_cache_write: Option<String>,
    pub web_search: Option<String>,
    pub internal_reasoning: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ModelLinks {
    pub details: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AABenchmarkEntry {
    pub intelligence_index: Option<f64>,
    pub coding_index: Option<f64>,
    pub agentic_index: Option<f64>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DABenchmarkEntry {
    pub arena: String,
    pub category: String,
    pub elo: f64,
    pub win_rate: f64,
    pub rank: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ModelBenchmarks {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artificial_analysis: Option<AABenchmarkEntry>,
    #[serde(default)]
    pub design_arena: Vec<DABenchmarkEntry>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ModelReasoning {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_effort: Option<Effort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_enabled: Option<bool>,
    pub mandatory: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported_efforts: Option<Vec<Effort>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_max_tokens: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct EndpointPricing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    pub prompt: String,
    pub completion: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct EndpointData {
    pub id: String,
    pub name: String,
    pub created: f64,
    pub description: String,
    pub architecture: EndpointArchitecture,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct EndpointArchitecture {
    pub tokenizer: Option<String>,
    pub instruct_type: Option<String>,
    pub modality: Option<String>,
}

/// Extended query parameters for `GET /models`.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ListModelsParams {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<ModelCategory>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_parameters: Option<SupportedParameters>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_modalities: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_modalities: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_price: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<f64>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_authors: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distillable: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zdr: Option<bool>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

impl ListModelsParams {
    pub fn builder() -> ListModelsParamsBuilder {
        ListModelsParamsBuilder::default()
    }

    fn is_empty(&self) -> bool {
        self.category.is_none()
            && self.supported_parameters.is_none()
            && self.output_modalities.is_none()
            && self.sort.is_none()
            && self.q.is_none()
            && self.input_modalities.is_none()
            && self.context.is_none()
            && self.min_price.is_none()
            && self.max_price.is_none()
            && self.arch.is_none()
            && self.model_authors.is_none()
            && self.providers.is_none()
            && self.distillable.is_none()
            && self.zdr.is_none()
            && self.region.is_none()
    }
}

/// Returns a list of models available through the API
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `category` - Optional category filter for the models.
/// * `supported_parameters` - Optional supported-parameter filter for the models.
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
    let http_client = crate::transport::new_client()?;
    let params = ListModelsParams {
        category,
        supported_parameters,
        ..Default::default()
    };
    list_models_with_params_and_client(&http_client, base_url, api_key, Some(&params)).await
}

pub(crate) async fn list_models_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    category: Option<ModelCategory>,
    supported_parameters: Option<SupportedParameters>,
) -> Result<Vec<Model>, OpenRouterError> {
    let params = ListModelsParams {
        category,
        supported_parameters,
        ..Default::default()
    };
    list_models_with_params_and_client(http_client, base_url, api_key, Some(&params)).await
}

/// Returns a list of models using the full upstream filter surface.
pub async fn list_models_with_params(
    base_url: &str,
    api_key: &str,
    params: Option<&ListModelsParams>,
) -> Result<Vec<Model>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_models_with_params_and_client(&http_client, base_url, api_key, params).await
}

pub(crate) async fn list_models_with_params_and_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    params: Option<&ListModelsParams>,
) -> Result<Vec<Model>, OpenRouterError> {
    let url = format!("{base_url}/models");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response = match params {
        Some(params) if !params.is_empty() => req.query(params).send().await?,
        _ => req.send().await?,
    };

    if response.status().is_success() {
        let model_list_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "model list").await?;
        Ok(model_list_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Returns metadata about a specific model.
pub async fn get_model(
    base_url: &str,
    api_key: &str,
    author: &str,
    slug: &str,
) -> Result<Model, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_model_with_client(&http_client, base_url, api_key, author, slug).await
}

pub(crate) async fn get_model_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    author: &str,
    slug: &str,
) -> Result<Model, OpenRouterError> {
    let encoded_author = encode(author);
    let encoded_slug = encode(slug);
    let url = format!("{base_url}/model/{encoded_author}/{encoded_slug}");

    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let model_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "model").await?;
        Ok(model_response.data)
    } else {
        transport_response::handle_error(response).await?;
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
    let http_client = crate::transport::new_client()?;
    list_model_endpoints_with_client(&http_client, base_url, api_key, author, slug).await
}

pub(crate) async fn list_model_endpoints_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    author: &str,
    slug: &str,
) -> Result<EndpointData, OpenRouterError> {
    let encoded_author = encode(author);
    let encoded_slug = encode(slug);
    let url = format!("{base_url}/models/{encoded_author}/{encoded_slug}/endpoints");

    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let endpoint_list_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "model endpoint list").await?;
        Ok(endpoint_list_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
