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

    /// Ordered list of provider names
    /// Router will attempt to use the first available provider in this list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,

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
}
