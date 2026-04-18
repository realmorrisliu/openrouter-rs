use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ProviderPreferences,
};

/// Request payload for `POST /rerank`.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct RerankRequest {
    #[builder(setter(into))]
    pub model: String,
    #[builder(setter(into))]
    pub query: String,
    pub documents: Vec<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,
}

impl RerankRequest {
    pub fn builder() -> RerankRequestBuilder {
        RerankRequestBuilder::default()
    }
}

/// The original document returned in a rerank result.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RerankDocument {
    pub text: String,
}

/// One scored rerank result.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RerankResult {
    pub index: u64,
    pub relevance_score: f64,
    pub document: RerankDocument,
}

/// Usage statistics returned by rerank providers.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RerankUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_units: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
}

/// Response payload for `POST /rerank`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RerankResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub results: Vec<RerankResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<RerankUsage>,
}

/// Submit a rerank request.
pub async fn create_rerank(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    request: &RerankRequest,
) -> Result<RerankResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_rerank_with_client(
        &http_client,
        base_url,
        api_key,
        x_title,
        http_referer,
        request,
    )
    .await
}

pub(crate) async fn create_rerank_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    request: &RerankRequest,
) -> Result<RerankResponse, OpenRouterError> {
    let url = format!("{base_url}/rerank");
    let response = transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        api_key,
        x_title,
        http_referer,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "rerank").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
