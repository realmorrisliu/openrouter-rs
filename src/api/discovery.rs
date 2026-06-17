use std::collections::HashMap;

use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ApiResponse,
};

/// Number-like value used by OpenRouter pricing fields.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
#[serde(untagged)]
pub enum BigNumber {
    String(String),
    Number(f64),
}

/// Public provider metadata returned by `GET /providers`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub struct TopProviderInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<f64>,
    pub is_moderated: bool,
}

/// Per-request token limits for a model.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PerRequestLimits {
    pub prompt_tokens: f64,
    pub completion_tokens: f64,
}

/// Model payload returned by `GET /models/user`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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
    pub supported_voices: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Count payload returned by `GET /models/count`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ModelsCountData {
    pub count: u64,
}

/// Percentile statistics payload used by endpoint throughput/latency.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PercentileStats {
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p99: f64,
}

/// Public endpoint payload returned by `GET /endpoints/zdr`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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
#[non_exhaustive]
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

/// One daily model-ranking row returned by `GET /datasets/rankings-daily`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RankingsDailyItem {
    pub date: String,
    pub model_permaslug: String,
    pub total_tokens: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Metadata for a daily rankings dataset response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RankingsDailyMeta {
    pub as_of: String,
    pub version: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Daily token totals for top public models plus an aggregated `other` row.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RankingsDailyResponse {
    pub data: Vec<RankingsDailyItem>,
    pub meta: RankingsDailyMeta,
}

/// Query parameters for `GET /datasets/app-rankings`.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct AppRankingsParams {
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcategory: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

impl AppRankingsParams {
    pub fn builder() -> AppRankingsParamsBuilder {
        AppRankingsParamsBuilder::default()
    }

    fn is_empty(&self) -> bool {
        self.category.is_none()
            && self.subcategory.is_none()
            && self.sort.is_none()
            && self.start_date.is_none()
            && self.end_date.is_none()
            && self.limit.is_none()
            && self.offset.is_none()
    }
}

/// One application ranking row returned by `GET /datasets/app-rankings`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AppRankingsItem {
    pub rank: u64,
    pub app_id: u64,
    pub app_name: String,
    pub total_tokens: String,
    pub total_requests: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// App rankings dataset response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AppRankingsResponse {
    pub data: Vec<AppRankingsItem>,
    pub meta: RankingsDailyMeta,
}

