use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ProviderPreferences,
};

/// One document input accepted by rerank requests.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RerankDocumentInput {
    Text(String),
    Multimodal {
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<String>,
    },
}

impl RerankDocumentInput {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn multimodal<T, U>(text: Option<T>, image: Option<U>) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        Self::Multimodal {
            text: text.map(Into::into),
            image: image.map(Into::into),
        }
    }
}

impl From<String> for RerankDocumentInput {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for RerankDocumentInput {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

/// Request payload for `POST /rerank`.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct RerankRequest {
    #[builder(setter(into))]
    pub model: String,
    #[builder(setter(into))]
    pub query: String,
    #[builder(setter(custom))]
    pub documents: Vec<RerankDocumentInput>,
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

impl RerankRequestBuilder {
    pub fn documents<T, S>(&mut self, items: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<RerankDocumentInput>,
    {
        self.documents = Some(items.into_iter().map(Into::into).collect());
        self
    }
}

/// The original document returned in a rerank result.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RerankDocument {
    pub text: String,
}

/// One scored rerank result.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RerankResult {
    pub index: u64,
    pub relevance_score: f64,
    pub document: RerankDocument,
}

/// Usage statistics returned by rerank providers.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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
#[non_exhaustive]
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
    app_categories: &Option<Vec<String>>,
    request: &RerankRequest,
) -> Result<RerankResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_rerank_with_client(
        &http_client,
        base_url,
        api_key,
        x_title,
        http_referer,
        app_categories,
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
    app_categories: &Option<Vec<String>>,
    request: &RerankRequest,
) -> Result<RerankResponse, OpenRouterError> {
    let url = format!("{base_url}/rerank");
    let response = transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?
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
