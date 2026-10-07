//! Asynchronous batches for chat, Responses, Messages, and embeddings.
use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    transport::{request, response},
};

#[derive(Serialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateBatchRequest {
    #[builder(setter(into))]
    pub endpoint: String,
    #[builder(setter(into))]
    pub model: String,
    pub requests: Vec<BatchRequest>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_window: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<BatchProviderPreferences>,
}

impl CreateBatchRequest {
    pub fn builder() -> CreateBatchRequestBuilder {
        CreateBatchRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BatchRequest {
    pub custom_id: String,
    pub body: serde_json::Map<String, Value>,
}

impl BatchRequest {
    pub fn new(custom_id: impl Into<String>, body: serde_json::Map<String, Value>) -> Self {
        Self {
            custom_id: custom_id.into(),
            body,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct BatchProviderPreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ListBatchesParams {
    #[builder(setter(strip_option), default)]
    pub limit: Option<u32>,
    #[builder(setter(into, strip_option), default)]
    pub after: Option<String>,
    #[builder(setter(custom), default)]
    pub status: Option<Vec<String>>,
    #[builder(setter(into, strip_option), default)]
    pub created_after: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub created_before: Option<String>,
}

impl ListBatchesParams {
    pub fn builder() -> ListBatchesParamsBuilder {
        ListBatchesParamsBuilder::default()
    }
}

impl ListBatchesParamsBuilder {
    pub fn status<I, S>(&mut self, values: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.status = Some(Some(values.into_iter().map(Into::into).collect()));
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Batch {
    pub id: String,
    pub object: String,
    pub endpoint: String,
    pub model: String,
    pub completion_window: String,
    pub status: String,
    pub created_at: i64,
    pub finalized_at: Option<i64>,
    pub request_counts: BatchRequestCounts,
    pub usage: Option<Value>,
    pub results: Option<Vec<BatchResult>>,
    pub error: Option<BatchError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BatchRequestCounts {
    pub total: i64,
    pub completed: i64,
    pub failed: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BatchError {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BatchResult {
    pub id: String,
    pub custom_id: String,
    pub response: Option<BatchResultResponse>,
    pub error: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BatchResultResponse {
    pub status_code: i64,
    pub request_id: Option<String>,
    pub body: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ListBatchesResponse {
    pub object: String,
    pub data: Vec<Batch>,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
    pub has_more: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DeleteBatchResponse {
    pub id: String,
    pub object: String,
    pub deletion: BatchDeletionTargets,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BatchDeletionTargets {
    pub openrouter: String,
    pub upstream: Option<BatchUpstreamDeletion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BatchUpstreamDeletion {
    pub provider: String,
    pub status: String,
}

pub async fn list(
    base_url: &str,
    api_key: &str,
    params: &ListBatchesParams,
) -> Result<ListBatchesResponse, OpenRouterError> {
    list_with_client(
        &crate::transport::new_client()?,
        base_url,
        api_key,
        (&None, &None, &None),
        params,
    )
    .await
}

pub(crate) async fn list_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    params: &ListBatchesParams,
) -> Result<ListBatchesResponse, OpenRouterError> {
    let mut query = Vec::new();
    if let Some(limit) = params.limit {
        query.push(("limit", limit.to_string()));
    }
    for (key, value) in [
        ("after", &params.after),
        ("created_after", &params.created_after),
        ("created_before", &params.created_before),
    ] {
        if let Some(value) = value {
            query.push((key, value.clone()));
        }
    }
    if let Some(statuses) = &params.status {
        query.extend(statuses.iter().map(|status| ("status", status.clone())));
    }
    parse(
        request::with_client_request_headers(
            request::get(http_client, &format!("{base_url}/batches")),
            api_key,
            metadata.0,
            metadata.1,
            metadata.2,
        )?
        .query(&query)
        .send()
        .await?,
    )
    .await
}

pub async fn create(
    base_url: &str,
    api_key: &str,
    body: &CreateBatchRequest,
) -> Result<Batch, OpenRouterError> {
    create_with_client(
        &crate::transport::new_client()?,
        base_url,
        api_key,
        (&None, &None, &None),
        body,
    )
    .await
}

pub(crate) async fn create_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    body: &CreateBatchRequest,
) -> Result<Batch, OpenRouterError> {
    parse(
        request::with_client_request_headers(
            request::post(http_client, &format!("{base_url}/batches")),
            api_key,
            metadata.0,
            metadata.1,
            metadata.2,
        )?
        .json(body)
        .send()
        .await?,
    )
    .await
}

pub async fn get(base_url: &str, api_key: &str, batch_id: &str) -> Result<Batch, OpenRouterError> {
    get_with_client(
        &crate::transport::new_client()?,
        base_url,
        api_key,
        (&None, &None, &None),
        batch_id,
    )
    .await
}

pub(crate) async fn get_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    batch_id: &str,
) -> Result<Batch, OpenRouterError> {
    let url = format!("{base_url}/batches/{}", encode(batch_id));
    parse(
        request::with_client_request_headers(
            request::get(http_client, &url),
            api_key,
            metadata.0,
            metadata.1,
            metadata.2,
        )?
        .send()
        .await?,
    )
    .await
}

pub async fn delete(
    base_url: &str,
    api_key: &str,
    batch_id: &str,
) -> Result<DeleteBatchResponse, OpenRouterError> {
    delete_with_client(
        &crate::transport::new_client()?,
        base_url,
        api_key,
        (&None, &None, &None),
        batch_id,
    )
    .await
}

pub(crate) async fn delete_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    batch_id: &str,
) -> Result<DeleteBatchResponse, OpenRouterError> {
    let url = format!("{base_url}/batches/{}", encode(batch_id));
    parse(
        request::with_client_request_headers(
            request::delete(http_client, &url),
            api_key,
            metadata.0,
            metadata.1,
            metadata.2,
        )?
        .send()
        .await?,
    )
    .await
}

async fn parse<T: serde::de::DeserializeOwned>(
    res: reqwest::Response,
) -> Result<T, OpenRouterError> {
    if res.status().is_success() {
        response::parse_json_response(res, "batch").await
    } else {
        Err(response::error_from_response(res).await)
    }
}
