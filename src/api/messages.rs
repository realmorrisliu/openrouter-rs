use std::collections::HashMap;

use derive_builder::Builder;
use futures_util::{AsyncBufReadExt, StreamExt, stream, stream::BoxStream};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use surf::http::headers::AUTHORIZATION;

use crate::{
    api::chat::{CacheControl, Plugin, TraceOptions},
    error::OpenRouterError,
    strip_option_vec_setter,
    types::ProviderPreferences,
    utils::handle_error,
};

/// Role for Anthropic-compatible messages.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnthropicRole {
    User,
    Assistant,
}

/// Text block for `system` prompts.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicSystemTextBlock {
    #[serde(rename = "type")]
    pub block_type: AnthropicSystemTextBlockType,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl AnthropicSystemTextBlock {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            block_type: AnthropicSystemTextBlockType::Text,
            text: text.into(),
            citations: None,
            cache_control: None,
            extra: HashMap::new(),
        }
    }
}

/// Block type for Anthropic system text blocks.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnthropicSystemTextBlockType {
    Text,
}

/// System prompt format for Anthropic messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AnthropicSystemPrompt {
    Text(String),
    Blocks(Vec<AnthropicSystemTextBlock>),
}

/// Message content for Anthropic messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AnthropicMessageContent {
    Text(String),
    Parts(Vec<AnthropicContentPart>),
}

impl From<String> for AnthropicMessageContent {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for AnthropicMessageContent {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<Vec<AnthropicContentPart>> for AnthropicMessageContent {
    fn from(value: Vec<AnthropicContentPart>) -> Self {
        Self::Parts(value)
    }
}

/// Multi-modal content part for Anthropic-compatible messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentPart {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<Vec<Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Image {
        source: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Document {
        source: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<Vec<Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    ToolUse {
        id: String,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        input: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<AnthropicMessageContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Thinking {
        thinking: String,
        signature: String,
    },
    RedactedThinking {
        data: String,
    },
    ServerToolUse {
        id: String,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        input: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    WebSearchToolResult {
        tool_use_id: String,
        content: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    SearchResult {
        source: String,
        title: String,
        content: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<Vec<Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

impl AnthropicContentPart {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            citations: None,
            cache_control: None,
        }
    }

    pub fn image_url(url: impl Into<String>) -> Self {
        Self::Image {
            source: json!({
                "type": "url",
                "url": url.into()
            }),
            cache_control: None,
        }
    }

    pub fn image_base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self::Image {
            source: json!({
                "type": "base64",
                "media_type": media_type.into(),
                "data": data.into()
            }),
            cache_control: None,
        }
    }

    pub fn document_url(url: impl Into<String>) -> Self {
        Self::Document {
            source: json!({
                "type": "url",
                "url": url.into()
            }),
            title: None,
            context: None,
            citations: None,
            cache_control: None,
        }
    }

    pub fn tool_use(
        id: impl Into<String>,
        name: impl Into<String>,
        input: impl Into<Value>,
    ) -> Self {
        Self::ToolUse {
            id: id.into(),
            name: name.into(),
            input: Some(input.into()),
            cache_control: None,
        }
    }

    pub fn tool_result(
        tool_use_id: impl Into<String>,
        content: impl Into<AnthropicMessageContent>,
    ) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: Some(content.into()),
            is_error: None,
            cache_control: None,
        }
    }
}

/// A user/assistant message in Anthropic format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicMessage {
    pub role: AnthropicRole,
    pub content: AnthropicMessageContent,
}

impl AnthropicMessage {
    pub fn new(role: AnthropicRole, content: impl Into<AnthropicMessageContent>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<AnthropicMessageContent>) -> Self {
        Self::new(AnthropicRole::User, content)
    }

    pub fn assistant(content: impl Into<AnthropicMessageContent>) -> Self {
        Self::new(AnthropicRole::Assistant, content)
    }

