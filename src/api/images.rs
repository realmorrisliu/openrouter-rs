use std::collections::HashMap;

use derive_builder::Builder;
use futures_util::{
    StreamExt,
    stream::{self, BoxStream},
};
use reqwest::{Client as HttpClient, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    strip_option_vec_setter,
    transport::{
        request as transport_request, response as transport_response, sse::response_lines,
    },
    utils::parse_sse_frames,
};

/// One image URL payload used as an image generation reference.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageUrl {
    pub url: String,
}

impl ImageUrl {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

/// Reference image used to guide image generation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageInputReference {
    #[serde(rename = "type")]
    pub content_type: String,
    pub image_url: ImageUrl,
}

impl ImageInputReference {
    pub fn new(url: impl Into<String>) -> Self {
        Self::image_url(url)
    }

    pub fn image_url(url: impl Into<String>) -> Self {
        Self {
            content_type: "image_url".to_string(),
            image_url: ImageUrl::new(url),
        }
    }
}

/// Provider-specific passthrough options for image generation.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ImageProviderOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, Value>>,
}

impl ImageProviderOptions {
    pub fn new(options: HashMap<String, Value>) -> Self {
        Self {
            options: Some(options),
        }
    }
}

/// Request payload for `POST /images`.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ImageGenerationRequest {
    #[builder(setter(into))]
    pub model: String,
    #[builder(setter(into))]
    pub prompt: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_references: Option<Vec<ImageInputReference>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<u32>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ImageProviderOptions>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[builder(setter(skip), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

impl ImageGenerationRequestBuilder {
    strip_option_vec_setter!(input_references, ImageInputReference);
}

impl ImageGenerationRequest {
    pub fn builder() -> ImageGenerationRequestBuilder {
        ImageGenerationRequestBuilder::default()
    }

    fn stream(&self, stream: bool) -> Self {
        let mut req = self.clone();
        req.stream = Some(stream);
        req
    }
}

/// One generated image returned by `POST /images`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GeneratedImage {
    pub b64_json: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Token and cost usage returned by image generation responses.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageGenerationUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_byok: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Non-streaming response returned by `POST /images`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageGenerationResponse {
    pub created: u64,
    pub data: Vec<GeneratedImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ImageGenerationUsage>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Descriptor for one supported image-generation request parameter.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageCapabilityDescriptor {
    #[serde(rename = "type")]
    pub capability_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Architecture metadata returned by image model discovery endpoints.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageModelArchitecture {
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Image model metadata returned by `GET /images/models`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created: u64,
    pub architecture: ImageModelArchitecture,
    pub supported_parameters: HashMap<String, ImageCapabilityDescriptor>,
    pub supports_streaming: bool,
    pub endpoints: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// One billable pricing line for an image provider.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImagePricingEntry {
    pub billable: String,
    pub unit: String,
    pub cost_usd: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Endpoint metadata for one image generation provider.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageEndpoint {
    pub provider_name: String,
    pub provider_slug: String,
    pub provider_tag: Option<String>,
    pub supported_parameters: HashMap<String, ImageCapabilityDescriptor>,
    #[serde(default)]
    pub allowed_passthrough_parameters: Vec<String>,
    pub supports_streaming: bool,
    pub pricing: Vec<ImagePricingEntry>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Response returned by `GET /images/models/{author}/{slug}/endpoints`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageModelEndpointsResponse {
    pub id: String,
    pub endpoints: Vec<ImageEndpoint>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Partial-image event emitted by streaming image generation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImagePartialImageEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub partial_image_index: u32,
    pub b64_json: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Completion event emitted by streaming image generation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageCompletedEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub b64_json: String,
    pub created: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ImageGenerationUsage>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Error details emitted by streaming image generation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageStreamError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Error event emitted by streaming image generation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageStreamErrorEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub error: ImageStreamError,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Streaming image generation event payload.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ImageStreamEvent {
    PartialImage(ImagePartialImageEvent),
    Completed(ImageCompletedEvent),
    Error(ImageStreamErrorEvent),
    Other(Value),
}

/// SSE data wrapper returned by `POST /images` when `stream=true`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageStreamingResponse {
    pub data: ImageStreamEvent,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Submit an image generation request.
pub async fn create_image_generation(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &ImageGenerationRequest,
) -> Result<ImageGenerationResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_image_generation_with_client(
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

pub(crate) async fn create_image_generation_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &ImageGenerationRequest,
) -> Result<ImageGenerationResponse, OpenRouterError> {
    let url = format!("{base_url}/images");
    let request = request.stream(false);
    let response = transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(&request)
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "image generation").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Stream image generation events.
pub async fn stream_image_generation(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &ImageGenerationRequest,
) -> Result<BoxStream<'static, Result<ImageStreamingResponse, OpenRouterError>>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    stream_image_generation_with_client(
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

pub(crate) async fn stream_image_generation_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &ImageGenerationRequest,
) -> Result<BoxStream<'static, Result<ImageStreamingResponse, OpenRouterError>>, OpenRouterError> {
    let url = format!("{base_url}/images");
    let request = request.stream(true);
    let response = transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(&request)
    .send()
    .await?;

    if response.status().is_success() {
        if is_sse_response(&response) {
            let lines = parse_sse_frames(response_lines(response))
                .filter_map(async |line| match line {
                    Ok(frame) if frame.data == "[DONE]" => None,
                    Ok(frame) => Some(
                        serde_json::from_str::<ImageStreamingResponse>(&frame.data)
                            .map_err(OpenRouterError::Serialization),
                    ),
                    Err(error) => Some(Err(error)),
                })
                .boxed();

            Ok(lines)
        } else {
            let response: ImageGenerationResponse =
                transport_response::parse_json_response(response, "image generation").await?;
            Ok(buffered_image_response_stream(response))
        }
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

fn is_sse_response(response: &reqwest::Response) -> bool {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .eq_ignore_ascii_case("text/event-stream")
        })
        .unwrap_or(false)
}

fn buffered_image_response_stream(
    response: ImageGenerationResponse,
) -> BoxStream<'static, Result<ImageStreamingResponse, OpenRouterError>> {
    let created = response.created;
    let data = response.data;
    let usage = response.usage;
    let response_extra = response.extra;

    stream::iter(data.into_iter().map(move |image| {
        Ok(ImageStreamingResponse {
            data: ImageStreamEvent::Completed(ImageCompletedEvent {
                event_type: "image_generation.completed".to_string(),
                b64_json: image.b64_json,
                created,
                media_type: image.media_type,
                usage: usage.clone(),
                extra: image.extra,
            }),
            extra: response_extra.clone(),
        })
    }))
    .boxed()
}

/// List all image generation models.
pub async fn list_image_models(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ImageModel>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_image_models_with_client(&http_client, base_url, api_key).await
}

pub(crate) async fn list_image_models_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ImageModel>, OpenRouterError> {
    let url = format!("{base_url}/images/models");
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let payload: crate::types::ApiResponse<Vec<ImageModel>> =
            transport_response::parse_json_response(response, "image models").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// List provider endpoints for one image generation model.
pub async fn list_image_model_endpoints(
    base_url: &str,
    api_key: &str,
    author: &str,
    slug: &str,
) -> Result<ImageModelEndpointsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_image_model_endpoints_with_client(&http_client, base_url, api_key, author, slug).await
}

pub(crate) async fn list_image_model_endpoints_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    author: &str,
    slug: &str,
) -> Result<ImageModelEndpointsResponse, OpenRouterError> {
    let encoded_author = encode(author);
    let encoded_slug = encode(slug);
    let url = format!("{base_url}/images/models/{encoded_author}/{encoded_slug}/endpoints");
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "image model endpoints").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
