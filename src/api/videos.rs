use std::collections::HashMap;

use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
};

/// One image URL payload used in video generation requests.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoImageUrl {
    pub url: String,
}

/// Reference image used to guide video generation.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoInputReference {
    #[serde(rename = "type")]
    pub content_type: String,
    pub image_url: VideoImageUrl,
}

impl VideoInputReference {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            content_type: "image_url".to_string(),
            image_url: VideoImageUrl { url: url.into() },
        }
    }
}

/// Frame image used as the first or last frame of a generated video.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoFrameImage {
    #[serde(rename = "type")]
    pub content_type: String,
    pub image_url: VideoImageUrl,
    pub frame_type: String,
}

impl VideoFrameImage {
    pub fn new(url: impl Into<String>, frame_type: impl Into<String>) -> Self {
        Self {
            content_type: "image_url".to_string(),
            image_url: VideoImageUrl { url: url.into() },
            frame_type: frame_type.into(),
        }
    }
}

/// Provider-specific passthrough options for video generation.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct VideoProviderOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, serde_json::Value>>,
}

/// Request payload for `POST /videos`.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct VideoGenerationRequest {
    #[builder(setter(into))]
    pub prompt: String,
    #[builder(setter(into))]
    pub model: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_images: Option<Vec<VideoFrameImage>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_references: Option<Vec<VideoInputReference>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<VideoProviderOptions>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

impl VideoGenerationRequest {
    pub fn builder() -> VideoGenerationRequestBuilder {
        VideoGenerationRequestBuilder::default()
    }
}

/// Usage payload returned by video generation status responses.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoGenerationUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_byok: Option<bool>,
}

/// Response payload returned by `POST /videos` and `GET /videos/{jobId}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoGenerationResponse {
    pub id: String,
    pub polling_url: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsigned_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<VideoGenerationUsage>,
}

/// Video model metadata returned by `GET /videos/models`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoModel {
    pub id: String,
    pub canonical_slug: String,
    pub name: String,
    pub created: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hugging_face_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_skus: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_resolutions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_aspect_ratios: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_sizes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_durations: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_frame_images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<bool>,
    #[serde(default)]
    pub allowed_passthrough_parameters: Vec<String>,
}

/// Submit a video generation request.
pub async fn create_video_generation(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &VideoGenerationRequest,
) -> Result<VideoGenerationResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_video_generation_with_client(
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

pub(crate) async fn create_video_generation_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &VideoGenerationRequest,
) -> Result<VideoGenerationResponse, OpenRouterError> {
    let url = format!("{base_url}/videos");
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
        transport_response::parse_json_response(response, "video generation").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// List all video generation models.
pub async fn list_video_models(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<VideoModel>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_video_models_with_client(&http_client, base_url, api_key).await
}

pub(crate) async fn list_video_models_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<VideoModel>, OpenRouterError> {
    let url = format!("{base_url}/videos/models");
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let payload: crate::types::ApiResponse<Vec<VideoModel>> =
            transport_response::parse_json_response(response, "video models").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Poll one video generation job by job id.
pub async fn get_video_generation(
    base_url: &str,
    api_key: &str,
    job_id: &str,
) -> Result<VideoGenerationResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_video_generation_with_client(&http_client, base_url, api_key, job_id).await
}

pub(crate) async fn get_video_generation_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    job_id: &str,
) -> Result<VideoGenerationResponse, OpenRouterError> {
    let url = format!("{base_url}/videos/{}", encode(job_id));
    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "video generation status").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Download binary content for a completed video generation job.
pub async fn get_video_content(
    base_url: &str,
    api_key: &str,
    job_id: &str,
    index: Option<u32>,
) -> Result<Vec<u8>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_video_content_with_client(&http_client, base_url, api_key, job_id, index).await
}

pub(crate) async fn get_video_content_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    job_id: &str,
    index: Option<u32>,
) -> Result<Vec<u8>, OpenRouterError> {
    let mut url = format!("{base_url}/videos/{}/content", encode(job_id));
    if let Some(index) = index {
        url = format!("{url}?index={index}");
    }

    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        Ok(response.bytes().await?.to_vec())
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
