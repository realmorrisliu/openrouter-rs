use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::{
    api::models,
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, ProviderPreferences},
};

/// Supported embedding encoding formats.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingEncodingFormat {
    Float,
    Base64,
}

/// Image URL content for multimodal embedding input.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmbeddingImageUrl {
    pub url: String,
}

/// One multimodal content part for embedding input.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EmbeddingContentPart {
    Text { text: String },
    ImageUrl { image_url: EmbeddingImageUrl },
}

/// One multimodal embedding input item.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmbeddingMultimodalInput {
    pub content: Vec<EmbeddingContentPart>,
}

/// Embedding request input variants.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum EmbeddingInput {
    Text(String),
    TextArray(Vec<String>),
    TokenArray(Vec<f64>),
    TokenArrayBatch(Vec<Vec<f64>>),
    MultimodalArray(Vec<EmbeddingMultimodalInput>),
}

impl From<String> for EmbeddingInput {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for EmbeddingInput {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(value: Vec<String>) -> Self {
        Self::TextArray(value)
    }
}

impl From<Vec<f64>> for EmbeddingInput {
    fn from(value: Vec<f64>) -> Self {
        Self::TokenArray(value)
    }
}

impl From<Vec<Vec<f64>>> for EmbeddingInput {
    fn from(value: Vec<Vec<f64>>) -> Self {
        Self::TokenArrayBatch(value)
    }
}

impl From<Vec<EmbeddingMultimodalInput>> for EmbeddingInput {
    fn from(value: Vec<EmbeddingMultimodalInput>) -> Self {
        Self::MultimodalArray(value)
    }
}

/// Request body for `POST /embeddings`.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct EmbeddingRequest {
    #[builder(setter(into))]
    pub input: EmbeddingInput,

    #[builder(setter(into))]
    pub model: String,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<EmbeddingEncodingFormat>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
}

impl EmbeddingRequest {
    pub fn builder() -> EmbeddingRequestBuilder {
        EmbeddingRequestBuilder::default()
    }

    pub fn new(model: impl Into<String>, input: impl Into<EmbeddingInput>) -> Self {
        Self::builder()
            .model(model.into())
            .input(input.into())
            .build()
            .expect("Failed to build EmbeddingRequest")
    }
}

/// One embedding vector payload.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum EmbeddingVector {
    Float(Vec<f64>),
    Base64(String),
}

/// One embedding item.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: EmbeddingVector,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
}

/// Token breakdown details for embedding requests.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmbeddingPromptTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_tokens: Option<u32>,
}

/// Token/cost usage for embedding request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<EmbeddingPromptTokensDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
}

/// Response body for `POST /embeddings`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmbeddingResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<EmbeddingUsage>,
}

/// Submit an embedding request.
pub async fn create_embedding(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    request: &EmbeddingRequest,
) -> Result<EmbeddingResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_embedding_with_client(
        &http_client,
        base_url,
        api_key,
        x_title,
        http_referer,
        request,
    )
    .await
}

pub(crate) async fn create_embedding_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    request: &EmbeddingRequest,
) -> Result<EmbeddingResponse, OpenRouterError> {
    let url = format!("{base_url}/embeddings");

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
        let embedding_response: EmbeddingResponse =
            transport_response::parse_json_response(response, "embedding").await?;
        Ok(embedding_response)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// List all embedding models.
pub async fn list_embedding_models(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<models::Model>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_embedding_models_with_client(&http_client, base_url, api_key).await
}

pub(crate) async fn list_embedding_models_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<models::Model>, OpenRouterError> {
    let url = format!("{base_url}/embeddings/models");

    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let models_response: ApiResponse<Vec<models::Model>> =
            transport_response::parse_json_response(response, "embedding models").await?;
        Ok(models_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
