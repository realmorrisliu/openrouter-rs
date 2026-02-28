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

/// Image URL with optional detail level for vision models.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageUrl {
    /// URL of the image (can be a web URL or base64 data URI)
    pub url: String,
    /// Detail level: "auto", "low", or "high"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ImageUrl {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            detail: None,
        }
    }

    pub fn with_detail(url: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            detail: Some(detail.into()),
        }
    }
}

/// Cache control type for prompt caching breakpoints.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CacheControlType {
    Ephemeral,
}

/// Cache control settings for text content parts.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub kind: CacheControlType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
}

impl CacheControl {
    /// Create cache control using default ephemeral TTL.
    pub fn ephemeral() -> Self {
        Self {
            kind: CacheControlType::Ephemeral,
            ttl: None,
        }
    }

    /// Create cache control with explicit TTL (e.g. "1h").
    pub fn ephemeral_with_ttl(ttl: impl Into<String>) -> Self {
        Self {
            kind: CacheControlType::Ephemeral,
            ttl: Some(ttl.into()),
        }
    }
}

/// A content part in a multi-modal message.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Text content
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Image URL content
    ImageUrl { image_url: ImageUrl },
}

impl ContentPart {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            cache_control: None,
        }
    }

    pub fn text_with_cache_control(text: impl Into<String>, cache_control: CacheControl) -> Self {
        Self::Text {
            text: text.into(),
            cache_control: Some(cache_control),
        }
    }

    pub fn cacheable_text(text: impl Into<String>) -> Self {
        Self::text_with_cache_control(text, CacheControl::ephemeral())
    }

    pub fn cacheable_text_with_ttl(text: impl Into<String>, ttl: impl Into<String>) -> Self {
        Self::text_with_cache_control(text, CacheControl::ephemeral_with_ttl(ttl))
    }

    pub fn image_url(url: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl::new(url),
        }
    }

    pub fn image_url_with_detail(url: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl::with_detail(url, detail),
        }
    }
}

/// Message content - either a simple string or multi-part content.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Content {
    /// Simple text content
    Text(String),
    /// Multi-part content (text, images, etc.)
    Parts(Vec<ContentPart>),
}

impl From<String> for Content {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for Content {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

impl From<Vec<ContentPart>> for Content {
    fn from(parts: Vec<ContentPart>) -> Self {
        Self::Parts(parts)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: Content,
    /// Optional name for tool messages or function calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Tool call ID for tool response messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Tool calls made by assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::types::ToolCall>>,
}

impl Message {
    pub fn new(role: Role, content: impl Into<Content>) -> Self {
        Self {
            role,
            content: content.into(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        }
    }

    /// Create a message with multi-part content (text and images).
    pub fn with_parts(role: Role, parts: Vec<ContentPart>) -> Self {
        Self {
            role,
            content: Content::Parts(parts),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        }
    }

    /// Create a tool response message
    pub fn tool_response(tool_call_id: &str, content: impl Into<Content>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            name: None,
            tool_call_id: Some(tool_call_id.to_string()),
            tool_calls: None,
        }
    }

    /// Create a tool response message with a specific tool name
    pub fn tool_response_named(
        tool_call_id: &str,
        tool_name: &str,
        content: impl Into<Content>,
    ) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            name: Some(tool_name.to_string()),
            tool_call_id: Some(tool_call_id.to_string()),
            tool_calls: None,
        }
    }

