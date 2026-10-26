use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

use crate::transport::RequestContext;
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias_target: Option<ModelAliasTarget>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Concrete model targeted by a model alias.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ModelAliasTarget {
    pub slug: String,
    pub name: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<Vec<PricingOverride>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Conditional override applied to base model pricing.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PricingOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utc_days: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_cache: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cache_read: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cache_write: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cache_write_1h: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_prompt_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utc_end: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utc_start: Option<f64>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_tool_choice: Option<ToolChoiceSupport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf_last_30m_by_workload: Option<HashMap<String, Value>>,
    pub name: String,
    pub context_length: f64,
    pub pricing: EndpointPricing,
    pub provider_name: String,
    pub supported_parameters: Vec<String>,
    pub quantization: Option<String>,
    pub max_completion_tokens: Option<f64>,
    pub max_prompt_tokens: Option<f64>,
    pub status: Option<serde_json::Value>,
    /// Whether the endpoint supports stateless voice cloning for speech requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_voice_cloning: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_image_reference: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_multiple_audio_references: Option<bool>,
    /// Service tier this endpoint sells, or `None` for a standard endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    /// Whether this STT endpoint can label transcript segments by speaker.
    #[serde(default)]
    pub supports_diarization: bool,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<Vec<PricingOverride>>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_modalities: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modalities: Option<Vec<String>>,
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
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_output_price: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_price: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_age_days: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age_days: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_intelligence_index: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_intelligence_index: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_coding_index: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_coding_index: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_agentic_index: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_agentic_index: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_tool_success_rate: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_success_rate: Option<f64>,
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
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
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
            && self.min_output_price.is_none()
            && self.max_output_price.is_none()
            && self.min_age_days.is_none()
            && self.max_age_days.is_none()
            && self.min_intelligence_index.is_none()
            && self.max_intelligence_index.is_none()
            && self.min_coding_index.is_none()
            && self.max_coding_index.is_none()
            && self.min_agentic_index.is_none()
            && self.max_agentic_index.is_none()
            && self.min_tool_success_rate.is_none()
            && self.max_tool_success_rate.is_none()
            && self.arch.is_none()
            && self.model_authors.is_none()
            && self.providers.is_none()
            && self.distillable.is_none()
            && self.zdr.is_none()
            && self.region.is_none()
            && self.limit.is_none()
            && self.offset.is_none()
    }
}

pub(crate) async fn list_models_with_client(
    ctx: &RequestContext<'_>,
    category: Option<ModelCategory>,
    supported_parameters: Option<SupportedParameters>,
) -> Result<Vec<Model>, OpenRouterError> {
    let params = ListModelsParams {
        category,
        supported_parameters,
        ..Default::default()
    };
    list_models_with_params_and_client(ctx, Some(&params)).await
}

pub(crate) async fn list_models_with_params_and_client(
    ctx: &RequestContext<'_>,
    params: Option<&ListModelsParams>,
) -> Result<Vec<Model>, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/models");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let req = match params {
        Some(params) if !params.is_empty() => req.query(params),
        _ => req,
    };
    let model_list_response: ApiResponse<_> =
        transport_response::send_json(req, "model list").await?;
    Ok(model_list_response.data)
}

pub(crate) async fn get_model_with_client(
    ctx: &RequestContext<'_>,
    author: &str,
    slug: &str,
) -> Result<Model, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let encoded_author = encode(author);
    let encoded_slug = encode(slug);
    let url = format!("{base_url}/model/{encoded_author}/{encoded_slug}");

    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let model_response: ApiResponse<_> = transport_response::send_json(req, "model").await?;
    Ok(model_response.data)
}

pub(crate) async fn list_model_endpoints_with_client(
    ctx: &RequestContext<'_>,
    author: &str,
    slug: &str,
) -> Result<EndpointData, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let encoded_author = encode(author);
    let encoded_slug = encode(slug);
    let url = format!("{base_url}/models/{encoded_author}/{encoded_slug}/endpoints");

    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let endpoint_list_response: ApiResponse<_> =
        transport_response::send_json(req, "model endpoint list").await?;
    Ok(endpoint_list_response.data)
}

/// Tool choice modes supported by a model endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolChoiceSupport {
    pub auto: bool,
    pub none: bool,
    pub required: bool,
    pub function: bool,
}
