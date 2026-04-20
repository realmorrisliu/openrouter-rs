use std::collections::HashMap;

use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
};

/// Supported audio output formats for `POST /tts`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TtsResponseFormat {
    Mp3,
    Pcm,
}

/// Provider-specific passthrough options for text-to-speech requests.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TtsProviderOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, serde_json::Value>>,
}

/// Request payload for `POST /tts`.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct TtsRequest {
    #[builder(setter(into))]
    pub input: String,
    #[builder(setter(into))]
    pub model: String,
    #[builder(setter(into))]
    pub voice: String,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<TtsProviderOptions>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<TtsResponseFormat>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}

impl TtsRequest {
    pub fn builder() -> TtsRequestBuilder {
        TtsRequestBuilder::default()
    }
}

/// Submit a text-to-speech request and return raw audio bytes.
pub async fn create_tts(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &TtsRequest,
) -> Result<Vec<u8>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_tts_with_client(
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

pub(crate) async fn create_tts_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &TtsRequest,
) -> Result<Vec<u8>, OpenRouterError> {
    let url = format!("{base_url}/tts");
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
        Ok(response.bytes().await?.to_vec())
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
