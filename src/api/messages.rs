use std::collections::HashMap;

use derive_builder::Builder;
use futures_util::{StreamExt, stream::BoxStream};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::{Value, json};

use crate::{
    api::chat::{CacheControl, Plugin, TraceOptions},
    error::OpenRouterError,
    strip_option_vec_setter,
    transport::{
        request as transport_request, response as transport_response, sse::response_lines,
    },
    types::{AnthropicCacheCreation, OpenRouterExperimentalMetadata, ProviderPreferences},
    utils::parse_sse_frames,
};

/// Role for Anthropic-compatible messages.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum AnthropicRole {
    User,
    Assistant,
    System,
}

/// Text block for `system` prompts.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum AnthropicSystemTextBlockType {
    Text,
}

/// System prompt format for Anthropic messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
#[serde(untagged)]
pub enum AnthropicSystemPrompt {
    Text(String),
    Blocks(Vec<AnthropicSystemTextBlock>),
}

/// Message content for Anthropic messages.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentPart {
    OpenrouterBashToolResult {
        tool_use_id: String,
        content: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        container_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        files: Option<Vec<Value>>,
    },
    OpenrouterShellToolResult {
        tool_use_id: String,
        content: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        container_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        files: Option<Vec<Value>>,
    },
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
    Compaction {
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        encrypted_content: Option<String>,
    },
    AdvisorToolResult {
        tool_use_id: String,
        content: Value,
    },
    ToolReference {
        tool_name: String,
    },
    ToolAddition {
        tool: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    ToolRemoval {
        tool: Value,
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

    pub fn document_file_id(file_id: impl Into<String>) -> Self {
        Self::Document {
            source: json!({
                "type": "file",
                "file_id": file_id.into()
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
#[non_exhaustive]
pub struct AnthropicMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clear_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_config: Option<AnthropicOutputConfig>,
    pub role: AnthropicRole,
    pub content: AnthropicMessageContent,
}

impl AnthropicMessage {
    pub fn new(role: AnthropicRole, content: impl Into<AnthropicMessageContent>) -> Self {
        Self {
            clear_at: None,
            output_config: None,
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

    pub fn system(content: impl Into<AnthropicMessageContent>) -> Self {
        Self::new(AnthropicRole::System, content)
    }

    pub fn with_parts(role: AnthropicRole, parts: Vec<AnthropicContentPart>) -> Self {
        Self {
            clear_at: None,
            output_config: None,
            role,
            content: AnthropicMessageContent::Parts(parts),
        }
    }
}

/// Anthropic metadata payload.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[derive(Serialize, Debug, Clone)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicThinking {
    Enabled {
        budget_tokens: u32,
    },
    Disabled,
    Adaptive,
    /// Provider thinking controls including display and block_binding.
    #[serde(untagged)]
    Configured(Value),
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
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum AnthropicOutputEffort {
    Low,
    Medium,
    High,
    Max,
    Xhigh,
}

/// Output config for Anthropic messages.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
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
#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct AnthropicMessagesRequest {
    #[builder(setter(into))]
    model: String,

    max_tokens: u32,

    messages: Vec<AnthropicMessage>,

    #[builder(setter(strip_option), default)]
    system: Option<AnthropicSystemPrompt>,

    #[builder(setter(strip_option), default)]
    metadata: Option<AnthropicMessagesMetadata>,

    #[builder(setter(custom), default)]
    stop_sequences: Option<Vec<String>>,

    #[builder(setter(skip), default)]
    stream: Option<bool>,

    #[builder(setter(strip_option), default)]
    experimental_metadata: Option<OpenRouterExperimentalMetadata>,

    #[builder(setter(strip_option), default)]
    temperature: Option<f64>,

    #[builder(setter(strip_option), default)]
    top_p: Option<f64>,

    #[builder(setter(strip_option), default)]
    top_k: Option<u32>,

    #[builder(setter(custom), default)]
    tools: Option<Vec<AnthropicTool>>,

    #[builder(setter(custom), default)]
    server_tools: Option<Vec<crate::types::ServerTool>>,

    #[builder(setter(strip_option), default)]
    tool_choice: Option<AnthropicToolChoice>,

    #[builder(setter(strip_option), default)]
    thinking: Option<AnthropicThinking>,

    #[builder(setter(into, strip_option), default)]
    service_tier: Option<String>,

    #[builder(setter(strip_option), default)]
    provider: Option<ProviderPreferences>,

    #[builder(setter(custom), default)]
    plugins: Option<Vec<Plugin>>,

    #[builder(setter(into, strip_option), default)]
    route: Option<String>,

    #[builder(setter(into, strip_option), default)]
    user: Option<String>,

    #[builder(setter(into, strip_option), default)]
    session_id: Option<String>,

    #[builder(setter(strip_option), default)]
    trace: Option<TraceOptions>,

    #[builder(setter(custom), default)]
    models: Option<Vec<String>>,

    #[builder(setter(strip_option), default)]
    output_config: Option<AnthropicOutputConfig>,
}

#[derive(Deserialize)]
struct AnthropicMessagesRequestWire {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    system: Option<AnthropicSystemPrompt>,
    metadata: Option<AnthropicMessagesMetadata>,
    stop_sequences: Option<Vec<String>>,
    stream: Option<bool>,
    #[serde(skip)]
    experimental_metadata: Option<OpenRouterExperimentalMetadata>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    top_k: Option<u32>,
    tools: Option<Vec<Value>>,
    tool_choice: Option<AnthropicToolChoice>,
    thinking: Option<AnthropicThinking>,
    service_tier: Option<String>,
    provider: Option<ProviderPreferences>,
    plugins: Option<Vec<Plugin>>,
    route: Option<String>,
    user: Option<String>,
    session_id: Option<String>,
    trace: Option<TraceOptions>,
    models: Option<Vec<String>>,
    output_config: Option<AnthropicOutputConfig>,
}

type SplitAnthropicMessageTools = (
    Option<Vec<AnthropicTool>>,
    Option<Vec<crate::types::ServerTool>>,
);

impl<'de> Deserialize<'de> for AnthropicMessagesRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AnthropicMessagesRequestWire::deserialize(deserializer)?;
        let (tools, server_tools) = split_anthropic_message_tools(wire.tools).map_err(|error| {
            de::Error::custom(format!("invalid Anthropic messages tools entry: {error}"))
        })?;

        Ok(Self {
            model: wire.model,
            max_tokens: wire.max_tokens,
            messages: wire.messages,
            system: wire.system,
            metadata: wire.metadata,
            stop_sequences: wire.stop_sequences,
            stream: wire.stream,
            experimental_metadata: wire.experimental_metadata,
            temperature: wire.temperature,
            top_p: wire.top_p,
            top_k: wire.top_k,
            tools,
            server_tools,
            tool_choice: wire.tool_choice,
            thinking: wire.thinking,
            service_tier: wire.service_tier,
            provider: wire.provider,
            plugins: wire.plugins,
            route: wire.route,
            user: wire.user,
            session_id: wire.session_id,
            trace: wire.trace,
            models: wire.models,
            output_config: wire.output_config,
        })
    }
}

fn split_anthropic_message_tools(
    tools: Option<Vec<Value>>,
) -> Result<SplitAnthropicMessageTools, serde_json::Error> {
    let Some(values) = tools else {
        return Ok((None, None));
    };
    let was_empty = values.is_empty();
    let mut anthropic_tools = Vec::new();
    let mut server_tools = Vec::new();

    for value in values {
        if value.get("name").is_some() {
            anthropic_tools.push(serde_json::from_value(value)?);
        } else if crate::types::ServerTool::is_server_tool_value(&value) {
            server_tools.push(serde_json::from_value(value)?);
        } else {
            anthropic_tools.push(serde_json::from_value(value)?);
        }
    }

    let tools = if anthropic_tools.is_empty() && !was_empty {
        None
    } else {
        Some(anthropic_tools)
    };
    let server_tools = if server_tools.is_empty() {
        None
    } else {
        Some(server_tools)
    };

    Ok((tools, server_tools))
}

fn insert_json_field<T, E>(
    map: &mut serde_json::Map<String, Value>,
    key: &str,
    value: &T,
) -> Result<(), E>
where
    T: Serialize,
    E: serde::ser::Error,
{
    map.insert(
        key.to_string(),
        serde_json::to_value(value).map_err(E::custom)?,
    );
    Ok(())
}

fn insert_json_option<T, E>(
    map: &mut serde_json::Map<String, Value>,
    key: &str,
    value: &Option<T>,
) -> Result<(), E>
where
    T: Serialize,
    E: serde::ser::Error,
{
    if let Some(value) = value {
        insert_json_field::<T, E>(map, key, value)?;
    }
    Ok(())
}

impl Serialize for AnthropicMessagesRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serde_json::Map::new();

        insert_json_field::<_, S::Error>(&mut map, "model", &self.model)?;
        insert_json_field::<_, S::Error>(&mut map, "max_tokens", &self.max_tokens)?;
        insert_json_field::<_, S::Error>(&mut map, "messages", &self.messages)?;
        insert_json_option::<_, S::Error>(&mut map, "system", &self.system)?;
        insert_json_option::<_, S::Error>(&mut map, "metadata", &self.metadata)?;
        insert_json_option::<_, S::Error>(&mut map, "stop_sequences", &self.stop_sequences)?;
        insert_json_option::<_, S::Error>(&mut map, "stream", &self.stream)?;
        insert_json_option::<_, S::Error>(&mut map, "temperature", &self.temperature)?;
        insert_json_option::<_, S::Error>(&mut map, "top_p", &self.top_p)?;
        insert_json_option::<_, S::Error>(&mut map, "top_k", &self.top_k)?;

        let anthropic_tools = self.tools.as_deref().unwrap_or_default();
        let server_tools = self.server_tools.as_deref().unwrap_or_default();
        if self.tools.is_some() || self.server_tools.is_some() {
            let mut tools = Vec::with_capacity(anthropic_tools.len() + server_tools.len());
            for tool in anthropic_tools {
                tools.push(serde_json::to_value(tool).map_err(serde::ser::Error::custom)?);
            }
            for tool in server_tools {
                tools.push(serde_json::to_value(tool).map_err(serde::ser::Error::custom)?);
            }
            map.insert("tools".to_string(), Value::Array(tools));
        }

        insert_json_option::<_, S::Error>(&mut map, "tool_choice", &self.tool_choice)?;
        insert_json_option::<_, S::Error>(&mut map, "thinking", &self.thinking)?;
        insert_json_option::<_, S::Error>(&mut map, "service_tier", &self.service_tier)?;
        insert_json_option::<_, S::Error>(&mut map, "provider", &self.provider)?;
        insert_json_option::<_, S::Error>(&mut map, "plugins", &self.plugins)?;
        insert_json_option::<_, S::Error>(&mut map, "route", &self.route)?;
        insert_json_option::<_, S::Error>(&mut map, "user", &self.user)?;
        insert_json_option::<_, S::Error>(&mut map, "session_id", &self.session_id)?;
        insert_json_option::<_, S::Error>(&mut map, "trace", &self.trace)?;
        insert_json_option::<_, S::Error>(&mut map, "models", &self.models)?;
        insert_json_option::<_, S::Error>(&mut map, "output_config", &self.output_config)?;

        Value::Object(map).serialize(serializer)
    }
}

impl AnthropicMessagesRequestBuilder {
    strip_option_vec_setter!(stop_sequences, String);
    strip_option_vec_setter!(tools, AnthropicTool);
    strip_option_vec_setter!(server_tools, crate::types::ServerTool);
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

    pub fn server_tool(&mut self, tool: crate::types::ServerTool) -> &mut Self {
        if let Some(Some(ref mut existing_tools)) = self.server_tools {
            existing_tools.push(tool);
        } else {
            self.server_tools = Some(Some(vec![tool]));
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

    pub fn messages(&self) -> &[AnthropicMessage] {
        &self.messages
    }

    pub fn tools(&self) -> Option<&[AnthropicTool]> {
        self.tools.as_deref()
    }

    pub fn server_tools(&self) -> Option<&[crate::types::ServerTool]> {
        self.server_tools.as_deref()
    }

    pub(crate) fn requires_openrouter_files_tool_header(&self) -> bool {
        self.server_tools
            .as_deref()
            .is_some_and(|tools| tools.iter().any(crate::types::ServerTool::is_files_tool))
    }

    fn stream(&self, stream: bool) -> Self {
        let mut req = self.clone();
        req.stream = Some(stream);
        req
    }

    pub fn experimental_metadata(&self) -> Option<OpenRouterExperimentalMetadata> {
        self.experimental_metadata
    }
}

/// Usage object in Anthropic messages response.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
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
    pub cache_creation: Option<AnthropicCacheCreation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Non-streaming response payload returned by `POST /messages`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
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
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicMessagesStreamEvent {
    MessageStart {
        message: Box<AnthropicMessagesResponse>,
    },
    MessageDelta {
        delta: Value,
        usage: Value,
    },
    MessageStop {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        openrouter_metadata: Option<Value>,
        #[serde(flatten)]
        extra: HashMap<String, Value>,
    },
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
            Self::MessageStop { .. } => "message_stop",
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
#[non_exhaustive]
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
    app_categories: &Option<Vec<String>>,
    request: &AnthropicMessagesRequest,
) -> Result<AnthropicMessagesResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_message_with_client(
        &http_client,
        base_url,
        api_key,
        x_title,
        http_referer,
        app_categories,
        request,
    )
    .await
}

pub(crate) async fn create_message_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &AnthropicMessagesRequest,
) -> Result<AnthropicMessagesResponse, OpenRouterError> {
    let url = format!("{base_url}/messages");
    let request = request.stream(false);

    let request_builder = transport_request::with_experimental_metadata_header(
        transport_request::with_client_request_headers(
            transport_request::post(http_client, &url),
            api_key,
            x_title,
            http_referer,
            app_categories,
        )?,
        &request.experimental_metadata,
    );
    let request_builder = transport_request::with_openrouter_files_tool_header(
        request_builder,
        request.requires_openrouter_files_tool_header(),
    );

    let response = request_builder.json(&request).send().await?;

    if response.status().is_success() {
        let response_data: AnthropicMessagesResponse =
            transport_response::parse_json_response(response, "messages API").await?;
        Ok(response_data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Send a streaming request to the Anthropic-compatible Messages API.
pub async fn stream_messages(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &AnthropicMessagesRequest,
) -> Result<BoxStream<'static, Result<AnthropicMessagesSseEvent, OpenRouterError>>, OpenRouterError>
{
    let http_client = crate::transport::new_client()?;
    stream_messages_with_client(
        &http_client,
        base_url,
        api_key,
        x_title,
        http_referer,
        app_categories,
        request,
    )
    .await
}

pub(crate) async fn stream_messages_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &AnthropicMessagesRequest,
) -> Result<BoxStream<'static, Result<AnthropicMessagesSseEvent, OpenRouterError>>, OpenRouterError>
{
    let url = format!("{base_url}/messages");
    let request = request.stream(true);

    let request_builder = transport_request::with_experimental_metadata_header(
        transport_request::with_client_request_headers(
            transport_request::post(http_client, &url),
            api_key,
            x_title,
            http_referer,
            app_categories,
        )?,
        &request.experimental_metadata,
    );
    let request_builder = transport_request::with_openrouter_files_tool_header(
        request_builder,
        request.requires_openrouter_files_tool_header(),
    );

    let response = request_builder.json(&request).send().await?;

    if response.status().is_success() {
        let stream = parse_sse_frames(response_lines(response))
            .filter_map(async |frame| match frame {
                Ok(frame) if frame.data == "[DONE]" => None,
                Ok(frame) => Some(
                    serde_json::from_str::<AnthropicMessagesStreamEvent>(&frame.data)
                        .map_err(OpenRouterError::Serialization)
                        .map(|payload| AnthropicMessagesSseEvent {
                            event: frame
                                .event
                                .unwrap_or_else(|| payload.event_type().to_string()),
                            data: payload,
                        }),
                ),
                Err(error) => Some(Err(error)),
            })
            .boxed();

        Ok(stream)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

impl<'de> Deserialize<'de> for AnthropicThinking {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let kind = value.get("type").and_then(Value::as_str);
        match kind {
            Some("enabled") => {
                let budget = value
                    .get("budget_tokens")
                    .and_then(Value::as_u64)
                    .and_then(|n| u32::try_from(n).ok())
                    .ok_or_else(|| {
                        serde::de::Error::custom("enabled thinking requires budget_tokens")
                    })?;
                if value.as_object().is_some_and(|o| o.len() == 2) {
                    Ok(Self::enabled(budget))
                } else {
                    Ok(Self::Configured(value))
                }
            }
            Some("adaptive") if value.as_object().is_some_and(|o| o.len() == 1) => {
                Ok(Self::Adaptive)
            }
            Some("disabled") if value.as_object().is_some_and(|o| o.len() == 1) => {
                Ok(Self::Disabled)
            }
            Some(_) => Ok(Self::Configured(value)),
            None => Err(serde::de::Error::custom("thinking requires a type")),
        }
    }
}
