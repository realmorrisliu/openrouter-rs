use std::collections::HashMap;

use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::OpenRouterError,
    strip_option_vec_setter,
    transport::{request as transport_request, response as transport_response},
    types::ApiResponse,
};

/// One analytics metric definition returned by `GET /analytics/meta`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AnalyticsMetric {
    pub name: String,
    pub display_label: String,
    pub is_rate: bool,
    pub display_format: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// One analytics dimension definition returned by `GET /analytics/meta`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AnalyticsDimension {
    pub name: String,
    pub display_label: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// One analytics filter operator definition returned by `GET /analytics/meta`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AnalyticsOperator {
    pub name: String,
    pub value_type: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// One analytics granularity definition returned by `GET /analytics/meta`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AnalyticsGranularity {
    pub name: String,
    pub display_label: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Analytics query metadata.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AnalyticsMeta {
    pub metrics: Vec<AnalyticsMetric>,
    pub dimensions: Vec<AnalyticsDimension>,
    pub operators: Vec<AnalyticsOperator>,
    pub granularities: Vec<AnalyticsGranularity>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Scalar value used inside analytics filters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum AnalyticsFilterScalar {
    String(String),
    Number(f64),
}

impl From<&str> for AnalyticsFilterScalar {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for AnalyticsFilterScalar {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<f64> for AnalyticsFilterScalar {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

/// Filter value accepted by analytics query requests.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum AnalyticsFilterValue {
    String(String),
    Number(f64),
    Array(Vec<AnalyticsFilterScalar>),
}

/// One analytics filter.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalyticsFilter {
    pub field: String,
    pub operator: String,
    pub value: AnalyticsFilterValue,
}

/// Analytics time range.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalyticsTimeRange {
    pub start: String,
    pub end: String,
}

/// Analytics ordering clause.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalyticsOrderBy {
    pub field: String,
    pub direction: String,
}

/// Request payload for `POST /analytics/query`.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct AnalyticsQueryRequest {
    #[builder(setter(custom))]
    pub metrics: Vec<String>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<AnalyticsFilter>>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_limit: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<AnalyticsOrderBy>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<AnalyticsTimeRange>,
}

impl AnalyticsQueryRequest {
    pub fn builder() -> AnalyticsQueryRequestBuilder {
        AnalyticsQueryRequestBuilder::default()
    }
}

impl AnalyticsQueryRequestBuilder {
    pub fn metrics<T, S>(&mut self, items: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.metrics = Some(items.into_iter().map(Into::into).collect());
        self
    }

    strip_option_vec_setter!(dimensions, String);
    strip_option_vec_setter!(filters, AnalyticsFilter);
}

/// Metadata returned with analytics query rows.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AnalyticsQueryMetadata {
    pub query_time_ms: f64,
    pub row_count: u64,
    pub truncated: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Analytics query result payload.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AnalyticsQueryResponse {
    #[serde(default, rename = "cachedAt")]
    pub cached_at: Option<f64>,
    pub data: Vec<HashMap<String, Value>>,
    pub metadata: AnalyticsQueryMetadata,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Get analytics query metadata (`GET /analytics/meta`).
pub async fn get_analytics_meta(
    base_url: &str,
    management_key: &str,
) -> Result<AnalyticsMeta, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_analytics_meta_with_client(&http_client, base_url, management_key).await
}

pub(crate) async fn get_analytics_meta_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
) -> Result<AnalyticsMeta, OpenRouterError> {
    let url = format!("{base_url}/analytics/meta");
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<AnalyticsMeta> =
            transport_response::parse_json_response(response, "analytics meta").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Query analytics (`POST /analytics/query`).
pub async fn query_analytics(
    base_url: &str,
    management_key: &str,
    request: &AnalyticsQueryRequest,
) -> Result<AnalyticsQueryResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    query_analytics_with_client(&http_client, base_url, management_key, request).await
}

pub(crate) async fn query_analytics_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    request: &AnalyticsQueryRequest,
) -> Result<AnalyticsQueryResponse, OpenRouterError> {
    let url = format!("{base_url}/analytics/query");
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<AnalyticsQueryResponse> =
            transport_response::parse_json_response(response, "analytics query").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