    pub fn with_parts(role: AnthropicRole, parts: Vec<AnthropicContentPart>) -> Self {
        Self {
            role,
            content: AnthropicMessageContent::Parts(parts),
        }
    }
}

/// Anthropic metadata payload.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnthropicMessagesMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl AnthropicMessagesMetadata {
    pub fn with_user_id(user_id: impl Into<String>) -> Self {
        Self {
            user_id: Some(user_id.into()),
            extra: HashMap::new(),
        }
    }
}

/// Tool definition for Anthropic-compatible messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl AnthropicTool {
    pub fn custom(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: impl Into<Value>,
    ) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            input_schema: Some(input_schema.into()),
            tool_type: Some("custom".to_string()),
            cache_control: None,
            extra: HashMap::new(),
        }
    }

    pub fn hosted(tool_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema: None,
            tool_type: Some(tool_type.into()),
            cache_control: None,
            extra: HashMap::new(),
        }
    }

    pub fn option(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Tool choice policy for Anthropic-compatible messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicToolChoice {
    Auto {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    Any {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    None,
    Tool {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
}

impl AnthropicToolChoice {
    pub fn auto() -> Self {
        Self::Auto {
            disable_parallel_tool_use: None,
        }
    }

    pub fn any() -> Self {
        Self::Any {
            disable_parallel_tool_use: None,
        }
    }

    pub fn none() -> Self {
        Self::None
    }

    pub fn tool(name: impl Into<String>) -> Self {
        Self::Tool {
            name: name.into(),
            disable_parallel_tool_use: None,
        }
    }
}

/// Thinking control for Anthropic-compatible messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicThinking {
    Enabled { budget_tokens: u32 },
    Disabled,
    Adaptive,
}

impl AnthropicThinking {
    pub fn enabled(budget_tokens: u32) -> Self {
        Self::Enabled { budget_tokens }
    }

    pub fn disabled() -> Self {
        Self::Disabled
    }

    pub fn adaptive() -> Self {
        Self::Adaptive
    }
}

/// Output effort level for Anthropic output configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnthropicOutputEffort {
    Low,
    Medium,
    High,
    Max,
}

/// Output config for Anthropic messages.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnthropicOutputConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<AnthropicOutputEffort>,
}

impl AnthropicOutputConfig {
    pub fn with_effort(effort: AnthropicOutputEffort) -> Self {
        Self {
            effort: Some(effort),
        }
    }
}

/// Request body for `POST /messages`.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct AnthropicMessagesRequest {
    #[builder(setter(into))]
    model: String,

    max_tokens: u32,

    messages: Vec<AnthropicMessage>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<AnthropicSystemPrompt>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<AnthropicMessagesMetadata>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,

    #[builder(setter(skip), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<AnthropicToolChoice>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<AnthropicThinking>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    service_tier: Option<String>,

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

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    models: Option<Vec<String>>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<AnthropicOutputConfig>,
}

impl AnthropicMessagesRequestBuilder {
    strip_option_vec_setter!(stop_sequences, String);
    strip_option_vec_setter!(tools, AnthropicTool);
    strip_option_vec_setter!(plugins, Plugin);
    strip_option_vec_setter!(models, String);

    pub fn tool(&mut self, tool: AnthropicTool) -> &mut Self {
        if let Some(Some(ref mut existing_tools)) = self.tools {
            existing_tools.push(tool);
        } else {
            self.tools = Some(Some(vec![tool]));
        }
        self
    }

    pub fn add_message(&mut self, message: AnthropicMessage) -> &mut Self {
        if let Some(ref mut messages) = self.messages {
            messages.push(message);
        } else {
            self.messages = Some(vec![message]);
        }
        self
    }

    pub fn thinking_enabled(&mut self, budget_tokens: u32) -> &mut Self {
        self.thinking = Some(Some(AnthropicThinking::enabled(budget_tokens)));
        self
    }
}

impl AnthropicMessagesRequest {
    pub fn builder() -> AnthropicMessagesRequestBuilder {
        AnthropicMessagesRequestBuilder::default()
    }

