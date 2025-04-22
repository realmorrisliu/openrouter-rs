use std::collections::HashMap;

use derive_builder::Builder;
use futures_util::{AsyncBufReadExt, StreamExt, stream::BoxStream};
use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;

use crate::{
    error::OpenRouterError,
    strip_option_map_setter, strip_option_vec_setter,
    types::{
        ProviderPreferences, ReasoningConfig, ResponseFormat, Role, completion::CompletionsResponse,
    },
    utils::handle_error,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn new(role: Role, content: &str) -> Self {
        Self {
            role,
            content: content.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct ChatCompletionRequest {
    #[builder(setter(into))]
    model: String,

    messages: Vec<Message>,

    #[builder(setter(skip), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u32>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    repetition_penalty: Option<f64>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    logit_bias: Option<HashMap<String, f64>>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_logprobs: Option<u32>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    min_p: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_a: Option<f64>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    transforms: Option<Vec<String>>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    models: Option<Vec<String>>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    route: Option<String>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<ProviderPreferences>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<ReasoningConfig>,
}

impl ChatCompletionRequestBuilder {
    strip_option_vec_setter!(models, String);
    strip_option_map_setter!(logit_bias, String, f64);
    strip_option_vec_setter!(transforms, String);
}

impl ChatCompletionRequest {
    pub fn builder() -> ChatCompletionRequestBuilder {
        ChatCompletionRequestBuilder::default()
    }

    pub fn new(model: &str, messages: Vec<Message>) -> Self {
        Self::builder()
            .model(model)
            .messages(messages)
            .build()
            .expect("Failed to build ChatCompletionRequest")
    }

    fn stream(&self, stream: bool) -> Self {
        let mut req = self.clone();
        req.stream = Some(stream);
        req
    }
}

/// Send a chat completion request to a selected model.
///
/// # Arguments
///
/// * `base_url` - The base URL for the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `x_title` - The name of the site for the request.
/// * `http_referer` - The URL of the site for the request.
/// * `request` - The chat completion request containing the model and messages.
///
/// # Returns
///
/// * `Result<CompletionsResponse, OpenRouterError>` - The response from the chat completion request.
pub async fn send_chat_completion(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    request: &ChatCompletionRequest,
) -> Result<CompletionsResponse, OpenRouterError> {
    let url = format!("{}/chat/completions", base_url);

    // Ensure that the request is not streaming to get a single response
    let request = request.stream(false);

    let mut surf_req = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .body_json(&request)?;

    if let Some(x_title) = x_title {
        surf_req = surf_req.header("X-Title", x_title);
    }
    if let Some(http_referer) = http_referer {
        surf_req = surf_req.header("HTTP-Referer", http_referer);
    }

    let mut response = surf_req.await?;

    if response.status().is_success() {
        let chat_response = response.body_json().await?;
        Ok(chat_response)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatCompletionStreamEvent {
    id: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    choices: Option<Vec<DeltaChoice>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeltaChoice {
    delta: Option<Message>,
}

/// Stream chat completion events from a selected model.
///
/// # Arguments
///
/// * `base_url` - The base URL for the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `request` - The chat completion request containing the model and messages.
///
/// # Returns
///
/// * `Result<BoxStream<'static, Result<ChatCompletionStreamEvent, OpenRouterError>>, OpenRouterError>` - A stream of chat completion events or an error.
pub async fn stream_chat_completion(
    base_url: &str,
    api_key: &str,
    request: &ChatCompletionRequest,
) -> Result<BoxStream<'static, Result<ChatCompletionStreamEvent, OpenRouterError>>, OpenRouterError>
{
    let url = format!("{}/chat/completions", base_url);

    // Ensure that the request is streaming to get a continuous response
    let request = request.stream(true);

    let response = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .body_json(&request)?
        .await?;

    if response.status().is_success() {
        let lines = response
            .lines()
            .filter_map(async |line| match line {
                Ok(line) => line
                    .strip_prefix("data: ")
                    .filter(|line| *line != "[DONE]")
                    .map(|line| serde_json::from_str::<ChatCompletionStreamEvent>(line))
                    .map(|event| event.map_err(|err| OpenRouterError::Serialization(err))),
                Err(error) => Some(Err(OpenRouterError::Io(error))),
            })
            .boxed();

        Ok(lines)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
