use std::collections::HashMap;

use futures_util::{AsyncBufReadExt, StreamExt, stream::BoxStream};
use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;

use crate::{
    error::OpenRouterError,
    setter,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatCompletionRequest {
    model: String,
    models: Option<Vec<String>>,
    messages: Vec<Message>,
    stream: Option<bool>,
    max_tokens: Option<u32>,
    temperature: Option<f64>,
    seed: Option<u32>,
    top_p: Option<f64>,
    top_k: Option<u32>,
    frequency_penalty: Option<f64>,
    presence_penalty: Option<f64>,
    repetition_penalty: Option<f64>,
    logit_bias: Option<HashMap<String, f64>>,
    top_logprobs: Option<u32>,
    min_p: Option<f64>,
    top_a: Option<f64>,
    transforms: Option<Vec<String>>,
    route: Option<String>,
    provider: Option<ProviderPreferences>,
    response_format: Option<ResponseFormat>,
    reasoning: Option<ReasoningConfig>,
}

#[derive(Default)]
pub struct ChatCompletionRequestBuilder {
    model: Option<String>,
    models: Option<Vec<String>>,
    messages: Option<Vec<Message>>,
    stream: Option<bool>,
    max_tokens: Option<u32>,
    temperature: Option<f64>,
    seed: Option<u32>,
    top_p: Option<f64>,
    top_k: Option<u32>,
    frequency_penalty: Option<f64>,
    presence_penalty: Option<f64>,
    repetition_penalty: Option<f64>,
    logit_bias: Option<HashMap<String, f64>>,
    top_logprobs: Option<u32>,
    min_p: Option<f64>,
    top_a: Option<f64>,
    transforms: Option<Vec<String>>,
    route: Option<String>,
    provider: Option<ProviderPreferences>,
    response_format: Option<ResponseFormat>,
    reasoning: Option<ReasoningConfig>,
}

impl ChatCompletionRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    setter!(model, into String);
    setter!(models, Vec<String>);
    setter!(messages, Vec<Message>);
    setter!(stream, bool);
    setter!(max_tokens, u32);
    setter!(temperature, f64);
    setter!(seed, u32);
    setter!(top_p, f64);
    setter!(top_k, u32);
    setter!(frequency_penalty, f64);
    setter!(presence_penalty, f64);
    setter!(repetition_penalty, f64);
    setter!(top_logprobs, u32);
    setter!(min_p, f64);
    setter!(top_a, f64);
    setter!(logit_bias, HashMap<String, f64>);
    setter!(transforms, Vec<String>);
    setter!(route, String);
    setter!(provider, ProviderPreferences);
    setter!(response_format, ResponseFormat);
    setter!(reasoning, ReasoningConfig);

    pub fn build(self) -> Result<ChatCompletionRequest, OpenRouterError> {
        Ok(ChatCompletionRequest {
            model: self
                .model
                .ok_or(OpenRouterError::Validation("model is required".into()))?,
            models: self.models,
            messages: self
                .messages
                .ok_or(OpenRouterError::Validation("messages are required".into()))?,
            stream: self.stream,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            seed: self.seed,
            top_p: self.top_p,
            top_k: self.top_k,
            frequency_penalty: self.frequency_penalty,
            presence_penalty: self.presence_penalty,
            repetition_penalty: self.repetition_penalty,
            logit_bias: self.logit_bias,
            top_logprobs: self.top_logprobs,
            min_p: self.min_p,
            top_a: self.top_a,
            transforms: self.transforms,
            route: self.route,
            provider: self.provider,
            response_format: self.response_format,
            reasoning: self.reasoning,
        })
    }
}

impl ChatCompletionRequest {
    pub fn builder() -> ChatCompletionRequestBuilder {
        ChatCompletionRequestBuilder::new()
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