    pub fn new(model: impl Into<String>, max_tokens: u32, messages: Vec<AnthropicMessage>) -> Self {
        Self::builder()
            .model(model.into())
            .max_tokens(max_tokens)
            .messages(messages)
            .build()
            .expect("Failed to build AnthropicMessagesRequest")
    }

    pub fn messages(&self) -> &Vec<AnthropicMessage> {
        &self.messages
    }

    pub fn tools(&self) -> Option<&Vec<AnthropicTool>> {
        self.tools.as_ref()
    }

    fn stream(&self, stream: bool) -> Self {
        let mut req = self.clone();
        req.stream = Some(stream);
        req
    }
}

/// Usage object in Anthropic messages response.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnthropicMessagesUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Non-streaming response payload returned by `POST /messages`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnthropicMessagesResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<AnthropicContentPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AnthropicMessagesUsage>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Streaming data event payload for `POST /messages` when `stream=true`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicMessagesStreamEvent {
    MessageStart {
        message: Box<AnthropicMessagesResponse>,
    },
    MessageDelta {
        delta: Value,
        usage: Value,
    },
    MessageStop,
    ContentBlockStart {
        index: u32,
        content_block: Box<AnthropicContentPart>,
    },
    ContentBlockDelta {
        index: u32,
        delta: Value,
    },
    ContentBlockStop {
        index: u32,
    },
    Ping,
    Error {
        error: Value,
    },
}

impl AnthropicMessagesStreamEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::MessageStart { .. } => "message_start",
            Self::MessageDelta { .. } => "message_delta",
            Self::MessageStop => "message_stop",
            Self::ContentBlockStart { .. } => "content_block_start",
            Self::ContentBlockDelta { .. } => "content_block_delta",
            Self::ContentBlockStop { .. } => "content_block_stop",
            Self::Ping => "ping",
            Self::Error { .. } => "error",
        }
    }
}

/// Streaming SSE envelope returned by `POST /messages`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicMessagesSseEvent {
    pub event: String,
    pub data: AnthropicMessagesStreamEvent,
}

/// Send a non-streaming request to the Anthropic-compatible Messages API.
pub async fn create_message(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    request: &AnthropicMessagesRequest,
) -> Result<AnthropicMessagesResponse, OpenRouterError> {
    let url = format!("{base_url}/messages");
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
        let response_data: AnthropicMessagesResponse = response.body_json().await?;
        Ok(response_data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Send a streaming request to the Anthropic-compatible Messages API.
pub async fn stream_messages(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    request: &AnthropicMessagesRequest,
) -> Result<BoxStream<'static, Result<AnthropicMessagesSseEvent, OpenRouterError>>, OpenRouterError>
{
    let url = format!("{base_url}/messages");
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
        let lines = response.lines();
        let stream = stream::unfold(
            (lines, None::<String>),
            |(mut lines, mut current_event)| async move {
                loop {
                    match lines.next().await {
                        Some(Ok(line)) => {
                            if line.is_empty() || line.starts_with(':') {
                                continue;
                            }
                            if let Some(event_name) = line.strip_prefix("event: ") {
                                current_event = Some(event_name.to_string());
                                continue;
                            }
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    return None;
                                }
                                let parsed =
                                    serde_json::from_str::<AnthropicMessagesStreamEvent>(data)
                                        .map_err(OpenRouterError::Serialization)
                                        .map(|payload| {
                                            let event =
                                                current_event.clone().unwrap_or_else(|| {
                                                    payload.event_type().to_string()
                                                });
                                            AnthropicMessagesSseEvent {
                                                event,
                                                data: payload,
                                            }
                                        });
                                return Some((parsed, (lines, None)));
                            }
                        }
                        Some(Err(error)) => {
                            return Some((Err(OpenRouterError::Io(error)), (lines, current_event)));
                        }
                        None => return None,
                    }
                }
            },
        )
        .boxed();

        Ok(stream)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