/// OpenRouter benchmark pricing payload.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BenchmarkPricing {
    pub prompt: String,
    pub completion: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// One Artificial Analysis benchmark row.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BenchmarksAAItem {
    pub model_permaslug: String,
    pub aa_name: String,
    pub intelligence_index: Option<f64>,
    pub coding_index: Option<f64>,
    pub agentic_index: Option<f64>,
    pub pricing: Option<BenchmarkPricing>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Metadata for Artificial Analysis benchmark rows.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BenchmarksAAMeta {
    pub as_of: String,
    pub version: String,
    pub source: String,
    pub source_url: String,
    pub citation: String,
    pub model_count: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Artificial Analysis benchmark dataset response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BenchmarksAAResponse {
    pub data: Vec<BenchmarksAAItem>,
    pub meta: BenchmarksAAMeta,
}

/// Placement distribution from Design Arena tournament matches.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DesignArenaTournamentStats {
    pub first_place: Option<u64>,
    pub second_place: Option<u64>,
    pub third_place: Option<u64>,
    pub fourth_place: Option<u64>,
    pub total: Option<u64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// One Design Arena benchmark row.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BenchmarksDAItem {
    pub model_permaslug: String,
    pub display_name: String,
    pub arena: String,
    pub category: String,
    pub elo: f64,
    pub win_rate: f64,
    pub avg_generation_time_ms: Option<f64>,
    pub tournament_stats: DesignArenaTournamentStats,
    pub pricing: Option<BenchmarkPricing>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// ELO bounds for a Design Arena response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DesignArenaEloBounds {
    pub min: f64,
    pub max: f64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Metadata for Design Arena benchmark rows.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BenchmarksDAMeta {
    pub as_of: String,
    pub version: String,
    pub source: String,
    pub source_url: String,
    pub citation: String,
    pub model_count: u64,
    pub arena: String,
    pub category: Option<String>,
    pub elo_bounds: DesignArenaEloBounds,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Design Arena benchmark dataset response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BenchmarksDAResponse {
    pub data: Vec<BenchmarksDAItem>,
    pub meta: BenchmarksDAMeta,
}

/// List all providers (`GET /providers`).
pub async fn list_providers(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<Provider>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_providers_with_client(&http_client, base_url, api_key).await
}

pub(crate) async fn list_providers_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<Provider>, OpenRouterError> {
    let url = format!("{base_url}/providers");
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<Vec<Provider>> =
            transport_response::parse_json_response(response, "provider list").await?;
        Ok(parsed.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// List models filtered by user settings (`GET /models/user`).
pub async fn list_models_for_user(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<UserModel>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_models_for_user_with_client(&http_client, base_url, api_key).await
}

pub(crate) async fn list_models_for_user_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<UserModel>, OpenRouterError> {
    let url = format!("{base_url}/models/user");
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<Vec<UserModel>> =
            transport_response::parse_json_response(response, "user model list").await?;
        Ok(parsed.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Count available models (`GET /models/count`).
pub async fn count_models(
    base_url: &str,
    api_key: &str,
) -> Result<ModelsCountData, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    count_models_with_client(&http_client, base_url, api_key).await
}

pub(crate) async fn count_models_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
) -> Result<ModelsCountData, OpenRouterError> {
    let url = format!("{base_url}/models/count");
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<ModelsCountData> =
            transport_response::parse_json_response(response, "model count").await?;
        Ok(parsed.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Return daily token totals for top public models (`GET /datasets/rankings-daily`).
pub async fn get_rankings_daily(
    base_url: &str,
    api_key: &str,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<RankingsDailyResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_rankings_daily_with_client(&http_client, base_url, api_key, start_date, end_date).await
}

pub(crate) async fn get_rankings_daily_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<RankingsDailyResponse, OpenRouterError> {
    #[derive(Serialize)]
    struct RankingsDailyQuery<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        start_date: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_date: Option<&'a str>,
    }

    let url = format!("{base_url}/datasets/rankings-daily");
    let query = RankingsDailyQuery {
        start_date,
        end_date,
    };
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response = if query.start_date.is_none() && query.end_date.is_none() {
        req.send().await?
    } else {
        req.query(&query).send().await?
    };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "rankings daily").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Return app rankings over a date window (`GET /datasets/app-rankings`).
pub async fn get_app_rankings(
    base_url: &str,
    api_key: &str,
    params: Option<&AppRankingsParams>,
) -> Result<AppRankingsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_app_rankings_with_client(&http_client, base_url, api_key, params).await
}

pub(crate) async fn get_app_rankings_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    params: Option<&AppRankingsParams>,
) -> Result<AppRankingsResponse, OpenRouterError> {
    let url = format!("{base_url}/datasets/app-rankings");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response = match params {
        Some(params) if !params.is_empty() => req.query(params).send().await?,
        _ => req.send().await?,
    };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "app rankings").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

#[derive(Serialize)]
struct BenchmarkMaxResultsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<u32>,
}

/// Return Artificial Analysis benchmark rows.
pub async fn get_benchmarks_artificial_analysis(
    base_url: &str,
    api_key: &str,
    max_results: Option<u32>,
) -> Result<BenchmarksAAResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_benchmarks_artificial_analysis_with_client(&http_client, base_url, api_key, max_results)
        .await
}

pub(crate) async fn get_benchmarks_artificial_analysis_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    max_results: Option<u32>,
) -> Result<BenchmarksAAResponse, OpenRouterError> {
    let url = format!("{base_url}/datasets/benchmarks/artificial-analysis");
    let query = BenchmarkMaxResultsQuery { max_results };
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response = if query.max_results.is_none() {
        req.send().await?
    } else {
        req.query(&query).send().await?
    };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "Artificial Analysis benchmarks").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

#[derive(Serialize)]
struct DesignArenaQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    arena: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<u32>,
}

/// Return Design Arena benchmark rows.
pub async fn get_benchmarks_design_arena(
    base_url: &str,
    api_key: &str,
    arena: Option<&str>,
    category: Option<&str>,
    max_results: Option<u32>,
) -> Result<BenchmarksDAResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_benchmarks_design_arena_with_client(
        &http_client,
        base_url,
        api_key,
        arena,
        category,
        max_results,
    )
    .await
}

pub(crate) async fn get_benchmarks_design_arena_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    arena: Option<&str>,
    category: Option<&str>,
    max_results: Option<u32>,
) -> Result<BenchmarksDAResponse, OpenRouterError> {
    let url = format!("{base_url}/datasets/benchmarks/design-arena");
    let query = DesignArenaQuery {
        arena,
        category,
        max_results,
    };
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response =
        if query.arena.is_none() && query.category.is_none() && query.max_results.is_none() {
            req.send().await?
        } else {
            req.query(&query).send().await?
        };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "Design Arena benchmarks").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// List ZDR-compatible endpoints (`GET /endpoints/zdr`).
pub async fn list_zdr_endpoints(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<PublicEndpoint>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_zdr_endpoints_with_client(&http_client, base_url, api_key).await
}

pub(crate) async fn list_zdr_endpoints_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<PublicEndpoint>, OpenRouterError> {
    let url = format!("{base_url}/endpoints/zdr");
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<Vec<PublicEndpoint>> =
            transport_response::parse_json_response(response, "ZDR endpoint list").await?;
        Ok(parsed.data)
    } else {
        transport_response::handle_error(response).await?;
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
    let http_client = crate::transport::new_client()?;
    get_activity_with_client(&http_client, base_url, management_key, date).await
}

pub(crate) async fn get_activity_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    date: Option<&str>,
) -> Result<Vec<ActivityItem>, OpenRouterError> {
    let url = if let Some(date) = date {
        format!("{base_url}/activity?date={}", encode(date))
    } else {
        format!("{base_url}/activity")
    };

    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let parsed: ApiResponse<Vec<ActivityItem>> =
            transport_response::parse_json_response(response, "activity list").await?;
        Ok(parsed.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
