use crate::{
    error::OpenRouterError,
    setter,
    types::{ProviderPreferences, ReasoningConfig},
    utils::handle_error,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct CompletionRequest {
    model: String,
    prompt: String,
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

impl CompletionRequest {
    pub fn new(model: &str, prompt: &str) -> Self {
        Self {
            model: model.to_string(),
            prompt: prompt.to_string(),
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
pub struct CompletionResponse {
    id: Option<String>,
    choices: Option<Vec<Choice>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    text: Option<String>,
    index: Option<u32>,
    finish_reason: Option<String>,
}

/// Send a completion request to a selected model (text-only format)
///
/// # Arguments
///
/// * `client` - The HTTP client to use for the request.
/// * `api_key` - The API key for authentication.
/// * `request` - The completion request containing the model, prompt, and other optional parameters.
///
/// # Returns
///
/// * `Result<CompletionResponse, OpenRouterError>` - The response from the completion request, containing the generated text and other details.
pub async fn send_completion_request(
    client: &Client,
    api_key: &str,
    request: &CompletionRequest,
) -> Result<CompletionResponse, OpenRouterError> {
    let url = "https://openrouter.ai/api/v1/completions";

    let response = client
        .post(url)
        .bearer_auth(api_key)
        .json(request)
        .send()
        .await?;

    if response.status().is_success() {
        let completion_response = response.json::<CompletionResponse>().await?;
        Ok(completion_response)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
