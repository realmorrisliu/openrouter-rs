use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;

use crate::{
    error::OpenRouterError,
    strip_option_map_setter, strip_option_vec_setter,
    types::{
        ProviderPreferences, ReasoningConfig, ResponseFormat, completion::CompletionsResponse,
    },
    utils::handle_error,
};

#[derive(Serialize, Deserialize, Debug, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct CompletionRequest {
    #[builder(setter(into))]
    model: String,

    #[builder(setter(into))]
    prompt: String,

    #[builder(setter(strip_option), default)]
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

impl CompletionRequestBuilder {
    strip_option_vec_setter!(models, String);
    strip_option_map_setter!(logit_bias, String, f64);
    strip_option_vec_setter!(transforms, String);
}

impl CompletionRequest {
    pub fn builder() -> CompletionRequestBuilder {
        CompletionRequestBuilder::default()
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
    let url = format!("{base_url}/completions");

    let mut surf_req = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
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
