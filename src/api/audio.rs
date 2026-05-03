use std::collections::HashMap;

use derive_builder::Builder;
use reqwest::{Client as HttpClient, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
};

const OFFICIAL_SPEECH_PATH: &str = "/audio/speech";
const TRANSCRIPTIONS_PATH: &str = "/audio/transcriptions";
const LEGACY_TTS_PATH: &str = "/tts";

/// Supported audio output formats for `POST /audio/speech`.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SpeechResponseFormat {
    Mp3,
    Pcm,
}

/// Provider-specific passthrough options for speech requests.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct SpeechProviderOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, serde_json::Value>>,
}

impl SpeechProviderOptions {
    pub fn new(options: HashMap<String, serde_json::Value>) -> Self {
        Self {
            options: Some(options),
        }
    }
}

/// Request payload for `POST /audio/speech`.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct SpeechRequest {
    #[builder(setter(into))]
    pub input: String,
    #[builder(setter(into))]
    pub model: String,
    #[builder(setter(into))]
    pub voice: String,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<SpeechProviderOptions>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<SpeechResponseFormat>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}

impl SpeechRequest {
    pub fn builder() -> SpeechRequestBuilder {
        SpeechRequestBuilder::default()
    }
}

/// Base64-encoded audio input for `POST /audio/transcriptions`.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct TranscriptionInputAudio {
    #[builder(setter(into))]
    pub data: String,
    #[builder(setter(into))]
    pub format: String,
}

impl TranscriptionInputAudio {
    pub fn builder() -> TranscriptionInputAudioBuilder {
        TranscriptionInputAudioBuilder::default()
    }

    pub fn new(data: impl Into<String>, format: impl Into<String>) -> Self {
        Self {
            data: data.into(),
            format: format.into(),
        }
    }
}

/// Provider-specific passthrough options for transcription requests.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct TranscriptionProviderOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, serde_json::Value>>,
}

impl TranscriptionProviderOptions {
    pub fn new(options: HashMap<String, serde_json::Value>) -> Self {
        Self {
            options: Some(options),
        }
    }
}

/// Request payload for `POST /audio/transcriptions`.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct TranscriptionRequest {
    #[builder(setter(into))]
    pub model: String,
    pub input_audio: TranscriptionInputAudio,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<TranscriptionProviderOptions>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
}

impl TranscriptionRequest {
    pub fn builder() -> TranscriptionRequestBuilder {
        TranscriptionRequestBuilder::default()
    }
}

/// Response payload from `POST /audio/transcriptions`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TranscriptionResponse {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TranscriptionUsage>,
}

/// Usage metadata for audio transcription requests.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct TranscriptionUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
}

/// Submit a speech request and return raw audio bytes.
pub async fn create_speech(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &SpeechRequest,
) -> Result<Vec<u8>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_speech_with_client(
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

pub(crate) async fn create_speech_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &SpeechRequest,
) -> Result<Vec<u8>, OpenRouterError> {
    let request_metadata = (x_title, http_referer, app_categories);
    let official_response = send_speech_request(
        http_client,
        base_url,
        api_key,
        request_metadata,
        request,
        OFFICIAL_SPEECH_PATH,
    )
    .await?;

    if official_response.status().is_success() {
        return Ok(official_response.bytes().await?.to_vec());
    }

    let official_error = transport_response::error_from_response(official_response).await;

    // Keep a narrow legacy fallback while upstream finishes the `/tts` -> `/audio/speech`
    // transition without masking request-level failures from the official route.
    if should_retry_legacy_tts(&official_error) {
        let legacy_response = send_speech_request(
            http_client,
            base_url,
            api_key,
            request_metadata,
            request,
            LEGACY_TTS_PATH,
        )
        .await?;

        if legacy_response.status().is_success() {
            return Ok(legacy_response.bytes().await?.to_vec());
        }

        transport_response::handle_error(legacy_response).await?;
    }

    Err(official_error)
}

/// Submit an audio transcription request.
pub async fn create_transcription(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &TranscriptionRequest,
) -> Result<TranscriptionResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_transcription_with_client(
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

pub(crate) async fn create_transcription_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &TranscriptionRequest,
) -> Result<TranscriptionResponse, OpenRouterError> {
    let url = format!("{base_url}{TRANSCRIPTIONS_PATH}");
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
        transport_response::parse_json_response(response, "audio transcription").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

async fn send_speech_request(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    request_metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    request: &SpeechRequest,
    endpoint_path: &str,
) -> Result<reqwest::Response, OpenRouterError> {
    let url = format!("{base_url}{endpoint_path}");
    Ok(transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        api_key,
        request_metadata.0,
        request_metadata.1,
        request_metadata.2,
    )?
    .json(request)
    .send()
    .await?)
}

fn should_retry_legacy_tts(error: &OpenRouterError) -> bool {
    let OpenRouterError::Api(api_error) = error else {
        return false;
    };

    let message = api_error.message.trim().to_ascii_lowercase();
    match api_error.status {
        StatusCode::NOT_FOUND => {
            is_generic_status_page(&message, "404", "not found")
                || is_path_specific_route_error(&message)
        }
        StatusCode::METHOD_NOT_ALLOWED => {
            is_generic_status_page(&message, "405", "method not allowed")
                || is_path_specific_route_error(&message)
        }
        _ => false,
    }
}

fn is_generic_status_page(message: &str, code: &str, reason_phrase: &str) -> bool {
    let message = message.trim_end_matches(['.', '!', '?', ';']);
    let bare_reason_phrase = matches!(message, "not found" | "method not allowed");
    let exact_status_page = message == format!("{code} page {reason_phrase}")
        || message == format!("{code} {reason_phrase}")
        || message == format!("http/1.1 {code} {reason_phrase}")
        || message == format!("http/2 {code} {reason_phrase}");

    bare_reason_phrase || exact_status_page
}

fn is_path_specific_route_error(message: &str) -> bool {
    let route_unavailable_signal = message.contains("cannot post")
        || message.contains("cannot get")
        || message.contains("route")
        || message.contains("endpoint")
        || message.contains("path")
        || message.contains("method not allowed");

    route_unavailable_signal && message.contains(OFFICIAL_SPEECH_PATH)
}
