use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;

use crate::{
    error::OpenRouterError,
    setter,
    types::{
        ProviderPreferences, ReasoningConfig, ResponseFormat, completion::CompletionsResponse,
    },
    utils::handle_error,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct CompletionRequest {
    model: String,
    models: Option<Vec<String>>,
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
    route: Option<String>,
    provider: Option<ProviderPreferences>,
    response_format: Option<ResponseFormat>,
    reasoning: Option<ReasoningConfig>,
}

#[derive(Default)]
pub struct CompletionRequestBuilder {
    model: Option<String>,
    models: Option<Vec<String>>,
    prompt: Option<String>,
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

impl CompletionRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    setter!(model, into String);
    setter!(models, Vec<String>);
    setter!(prompt, into String);
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
    setter!(route, String);
    setter!(provider, ProviderPreferences);
    setter!(response_format, ResponseFormat);
    setter!(reasoning, ReasoningConfig);

    pub fn build(self) -> Result<CompletionRequest, OpenRouterError> {
        Ok(CompletionRequest {
            model: self
                .model
                .ok_or(OpenRouterError::Validation("model is required".into()))?,
            models: self.models,
            prompt: self
                .prompt
                .ok_or(OpenRouterError::Validation("prompt is required".into()))?,
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

impl CompletionRequest {
    pub fn builder() -> CompletionRequestBuilder {
        CompletionRequestBuilder::new()
    }

    pub fn new(model: &str, prompt: &str) -> Self {
        Self::builder()
            .model(model)
            .prompt(prompt)
            .build()
            .expect("Failed to build CompletionRequest")
    }
}

/// Send a completion request to a selected model (text-only format)
///
/// # Arguments
///
/// * `base_url` - The API URL for the request.
/// * `api_key` - The API key for authentication.
/// * `x_title` - The name of the site for the request.
/// * `http_referer` - The URL of the site for the request.
/// * `request` - The completion request containing the model, prompt, and other optional parameters.
///
/// # Returns
///
/// * `Result<CompletionsResponse, OpenRouterError>` - The response from the completion request, containing the generated text and other details.
pub async fn send_completion_request(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    request: &CompletionRequest,
) -> Result<CompletionsResponse, OpenRouterError> {
    let url = format!("{}/completions", base_url);

    let mut surf_req = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .body_json(request)?;

    if let Some(x_title) = x_title {
        surf_req = surf_req.header("X-Title", x_title);
    }
    if let Some(http_referer) = http_referer {
        surf_req = surf_req.header("HTTP-Referer", http_referer);
    }

    let mut response = surf_req.await?;

    if response.status().is_success() {
        let completion_response = response.body_json().await?;
        Ok(completion_response)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
