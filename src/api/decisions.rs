use std::collections::HashMap;

use derive_builder::Builder;
use reqwest::{Client as HttpClient, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    api::chat::TraceOptions,
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ProviderPreferences,
};

#[derive(Serialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct DecisionsRequest {
    #[builder(setter(into))]
    pub model: String,
    #[builder(setter(into))]
    pub state: Value,
    #[builder(setter(custom))]
    pub questions: HashMap<String, Value>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<TraceOptions>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl DecisionsRequest {
    pub fn builder() -> DecisionsRequestBuilder {
        DecisionsRequestBuilder::default()
    }
}

impl DecisionsRequestBuilder {
    pub fn questions<I, K, V>(&mut self, questions: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        self.questions = Some(
            questions
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        );
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DecisionsUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DecisionsResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub answers: HashMap<String, Value>,
    pub usage: DecisionsUsage,
}

pub async fn create(
    base_url: &str,
    api_key: &str,
    request: &DecisionsRequest,
) -> Result<DecisionsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_with_client(
        &http_client,
        base_url,
        api_key,
        &None,
        &None,
        &None,
        request,
    )
    .await
}

pub(crate) async fn create_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &DecisionsRequest,
) -> Result<DecisionsResponse, OpenRouterError> {
    let mut url = Url::parse(base_url)
        .map_err(|error| OpenRouterError::ConfigError(format!("invalid base URL: {error}")))?;
    url.set_path("/api/alpha/decisions");
    url.set_query(None);
    url.set_fragment(None);
    let request_builder = transport_request::with_client_request_headers(
        transport_request::post(http_client, url.as_str()),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?;
    let response = request_builder.json(request).send().await?;
    parse_result(response, "decisions").await
}

pub async fn create_system_one(
    base_url: &str,
    api_key: &str,
    request: &DecisionsRequest,
) -> Result<DecisionsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_system_one_with_client(
        &http_client,
        base_url,
        api_key,
        &None,
        &None,
        &None,
        request,
    )
    .await
}

pub(crate) async fn create_system_one_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &DecisionsRequest,
) -> Result<DecisionsResponse, OpenRouterError> {
    let request_builder = transport_request::with_client_request_headers(
        transport_request::post(http_client, &format!("{base_url}/systemone")),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?;
    let response = request_builder.json(request).send().await?;
    parse_result(response, "system one").await
}

async fn parse_result<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
    context: &str,
) -> Result<T, OpenRouterError> {
    if response.status().is_success() {
        transport_response::parse_json_response(response, context).await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
