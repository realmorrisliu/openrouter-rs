use std::collections::HashMap;

use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::{
    api::models::{ModelReasoning, PricingOverride},
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<Vec<PricingOverride>>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ModelReasoning>,
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
    /// Whether the endpoint supports stateless voice cloning for speech requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_voice_cloning: Option<bool>,
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
    /// Workspace attribution, present only when `group_by=workspace` is requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Query parameters for `GET /activity`.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ActivityParams {
    /// Filter by a single UTC date in the last 30 days (`YYYY-MM-DD`).
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Filter by API key hash (SHA-256 hex string, as returned by the keys API).
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
    /// Filter by org member user ID (organization accounts only).
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Set to `workspace` to split each row per workspace.
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    /// Filter by workspace ID (UUID).
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

impl ActivityParams {
    pub fn builder() -> ActivityParamsBuilder {
        ActivityParamsBuilder::default()
    }

    fn is_empty(&self) -> bool {
        self.date.is_none()
            && self.api_key_hash.is_none()
            && self.user_id.is_none()
            && self.group_by.is_none()
            && self.workspace_id.is_none()
    }
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

/// Query parameters for `GET /datasets/rankings-daily`.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct RankingsDailyParams {
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_bucket: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_type: Option<String>,
}

impl RankingsDailyParams {
    pub fn builder() -> RankingsDailyParamsBuilder {
        RankingsDailyParamsBuilder::default()
    }
}

/// One aggregated cost-per-session cell returned by `GET /datasets/session-cost`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SessionCostItem {
    /// Stable public slug of the harness.
    pub app_slug: String,
    /// Published harness display label.
    pub app_name: String,
    /// Inclusive session turn-count range (e.g. `10-49-turns`).
    pub turn_range: String,
    /// Exact model permaslug.
    pub model_permaslug: String,
    /// Median USD spend per sampled session.
    pub median_session_cost_usd: f64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Metadata for a session-cost dataset response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SessionCostMeta {
    /// ISO-8601 timestamp when the response was generated.
    pub as_of: String,
    /// Dataset version.
    pub version: String,
    /// Number of days in the weekly session sample window, if published.
    pub window_days: Option<u64>,
    /// UTC date of the final day in the session sample window, if published.
    pub window_end_date: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Aggregated cost-per-session cells returned by `GET /datasets/session-cost`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SessionCostResponse {
    pub data: Vec<SessionCostItem>,
    pub meta: SessionCostMeta,
}

/// Query parameters for `GET /datasets/session-cost`.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct SessionCostParams {
    /// Filter to one published harness slug.
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_slug: Option<String>,
    /// Exact model permaslug filter. Works across all harness apps.
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Filter by the inclusive number of turns in a session
    /// (`1-turn`, `2-9-turns`, `10-49-turns`, `50-plus-turns`).
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_range: Option<String>,
    /// Maximum number of cells to return (1-500). Defaults to 100.
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Number of sorted cells to skip (0-5000). Defaults to 0.
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

impl SessionCostParams {
    pub fn builder() -> SessionCostParamsBuilder {
        SessionCostParamsBuilder::default()
    }

    fn is_empty(&self) -> bool {
        self.app_slug.is_none()
            && self.model.is_none()
            && self.turn_range.is_none()
            && self.limit.is_none()
            && self.offset.is_none()
    }
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

/// Top model share for one task classification.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TaskClassificationModel {
    pub id: String,
    pub tag_usage_share: f64,
    pub tag_token_share: f64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// One task classification row returned by `GET /classifications/task`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TaskClassificationItem {
    pub tag: String,
    pub display_name: String,
    pub macro_category: String,
    pub usage_share: f64,
    pub token_share: f64,
    pub category_usage_share: f64,
    pub category_token_share: f64,
    pub models: Vec<TaskClassificationModel>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Aggregate market-share data for one task macro-category.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TaskClassificationMacroCategory {
    pub key: String,
    pub label: String,
    pub usage_share: f64,
    pub token_share: f64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Data payload returned by `GET /classifications/task`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TaskClassificationsData {
    pub window_days: u64,
    pub as_of: String,
    pub classifications: Vec<TaskClassificationItem>,
    pub macro_categories: Vec<TaskClassificationMacroCategory>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Task classification response returned by `GET /classifications/task`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TaskClassificationsResponse {
    pub data: TaskClassificationsData,
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

/// Query parameters for the unified benchmarks endpoint.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UnifiedBenchmarksParams {
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    /// Return results for one exact OpenRouter benchmark (e.g. `gpqa_diamond`,
    /// `search_widesearch`).
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benchmark_type: Option<String>,
    /// Search benchmarks only: include the published lane configuration whitelist.
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_run_config: Option<bool>,
    /// OpenRouter search benchmarks only: filter by the search engine used.
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_engine: Option<String>,
    /// OpenRouter search benchmarks only: filter by request surface (`server-tool`
    /// or `plugin`).
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_surface: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arena: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
}

impl UnifiedBenchmarksParams {
    pub fn builder() -> UnifiedBenchmarksParamsBuilder {
        UnifiedBenchmarksParamsBuilder::default()
    }

    pub fn artificial_analysis() -> Self {
        Self {
            source: Some("artificial-analysis".to_string()),
            ..Default::default()
        }
    }

    pub fn design_arena() -> Self {
        Self {
            source: Some("design-arena".to_string()),
            ..Default::default()
        }
    }

    pub fn openrouter() -> Self {
        Self {
            source: Some("openrouter".to_string()),
            ..Default::default()
        }
    }
}

/// One Artificial Analysis row returned by `GET /benchmarks`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UnifiedBenchmarksAAItem {
    pub source: String,
    pub model_permaslug: String,
    pub display_name: String,
    pub intelligence_index: Option<f64>,
    pub coding_index: Option<f64>,
    pub agentic_index: Option<f64>,
    pub pricing: Option<BenchmarkPricing>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// One Design Arena row returned by `GET /benchmarks`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UnifiedBenchmarksDAItem {
    pub source: String,
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

/// One OpenRouter evaluation row returned by `GET /benchmarks`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UnifiedBenchmarksORItem {
    pub source: String,
    pub model_permaslug: String,
    pub display_name: String,
    pub benchmark_type: String,
    pub accuracy: f64,
    pub accuracy_stddev: Option<f64>,
    pub avg_cost_per_task: Option<f64>,
    pub total_tasks: u64,
    pub last_run_timestamp: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// One benchmark row returned by `GET /benchmarks`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum UnifiedBenchmarkItem {
    DesignArena(UnifiedBenchmarksDAItem),
    OpenRouter(UnifiedBenchmarksORItem),
    ArtificialAnalysis(UnifiedBenchmarksAAItem),
    Other(HashMap<String, serde_json::Value>),
}

/// Metadata for the unified benchmarks endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UnifiedBenchmarksMeta {
    pub as_of: String,
    pub version: String,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub citation: Option<String>,
    pub model_count: u64,
    pub task_type: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Unified benchmark response returned by `GET /benchmarks`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UnifiedBenchmarksResponse {
    pub data: Vec<UnifiedBenchmarkItem>,
    pub meta: UnifiedBenchmarksMeta,
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
    let params = RankingsDailyParams {
        start_date: start_date.map(str::to_owned),
        end_date: end_date.map(str::to_owned),
        ..Default::default()
    };
    get_rankings_daily_with_params_and_client(http_client, base_url, api_key, Some(&params)).await
}

pub async fn get_rankings_daily_with_params(
    base_url: &str,
    api_key: &str,
    params: Option<&RankingsDailyParams>,
) -> Result<RankingsDailyResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_rankings_daily_with_params_and_client(&http_client, base_url, api_key, params).await
}

pub(crate) async fn get_rankings_daily_with_params_and_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    params: Option<&RankingsDailyParams>,
) -> Result<RankingsDailyResponse, OpenRouterError> {
    let url = format!("{base_url}/datasets/rankings-daily");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response = match params {
        Some(params) => req.query(params).send().await?,
        None => req.send().await?,
    };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "rankings daily").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Return aggregated cost-per-session cells (`GET /datasets/session-cost`).
///
/// Weekly refreshed, privacy-preserving medians of per-session USD spend for the
/// published harnesses. Licensed under CC BY 4.0 by OpenRouter.
pub async fn get_session_cost(
    base_url: &str,
    api_key: &str,
    params: Option<&SessionCostParams>,
) -> Result<SessionCostResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_session_cost_with_client(&http_client, base_url, api_key, params).await
}

pub(crate) async fn get_session_cost_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    params: Option<&SessionCostParams>,
) -> Result<SessionCostResponse, OpenRouterError> {
    let url = format!("{base_url}/datasets/session-cost");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response = match params {
        Some(params) if !params.is_empty() => req.query(params).send().await?,
        _ => req.send().await?,
    };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "session cost").await
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

