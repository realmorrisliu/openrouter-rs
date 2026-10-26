use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::transport::RequestContext;
use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ApiResponse,
};

#[derive(Serialize, Deserialize, Debug, Clone, derive_builder::Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct GenerationFeedbackRequest {
    #[builder(setter(into))]
    pub generation_id: String,
    #[builder(setter(into))]
    pub category: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl GenerationFeedbackRequest {
    pub fn builder() -> GenerationFeedbackRequestBuilder {
        GenerationFeedbackRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GenerationFeedbackData {
    pub success: bool,
}

/// One provider attempt recorded for a generation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ProviderResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_byok: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_permaslug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routed_service_tier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct GenerationData {
    pub id: String,
    pub total_cost: f64,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_region: Option<String>,
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
    pub latency: Option<f64>,
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
    pub preset_id: Option<String>,
    pub response_cache_source_id: Option<String>,
    pub service_tier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_responses: Option<Vec<ProviderResponse>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_type: Option<String>,
}

pub(crate) async fn submit_generation_feedback_with_client(
    ctx: &RequestContext<'_>,
    request: &GenerationFeedbackRequest,
) -> Result<GenerationFeedbackData, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/generation/feedback");
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request);
    let payload: ApiResponse<GenerationFeedbackData> =
        transport_response::send_json(req, "generation feedback").await?;
    Ok(payload.data)
}

/// One failed upstream attempt recorded behind a stored generation error.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GenerationContentErrorAttempt {
    pub code: i64,
    pub message: String,
    pub provider_name: Option<String>,
    pub raw: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// The stored failure for a generation, or `None` when it succeeded.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GenerationContentError {
    /// HTTP status returned to the client.
    pub status: Option<i64>,
    /// Error message returned to the client.
    pub message: Option<String>,
    /// Provider whose error was returned to the client, when known.
    pub provider_name: Option<String>,
    /// Raw error body behind the returned error, when stored.
    pub raw: Option<String>,
    /// Every upstream attempt that failed before the returned error, in attempt order.
    #[serde(default)]
    pub previous_errors: Vec<GenerationContentErrorAttempt>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Stored prompt/input and completion/output content returned by `GET /generation/content`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GenerationContentData {
    #[serde(default)]
    pub input: HashMap<String, Value>,
    #[serde(default)]
    pub output: HashMap<String, Value>,
    /// The stored failure for this generation, when one was recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<GenerationContentError>,
}

pub(crate) async fn get_generation_with_client(
    ctx: &RequestContext<'_>,
    id: impl Into<String>,
) -> Result<GenerationData, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let id = id.into();
    let url = format!("{base_url}/generation?id={id}");

    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let generation_response: ApiResponse<_> =
        transport_response::send_json(req, "generation").await?;
    Ok(generation_response.data)
}

pub(crate) async fn get_generation_content_with_client(
    ctx: &RequestContext<'_>,
    id: impl Into<String>,
) -> Result<GenerationContentData, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let id = id.into();
    let url = format!("{base_url}/generation/content?id={id}");

    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let generation_response: ApiResponse<_> =
        transport_response::send_json(req, "generation content").await?;
    Ok(generation_response.data)
}
