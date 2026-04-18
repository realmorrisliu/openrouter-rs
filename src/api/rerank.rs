use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    error::OpenRouterError,
    types::ProviderPreferences,
    utils::{handle_error, parse_json_response, with_client_request_headers},
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
    let url = format!("{base_url}/rerank");
    let response = with_client_request_headers(surf::post(url), api_key, x_title, http_referer)
        .body_json(request)?
        .await?;

    if response.status().is_success() {
        parse_json_response(response, "rerank").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
