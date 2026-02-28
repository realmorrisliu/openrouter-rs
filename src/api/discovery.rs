use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;
use urlencoding::encode;

use crate::{error::OpenRouterError, types::ApiResponse, utils::handle_error};

/// Number-like value used by OpenRouter pricing fields.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum BigNumber {
    String(String),
    Number(f64),
}

/// Public provider metadata returned by `GET /providers`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Provider {
    pub name: String,
    pub slug: String,
    pub privacy_policy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_service_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_page_url: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Model pricing payload returned by model discovery endpoints.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicPricing {
    pub prompt: BigNumber,
    pub completion: BigNumber,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_token: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_output: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_output: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_cache: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_reasoning: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cache_read: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cache_write: Option<BigNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount: Option<f64>,
}

/// Model architecture data in model discovery responses.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelArchitecture {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruct_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_modalities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_modalities: Option<Vec<String>>,
}

/// Top provider metadata in model discovery responses.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopProviderInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<f64>,
    pub is_moderated: bool,
}

/// Per-request token limits for a model.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerRequestLimits {
    pub prompt_tokens: f64,
    pub completion_tokens: f64,
}

/// Model payload returned by `GET /models/user`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserModel {
    pub id: String,
    pub canonical_slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hugging_face_id: Option<String>,
    pub name: String,
    pub created: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub pricing: PublicPricing,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<f64>,
    pub architecture: ModelArchitecture,
    pub top_provider: TopProviderInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_request_limits: Option<PerRequestLimits>,
    #[serde(default)]
    pub supported_parameters: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Count payload returned by `GET /models/count`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelsCountData {
    pub count: u64,
}

/// Percentile statistics payload used by endpoint throughput/latency.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PercentileStats {
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p99: f64,
}

/// Public endpoint payload returned by `GET /endpoints/zdr`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicEndpoint {
    pub name: String,
    pub model_id: String,
    pub model_name: String,
    pub context_length: f64,
    pub pricing: PublicPricing,
    pub provider_name: String,
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<f64>,
    #[serde(default)]
    pub supported_parameters: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_last_30m: Option<f64>,
    pub supports_implicit_caching: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_last_30m: Option<PercentileStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throughput_last_30m: Option<PercentileStats>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Activity item payload returned by `GET /activity`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActivityItem {
    pub date: String,
    pub model: String,
    pub model_permaslug: String,
    pub endpoint_id: String,
    pub provider_name: String,
    pub usage: f64,
    pub byok_usage_inference: f64,
    pub requests: f64,
    pub prompt_tokens: f64,
    pub completion_tokens: f64,
    pub reasoning_tokens: f64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// List all providers (`GET /providers`).
pub async fn list_providers(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<Provider>, OpenRouterError> {
    let url = format!("{base_url}/providers");
    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<Vec<Provider>> = response.body_json().await?;
        Ok(parsed.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// List models filtered by user settings (`GET /models/user`).
pub async fn list_models_for_user(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<UserModel>, OpenRouterError> {
    let url = format!("{base_url}/models/user");
    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<Vec<UserModel>> = response.body_json().await?;
        Ok(parsed.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Count available models (`GET /models/count`).
pub async fn count_models(
    base_url: &str,
    api_key: &str,
) -> Result<ModelsCountData, OpenRouterError> {
    let url = format!("{base_url}/models/count");
    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<ModelsCountData> = response.body_json().await?;
        Ok(parsed.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// List ZDR-compatible endpoints (`GET /endpoints/zdr`).
pub async fn list_zdr_endpoints(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<PublicEndpoint>, OpenRouterError> {
    let url = format!("{base_url}/endpoints/zdr");
    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<Vec<PublicEndpoint>> = response.body_json().await?;
        Ok(parsed.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Get endpoint-grouped activity (`GET /activity`).
///
/// `date` is optional and should be in `YYYY-MM-DD` format.
pub async fn get_activity(
    base_url: &str,
    management_key: &str,
    date: Option<&str>,
) -> Result<Vec<ActivityItem>, OpenRouterError> {
    let url = if let Some(date) = date {
        format!("{base_url}/activity?date={}", encode(date))
    } else {
        format!("{base_url}/activity")
    };

    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {management_key}"))
        .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<Vec<ActivityItem>> = response.body_json().await?;
        Ok(parsed.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
