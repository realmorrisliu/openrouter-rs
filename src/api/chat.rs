use std::collections::HashMap;

use futures_util::{AsyncBufReadExt, StreamExt, stream::BoxStream};
use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;

use crate::{
    error::OpenRouterError,
    setter,
    types::{ProviderPreferences, ReasoningConfig, Role},
    utils::handle_error,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    role: Role,
    content: String,
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
    models: Option<Vec<String>>,
    route: Option<String>,
    provider: Option<ProviderPreferences>,
    reasoning: Option<ReasoningConfig>,
}

impl ChatCompletionRequest {
    pub fn new(model: &str, messages: Vec<Message>) -> Self {
        Self {
            model: model.to_string(),
            messages,
            stream: None,
            max_tokens: None,
            temperature: None,
            seed: None,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            logit_bias: None,
            top_logprobs: None,
            min_p: None,
            top_a: None,
            transforms: None,
            models: None,
            route: None,
            provider: None,
            reasoning: None,
        }
    }

    fn stream(&self, stream: bool) -> Self {
        let mut req = self.clone();
        req.stream = Some(stream);
        req
    }

    setter!(max_tokens, u32);
    setter!(temperature, f64);
    setter!(seed, u32);
    setter!(top_p, f64);
    setter!(top_k, u32);
    setter!(frequency_penalty, f64);
    setter!(presence_penalty, f64);
    setter!(repetition_penalty, f64);
    setter!(logit_bias, HashMap<String, f64>);
    setter!(top_logprobs, u32);
    setter!(min_p, f64);
    setter!(top_a, f64);
    setter!(transforms, Vec<String>);
    setter!(models, Vec<String>);
    setter!(route, String);
    setter!(provider, ProviderPreferences);
    setter!(reasoning, ReasoningConfig);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatCompletionResponse {
    id: Option<String>,
    choices: Option<Vec<Choice>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    message: Option<Message>,
}

/// Send a chat completion request to a selected model.
///
/// # Arguments
///
/// * `base_url` - The base URL for the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `request` - The chat completion request containing the model and messages.
///
/// # Returns
///
/// * `Result<ChatCompletionResponse, OpenRouterError>` - The response from the chat completion request.
pub async fn send_chat_completion(
    base_url: &str,
    api_key: &str,
    request: &ChatCompletionRequest,
) -> Result<ChatCompletionResponse, OpenRouterError> {
    let url = format!("{}/chat/completions", base_url);

    // Ensure that the request is not streaming to get a single response
    let request = request.stream(false);

    let mut response = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .body_json(&request)?
        .await?;

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
