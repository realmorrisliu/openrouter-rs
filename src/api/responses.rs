use std::collections::HashMap;

use derive_builder::Builder;
use futures_util::{AsyncBufReadExt, StreamExt, stream::BoxStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surf::http::headers::AUTHORIZATION;

use crate::{
    api::chat::{Plugin, TraceOptions},
    error::OpenRouterError,
    strip_option_map_setter, strip_option_vec_setter,
    types::ProviderPreferences,
    utils::handle_error,
};

/// Request body for the OpenRouter Responses API (`POST /responses`).
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct ResponsesRequest {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<Value>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Value>>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<Value>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    models: Option<Vec<String>>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<Value>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<Value>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_logprobs: Option<u32>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tool_calls: Option<u32>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<f64>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    image_config: Option<HashMap<String, Value>>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    modalities: Option<Vec<String>>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_cache_key: Option<String>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_response_id: Option<String>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt: Option<Value>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    include: Option<Vec<String>>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<bool>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_identifier: Option<String>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    store: Option<bool>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    service_tier: Option<String>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    truncation: Option<String>,

    #[builder(setter(skip), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<ProviderPreferences>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    plugins: Option<Vec<Plugin>>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    route: Option<String>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<TraceOptions>,
}

impl ResponsesRequestBuilder {
    strip_option_map_setter!(metadata, String, String);
    strip_option_vec_setter!(tools, Value);
    strip_option_vec_setter!(models, String);
    strip_option_map_setter!(image_config, String, Value);
    strip_option_vec_setter!(modalities, String);
    strip_option_vec_setter!(include, String);
    strip_option_vec_setter!(plugins, Plugin);
}

impl ResponsesRequest {
    pub fn builder() -> ResponsesRequestBuilder {
        ResponsesRequestBuilder::default()
    }

    pub fn new(model: impl Into<String>, input: Value) -> Self {
        Self::builder()
            .model(model.into())
            .input(input)
            .build()
            .expect("Failed to build ResponsesRequest")
    }

    fn stream(&self, stream: bool) -> Self {
        let mut req = self.clone();
        req.stream = Some(stream);
        req
    }
}

/// Non-streaming response payload returned by `POST /responses`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponsesResponse {
    pub id: Option<String>,
    #[serde(rename = "object")]
    pub object_type: Option<String>,
    pub created_at: Option<u64>,
    pub model: Option<String>,
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Streaming event payload returned by `POST /responses` when `stream=true`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponsesStreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence_number: Option<u64>,
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// Send a non-streaming request to the Responses API.
pub async fn create_response(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    request: &ResponsesRequest,
) -> Result<ResponsesResponse, OpenRouterError> {
    let url = format!("{base_url}/responses");
    let request = request.stream(false);

    let mut surf_req = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .body_json(&request)?;

    if let Some(x_title) = x_title {
        surf_req = surf_req.header("X-Title", x_title);
    }
    if let Some(http_referer) = http_referer {
        surf_req = surf_req.header("HTTP-Referer", http_referer);
    }

    let mut response = surf_req.await?;

    if response.status().is_success() {
        let response_data: ResponsesResponse = response.body_json().await?;
        Ok(response_data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Send a streaming request to the Responses API.
pub async fn stream_response(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    request: &ResponsesRequest,
) -> Result<BoxStream<'static, Result<ResponsesStreamEvent, OpenRouterError>>, OpenRouterError> {
    let url = format!("{base_url}/responses");
    let request = request.stream(true);

    let mut surf_req = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .body_json(&request)?;

    if let Some(x_title) = x_title {
        surf_req = surf_req.header("X-Title", x_title);
    }
    if let Some(http_referer) = http_referer {
        surf_req = surf_req.header("HTTP-Referer", http_referer);
    }

    let response = surf_req.await?;

    if response.status().is_success() {
        let lines = response
            .lines()
            .filter_map(async |line| match line {
                Ok(line) => line
                    .strip_prefix("data: ")
                    .filter(|line| *line != "[DONE]")
                    .map(serde_json::from_str::<ResponsesStreamEvent>)
                    .map(|event| event.map_err(OpenRouterError::Serialization)),
                Err(error) => Some(Err(OpenRouterError::Io(error))),
            })
            .boxed();

        Ok(lines)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