/// Return task classification market-share data (`GET /classifications/task`).
pub async fn get_task_classifications(
    base_url: &str,
    api_key: &str,
    window: Option<&str>,
) -> Result<TaskClassificationsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_task_classifications_with_client(&http_client, base_url, api_key, window).await
}

pub(crate) async fn get_task_classifications_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    window: Option<&str>,
) -> Result<TaskClassificationsResponse, OpenRouterError> {
    let url = format!("{base_url}/classifications/task");
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let response = match window {
        Some(window) => req.query(&[("window", window)]).send().await?,
        None => req.send().await?,
    };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "task classifications").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Return benchmark rows from a selected benchmark source (`GET /benchmarks`).
pub async fn get_benchmarks(
    base_url: &str,
    api_key: &str,
    params: &UnifiedBenchmarksParams,
) -> Result<UnifiedBenchmarksResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_benchmarks_with_client(&http_client, base_url, api_key, params).await
}

pub(crate) async fn get_benchmarks_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    params: &UnifiedBenchmarksParams,
) -> Result<UnifiedBenchmarksResponse, OpenRouterError> {
    let url = format!("{base_url}/benchmarks");
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .query(params)
            .send()
            .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "benchmarks").await
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
#[deprecated(note = "use get_benchmarks with source `artificial-analysis`")]
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
#[deprecated(note = "use get_benchmarks with source `design-arena`")]
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
    let params = ActivityParams {
        date: date.map(str::to_owned),
        ..Default::default()
    };
    get_activity_with_params_and_client(http_client, base_url, management_key, Some(&params)).await
}

/// Get endpoint-grouped activity with the full filter surface (`GET /activity`).
pub async fn get_activity_with_params(
    base_url: &str,
    management_key: &str,
    params: Option<&ActivityParams>,
) -> Result<Vec<ActivityItem>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_activity_with_params_and_client(&http_client, base_url, management_key, params).await
}

pub(crate) async fn get_activity_with_params_and_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    params: Option<&ActivityParams>,
) -> Result<Vec<ActivityItem>, OpenRouterError> {
    let url = format!("{base_url}/activity");
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    let response = match params {
        Some(params) if !params.is_empty() => req.query(params).send().await?,
        _ => req.send().await?,
    };

    if response.status().is_success() {
        let parsed: ApiResponse<Vec<ActivityItem>> =
            transport_response::parse_json_response(response, "activity list").await?;
        Ok(parsed.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
