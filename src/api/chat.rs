use crate::{
    error::OpenRouterError,
    setter,
    types::{ProviderPreferences, ReasoningConfig, Role},
    utils::handle_error,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

    setter!(stream, bool);
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
/// * `client` - The HTTP client to use for the request.
/// * `api_key` - The API key for authentication.
/// * `request` - The chat completion request containing the model and messages.
///
/// # Returns
///
/// * `Result<ChatCompletionResponse, OpenRouterError>` - The response from the chat completion request.
pub async fn send_chat_completion(
    client: &Client,
    api_key: &str,
    request: &ChatCompletionRequest,
) -> Result<ChatCompletionResponse, OpenRouterError> {
    let url = "https://openrouter.ai/api/v1/chat/completions";

    let response = client
        .post(url)
        .bearer_auth(api_key)
        .json(request)
        .send()
        .await?;

    if response.status().is_success() {
        let chat_response = response.json::<ChatCompletionResponse>().await?;
        Ok(chat_response)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
