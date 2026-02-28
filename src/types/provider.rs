use serde::{Deserialize, Serialize};

/// Sorting strategy for provider selection when no order is specified
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProviderSortBy {
    /// Sort by price (cheapest first)
    Price,
    /// Sort by throughput (highest first)
    Throughput,
    /// Sort by latency (lowest first)
    Latency,
}

/// Data collection policy preference
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DataCollectionPolicy {
    /// Allow providers which may collect user data (default)
    Allow,
    /// Only allow providers that don't collect user data
    Deny,
}

/// Model quantization levels
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Quantization {
    Int4,
    Int8,
    Fp4,
    Fp6,
    Fp8,
    Fp16,
    Bf16,
    Fp32,
    Unknown,
}

/// Numeric threshold value or percentile cutoffs.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PerformancePreference {
    Value(f64),
    Percentiles(PercentileCutoffs),
}

/// Percentile-based throughput/latency cutoffs.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct PercentileCutoffs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p50: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p75: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p90: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p99: Option<f64>,
}

/// Price limit represented as either number or string.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PriceLimit {
    Number(f64),
    String(String),
}

impl From<f64> for PriceLimit {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for PriceLimit {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for PriceLimit {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

/// Maximum price constraints for provider routing.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct MaxPrice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<PriceLimit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<PriceLimit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<PriceLimit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<PriceLimit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<PriceLimit>,
}

/// Configuration for how OpenRouter routes requests to AI providers
///
/// See detailed documentation at: <https://openrouter.ai/docs/features/provider-routing>
///
/// This struct allows you to control:
/// - Provider fallback behavior
/// - Data collection policies
/// - Provider selection order
/// - Provider blacklisting
/// - Quantization preferences
/// - Routing optimization strategies
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ProviderPreferences {
    /// Whether to allow backup providers to serve requests
    /// - true: (default) when the primary provider is unavailable, use the next best provider
    /// - false: use only the primary/custom provider, and return error if unavailable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,

    /// Whether to filter providers to only those that support the provided parameters
    /// If false or omitted, providers will receive only the parameters they support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,

    /// Data collection setting
    /// If no available provider meets the requirement, request will fail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<DataCollectionPolicy>,

    /// Restrict routing to only ZDR endpoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zdr: Option<bool>,

    /// Restrict routing to models that permit text distillation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_distillable_text: Option<bool>,

    /// Ordered list of provider names
    /// Router will attempt to use the first available provider in this list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,

    /// List of provider names to allow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<String>>,

    /// List of provider names to ignore
    /// Merged with account-wide ignored provider settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,

    /// Quantization levels to filter providers by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantizations: Option<Vec<Quantization>>,

    /// Sorting strategy when no order is specified
    /// When set, no load balancing is performed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<ProviderSortBy>,

    /// Maximum price constraints (USD per million tokens / per request).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<MaxPrice>,

    /// Preferred minimum throughput (tokens/sec) for routing preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_min_throughput: Option<PerformancePreference>,

    /// Preferred maximum latency (seconds) for routing preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_max_latency: Option<PerformancePreference>,
}