    /// Create a message with a specific name
    pub fn named(role: Role, name: &str, content: impl Into<Content>) -> Self {
        Self {
            role,
            content: content.into(),
            name: Some(name.to_string()),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    /// Create an assistant message with tool calls
    pub fn assistant_with_tool_calls(
        content: impl Into<Content>,
        tool_calls: Vec<crate::types::ToolCall>,
    ) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
            tool_call_id: None,
            tool_calls: Some(tool_calls),
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

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    include_reasoning: Option<bool>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<crate::types::Tool>>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<crate::types::ToolChoice>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,
}

impl ChatCompletionRequestBuilder {
    strip_option_vec_setter!(models, String);
    strip_option_map_setter!(logit_bias, String, f64);
    strip_option_vec_setter!(transforms, String);
    strip_option_vec_setter!(tools, crate::types::Tool);

    /// Enable reasoning with default settings (medium effort)
    pub fn enable_reasoning(&mut self) -> &mut Self {
        use crate::types::ReasoningConfig;
        self.reasoning = Some(Some(ReasoningConfig::enabled()));
        self
    }

    /// Set reasoning effort level
    pub fn reasoning_effort(&mut self, effort: crate::types::Effort) -> &mut Self {
        use crate::types::ReasoningConfig;
        self.reasoning = Some(Some(ReasoningConfig::with_effort(effort)));
        self
    }

    /// Set reasoning max tokens
    pub fn reasoning_max_tokens(&mut self, max_tokens: u32) -> &mut Self {
        use crate::types::ReasoningConfig;
        self.reasoning = Some(Some(ReasoningConfig::with_max_tokens(max_tokens)));
        self
    }

    /// Exclude reasoning from response (use reasoning internally but don't return it)
    pub fn exclude_reasoning(&mut self) -> &mut Self {
        use crate::types::ReasoningConfig;
        self.reasoning = Some(Some(ReasoningConfig::excluded()));
        self
    }

    /// Add a single tool to the request
    pub fn tool(&mut self, tool: crate::types::Tool) -> &mut Self {
        if let Some(Some(ref mut existing_tools)) = self.tools {
            existing_tools.push(tool);
        } else {
            self.tools = Some(Some(vec![tool]));
        }
        self
    }

    /// Set tool choice to auto (model chooses whether to use tools)
    pub fn tool_choice_auto(&mut self) -> &mut Self {
        self.tool_choice = Some(Some(crate::types::ToolChoice::auto()));
        self
    }

    /// Set tool choice to none (model will not use tools)
    pub fn tool_choice_none(&mut self) -> &mut Self {
        self.tool_choice = Some(Some(crate::types::ToolChoice::none()));
        self
    }

    /// Set tool choice to required (model must use tools)
    pub fn tool_choice_required(&mut self) -> &mut Self {
        self.tool_choice = Some(Some(crate::types::ToolChoice::required()));
        self
    }

    /// Force the model to use a specific tool
    pub fn force_tool(&mut self, tool_name: &str) -> &mut Self {
        self.tool_choice = Some(Some(crate::types::ToolChoice::force_tool(tool_name)));
        self
    }

    /// Add a typed tool to the request
    ///
    /// This method allows adding strongly-typed tools using the TypedTool trait.
    /// The tool's JSON Schema is automatically generated from the Rust type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openrouter_rs::types::typed_tool::TypedTool;
    /// use serde::{Deserialize, Serialize};
    /// use schemars::JsonSchema;
    ///
    /// #[derive(Serialize, Deserialize, JsonSchema)]
    /// struct WeatherParams {
    ///     location: String,
    /// }
    ///
    /// impl TypedTool for WeatherParams {
    ///     fn name() -> &'static str { "get_weather" }
    ///     fn description() -> &'static str { "Get weather for location" }
    /// }
    ///
    /// let request = ChatCompletionRequest::builder()
    ///     .model("anthropic/claude-sonnet-4")
    ///     .typed_tool::<WeatherParams>()
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn typed_tool<T: crate::types::TypedTool>(&mut self) -> &mut Self {
        let tool = T::create_tool();
        self.tool(tool)
    }

    /// Add multiple typed tools to the request
    ///
    /// This is a convenience method for adding multiple typed tools at once.
    /// Each tool type must implement the TypedTool trait.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use openrouter_rs::types::typed_tool::TypedTool;
    /// # use serde::{Deserialize, Serialize};
    /// # use schemars::JsonSchema;
    /// # #[derive(Serialize, Deserialize, JsonSchema)]
    /// # struct WeatherParams { location: String }
    /// # impl TypedTool for WeatherParams {
    /// #     fn name() -> &'static str { "get_weather" }
    /// #     fn description() -> &'static str { "Get weather" }
    /// # }
    /// # #[derive(Serialize, Deserialize, JsonSchema)]
    /// # struct CalculatorParams { a: f64, b: f64 }
    /// # impl TypedTool for CalculatorParams {
    /// #     fn name() -> &'static str { "calculator" }
    /// #     fn description() -> &'static str { "Calculate" }
    /// # }
    ///
    /// let request = ChatCompletionRequest::builder()
    ///     .model("anthropic/claude-sonnet-4")
    ///     .typed_tools_batch(&[
    ///         WeatherParams::create_tool(),
    ///         CalculatorParams::create_tool(),
    ///     ])
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn typed_tools_batch(&mut self, tools: &[crate::types::Tool]) -> &mut Self {
        for tool in tools {
            self.tool(tool.clone());
        }
        self
    }

    /// Force the model to use a specific typed tool
    ///
    /// This method combines the typed tool functionality with tool choice forcing.
    /// The specified typed tool will be added to the tools list and forced as the choice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use openrouter_rs::types::typed_tool::TypedTool;
    /// # use serde::{Deserialize, Serialize};
    /// # use schemars::JsonSchema;
    /// # #[derive(Serialize, Deserialize, JsonSchema)]
    /// # struct WeatherParams { location: String }
    /// # impl TypedTool for WeatherParams {
    /// #     fn name() -> &'static str { "get_weather" }
    /// #     fn description() -> &'static str { "Get weather" }
    /// # }
    ///
    /// let request = ChatCompletionRequest::builder()
    ///     .model("anthropic/claude-sonnet-4")
    ///     .force_typed_tool::<WeatherParams>()
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn force_typed_tool<T: crate::types::TypedTool>(&mut self) -> &mut Self {
        let tool_name = T::name();
        let tool = T::create_tool();
        self.tool(tool);
        self.force_tool(tool_name);
        self
    }
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

    /// Get the tools defined in this request
    pub fn tools(&self) -> Option<&Vec<crate::types::Tool>> {
        self.tools.as_ref()
    }

    /// Get the tool choice setting
    pub fn tool_choice(&self) -> Option<&crate::types::ToolChoice> {
        self.tool_choice.as_ref()
    }

    /// Get the parallel tool calls setting
    pub fn parallel_tool_calls(&self) -> Option<bool> {
        self.parallel_tool_calls
    }

    /// Get the messages in this request
    pub fn messages(&self) -> &Vec<Message> {
        &self.messages
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
    let url = format!("{base_url}/chat/completions");

    // Ensure that the request is not streaming to get a single response
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
        let body_text = response.body_string().await?;
        let chat_response: CompletionsResponse = serde_json::from_str(&body_text).map_err(|e| {
            eprintln!("Failed to deserialize response: {e}\nBody: {body_text}");
            OpenRouterError::Serialization(e)
        })?;
        Ok(chat_response)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
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
/// * `Result<BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>, OpenRouterError>` - A stream of chat completion events or an error.
pub async fn stream_chat_completion(
    base_url: &str,
    api_key: &str,
    request: &ChatCompletionRequest,
) -> Result<BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>, OpenRouterError> {
    let url = format!("{base_url}/chat/completions");

    // Ensure that the request is streaming to get a continuous response
    let request = request.stream(true);

    let response = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .body_json(&request)?
        .await?;

    if response.status().is_success() {
        let lines = response
            .lines()
            .filter_map(async |line| match line {
                Ok(line) => line
                    .strip_prefix("data: ")
                    .filter(|line| *line != "[DONE]")
                    .map(serde_json::from_str::<CompletionsResponse>)
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
