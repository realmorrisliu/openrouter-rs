use std::collections::HashMap;

use derive_builder::Builder;
use futures_util::{StreamExt, stream::BoxStream};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::Value;

use crate::{
    error::OpenRouterError,
    strip_option_map_setter, strip_option_vec_setter,
    transport::{
        request as transport_request, response as transport_response, sse::response_lines,
    },
    types::{
        OpenRouterExperimentalMetadata, ProviderPreferences, ReasoningConfig, ResponseFormat, Role,
        completion::CompletionsResponse,
    },
    utils::parse_sse_frames,
};

/// Image URL with optional detail level for vision models.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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

/// Audio input payload for multimodal requests.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InputAudio {
    /// Base64-encoded audio data.
    pub data: String,
    /// Audio format (e.g. wav, mp3, flac).
    pub format: String,
}

impl InputAudio {
    pub fn new(data: impl Into<String>, format: impl Into<String>) -> Self {
        Self {
            data: data.into(),
            format: format.into(),
        }
    }
}

/// Video URL payload for multimodal requests.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct VideoUrl {
    /// URL of the input video.
    pub url: String,
}

impl VideoUrl {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

/// File payload for multimodal requests.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct FileInput {
    /// File content as URL or data URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    /// File id for previously uploaded files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// Optional filename metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

impl FileInput {
    pub fn from_data(file_data: impl Into<String>) -> Self {
        Self {
            file_data: Some(file_data.into()),
            file_id: None,
            filename: None,
        }
    }

    pub fn from_id(file_id: impl Into<String>) -> Self {
        Self {
            file_data: None,
            file_id: Some(file_id.into()),
            filename: None,
        }
    }

    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }
}

/// Cache control type for prompt caching breakpoints.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum CacheControlType {
    Ephemeral,
}

/// Cache control settings for text content parts.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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
#[non_exhaustive]
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
    /// Audio input content
    InputAudio { input_audio: InputAudio },
    /// Video URL content
    VideoUrl { video_url: VideoUrl },
    /// Legacy video input content
    InputVideo { video_url: VideoUrl },
    /// File content
    File { file: FileInput },
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

    pub fn input_audio(data: impl Into<String>, format: impl Into<String>) -> Self {
        Self::InputAudio {
            input_audio: InputAudio::new(data, format),
        }
    }

    pub fn video_url(url: impl Into<String>) -> Self {
        Self::VideoUrl {
            video_url: VideoUrl::new(url),
        }
    }

    pub fn input_video(url: impl Into<String>) -> Self {
        Self::InputVideo {
            video_url: VideoUrl::new(url),
        }
    }

    pub fn file_data(file_data: impl Into<String>) -> Self {
        Self::File {
            file: FileInput::from_data(file_data),
        }
    }

    pub fn file_data_with_filename(
        file_data: impl Into<String>,
        filename: impl Into<String>,
    ) -> Self {
        Self::File {
            file: FileInput::from_data(file_data).filename(filename),
        }
    }

    pub fn file_id(file_id: impl Into<String>) -> Self {
        Self::File {
            file: FileInput::from_id(file_id),
        }
    }

    pub fn file_id_with_filename(file_id: impl Into<String>, filename: impl Into<String>) -> Self {
        Self::File {
            file: FileInput::from_id(file_id).filename(filename),
        }
    }
}

/// Message content - either a simple string or multi-part content.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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
#[non_exhaustive]
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

/// Output modality for chat responses.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Text,
    Image,
    Audio,
}

/// Streaming debug options.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct DebugOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo_upstream_body: Option<bool>,
}

/// Streaming configuration options.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct StreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
}

/// Trace metadata used for observability.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct TraceOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Plugin configuration payload.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct Plugin {
    pub id: String,
    #[serde(flatten)]
    pub config: HashMap<String, Value>,
}

impl Plugin {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            config: HashMap::new(),
        }
    }

    pub fn option(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }
}

/// Stop sequence configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
#[serde(untagged)]
pub enum StopSequence {
    Single(String),
    Multiple(Vec<String>),
}

impl From<String> for StopSequence {
    fn from(value: String) -> Self {
        Self::Single(value)
    }
}

impl From<&str> for StopSequence {
    fn from(value: &str) -> Self {
        Self::Single(value.to_string())
    }
}

impl From<Vec<String>> for StopSequence {
    fn from(value: Vec<String>) -> Self {
        Self::Multiple(value)
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ChatCompletionRequest {
    #[builder(setter(into))]
    model: String,

    messages: Vec<Message>,

    #[builder(setter(skip), default)]
    stream: Option<bool>,

    #[builder(setter(strip_option), default)]
    experimental_metadata: Option<OpenRouterExperimentalMetadata>,

    #[builder(setter(strip_option), default)]
    max_tokens: Option<u32>,

    #[builder(setter(strip_option), default)]
    max_completion_tokens: Option<u32>,

    #[builder(setter(strip_option), default)]
    temperature: Option<f64>,

    #[builder(setter(strip_option), default)]
    seed: Option<u32>,

    #[builder(setter(strip_option), default)]
    top_p: Option<f64>,

    #[builder(setter(strip_option), default)]
    top_k: Option<u32>,

    #[builder(setter(strip_option), default)]
    frequency_penalty: Option<f64>,

    #[builder(setter(strip_option), default)]
    presence_penalty: Option<f64>,

    #[builder(setter(strip_option), default)]
    repetition_penalty: Option<f64>,

    #[builder(setter(custom), default)]
    logit_bias: Option<HashMap<String, f64>>,

    #[builder(setter(strip_option), default)]
    logprobs: Option<bool>,

    #[builder(setter(strip_option), default)]
    top_logprobs: Option<u32>,

    #[builder(setter(strip_option), default)]
    min_p: Option<f64>,

    #[builder(setter(strip_option), default)]
    top_a: Option<f64>,

    #[builder(setter(custom), default)]
    transforms: Option<Vec<String>>,

    #[builder(setter(custom), default)]
    models: Option<Vec<String>>,

    #[builder(setter(into, strip_option), default)]
    route: Option<String>,

    #[builder(setter(into, strip_option), default)]
    user: Option<String>,

    #[builder(setter(into, strip_option), default)]
    session_id: Option<String>,

    #[builder(setter(strip_option), default)]
    cache_control: Option<CacheControl>,

    #[builder(setter(strip_option), default)]
    trace: Option<TraceOptions>,

    #[builder(setter(strip_option), default)]
    provider: Option<ProviderPreferences>,

    #[builder(setter(custom), default)]
    metadata: Option<HashMap<String, String>>,

    #[builder(setter(custom), default)]
    plugins: Option<Vec<Plugin>>,

    #[builder(setter(custom), default)]
    modalities: Option<Vec<Modality>>,

    #[builder(setter(custom), default)]
    image_config: Option<HashMap<String, Value>>,

    #[builder(setter(strip_option), default)]
    response_format: Option<ResponseFormat>,

    #[builder(setter(strip_option), default)]
    reasoning: Option<ReasoningConfig>,

    #[builder(setter(strip_option), default)]
    include_reasoning: Option<bool>,

    #[builder(setter(into, strip_option), default)]
    stop: Option<StopSequence>,

    #[builder(setter(strip_option), default)]
    stream_options: Option<StreamOptions>,

    #[builder(setter(strip_option), default)]
    debug: Option<DebugOptions>,

    #[builder(setter(custom), default)]
    tools: Option<Vec<crate::types::Tool>>,

    #[builder(setter(custom), default)]
    server_tools: Option<Vec<crate::types::ServerTool>>,

    #[builder(setter(strip_option), default)]
    tool_choice: Option<crate::types::ToolChoice>,

    #[builder(setter(strip_option), default)]
    parallel_tool_calls: Option<bool>,
}

#[derive(Deserialize)]
struct ChatCompletionRequestWire {
    model: String,
    messages: Vec<Message>,
    stream: Option<bool>,
    #[serde(skip)]
    experimental_metadata: Option<OpenRouterExperimentalMetadata>,
    max_tokens: Option<u32>,
    max_completion_tokens: Option<u32>,
    temperature: Option<f64>,
    seed: Option<u32>,
    top_p: Option<f64>,
    top_k: Option<u32>,
    frequency_penalty: Option<f64>,
    presence_penalty: Option<f64>,
    repetition_penalty: Option<f64>,
    logit_bias: Option<HashMap<String, f64>>,
    logprobs: Option<bool>,
    top_logprobs: Option<u32>,
    min_p: Option<f64>,
    top_a: Option<f64>,
    transforms: Option<Vec<String>>,
    models: Option<Vec<String>>,
    route: Option<String>,
    user: Option<String>,
    session_id: Option<String>,
    cache_control: Option<CacheControl>,
    trace: Option<TraceOptions>,
    provider: Option<ProviderPreferences>,
    metadata: Option<HashMap<String, String>>,
    plugins: Option<Vec<Plugin>>,
    modalities: Option<Vec<Modality>>,
    image_config: Option<HashMap<String, Value>>,
    response_format: Option<ResponseFormat>,
    reasoning: Option<ReasoningConfig>,
    include_reasoning: Option<bool>,
    stop: Option<StopSequence>,
    stream_options: Option<StreamOptions>,
    debug: Option<DebugOptions>,
    tools: Option<Vec<Value>>,
    tool_choice: Option<crate::types::ToolChoice>,
    parallel_tool_calls: Option<bool>,
}

type SplitChatTools = (
    Option<Vec<crate::types::Tool>>,
    Option<Vec<crate::types::ServerTool>>,
);

impl<'de> Deserialize<'de> for ChatCompletionRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ChatCompletionRequestWire::deserialize(deserializer)?;
        let (tools, server_tools) = split_chat_request_tools(wire.tools)
            .map_err(|error| de::Error::custom(format!("invalid chat tools entry: {error}")))?;

        Ok(Self {
            model: wire.model,
            messages: wire.messages,
            stream: wire.stream,
            experimental_metadata: wire.experimental_metadata,
            max_tokens: wire.max_tokens,
            max_completion_tokens: wire.max_completion_tokens,
            temperature: wire.temperature,
            seed: wire.seed,
            top_p: wire.top_p,
            top_k: wire.top_k,
            frequency_penalty: wire.frequency_penalty,
            presence_penalty: wire.presence_penalty,
            repetition_penalty: wire.repetition_penalty,
            logit_bias: wire.logit_bias,
            logprobs: wire.logprobs,
            top_logprobs: wire.top_logprobs,
            min_p: wire.min_p,
            top_a: wire.top_a,
            transforms: wire.transforms,
            models: wire.models,
            route: wire.route,
            user: wire.user,
            session_id: wire.session_id,
            cache_control: wire.cache_control,
            trace: wire.trace,
            provider: wire.provider,
            metadata: wire.metadata,
            plugins: wire.plugins,
            modalities: wire.modalities,
            image_config: wire.image_config,
            response_format: wire.response_format,
            reasoning: wire.reasoning,
            include_reasoning: wire.include_reasoning,
            stop: wire.stop,
            stream_options: wire.stream_options,
            debug: wire.debug,
            tools,
            server_tools,
            tool_choice: wire.tool_choice,
            parallel_tool_calls: wire.parallel_tool_calls,
        })
    }
}

fn split_chat_request_tools(
    tools: Option<Vec<Value>>,
) -> Result<SplitChatTools, serde_json::Error> {
    let Some(values) = tools else {
        return Ok((None, None));
    };
    let was_empty = values.is_empty();
    let mut function_tools = Vec::new();
    let mut server_tools = Vec::new();

    for value in values {
        let is_function_tool = value.get("function").is_some()
            || value
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|tool_type| tool_type == "function");
        if is_function_tool {
            function_tools.push(serde_json::from_value(value)?);
        } else if crate::types::ServerTool::is_server_tool_value(&value) {
            server_tools.push(serde_json::from_value(value)?);
        } else {
            function_tools.push(serde_json::from_value(value)?);
        }
    }

    let tools = if function_tools.is_empty() && !was_empty {
        None
    } else {
        Some(function_tools)
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

impl Serialize for ChatCompletionRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serde_json::Map::new();

        insert_json_field::<_, S::Error>(&mut map, "model", &self.model)?;
        insert_json_field::<_, S::Error>(&mut map, "messages", &self.messages)?;
        insert_json_option::<_, S::Error>(&mut map, "stream", &self.stream)?;
        insert_json_option::<_, S::Error>(&mut map, "max_tokens", &self.max_tokens)?;
        insert_json_option::<_, S::Error>(
            &mut map,
            "max_completion_tokens",
            &self.max_completion_tokens,
        )?;
        insert_json_option::<_, S::Error>(&mut map, "temperature", &self.temperature)?;
        insert_json_option::<_, S::Error>(&mut map, "seed", &self.seed)?;
        insert_json_option::<_, S::Error>(&mut map, "top_p", &self.top_p)?;
        insert_json_option::<_, S::Error>(&mut map, "top_k", &self.top_k)?;
        insert_json_option::<_, S::Error>(&mut map, "frequency_penalty", &self.frequency_penalty)?;
        insert_json_option::<_, S::Error>(&mut map, "presence_penalty", &self.presence_penalty)?;
        insert_json_option::<_, S::Error>(
            &mut map,
            "repetition_penalty",
            &self.repetition_penalty,
        )?;
        insert_json_option::<_, S::Error>(&mut map, "logit_bias", &self.logit_bias)?;
        insert_json_option::<_, S::Error>(&mut map, "logprobs", &self.logprobs)?;
        insert_json_option::<_, S::Error>(&mut map, "top_logprobs", &self.top_logprobs)?;
        insert_json_option::<_, S::Error>(&mut map, "min_p", &self.min_p)?;
        insert_json_option::<_, S::Error>(&mut map, "top_a", &self.top_a)?;
        insert_json_option::<_, S::Error>(&mut map, "transforms", &self.transforms)?;
        insert_json_option::<_, S::Error>(&mut map, "models", &self.models)?;
        insert_json_option::<_, S::Error>(&mut map, "route", &self.route)?;
        insert_json_option::<_, S::Error>(&mut map, "user", &self.user)?;
        insert_json_option::<_, S::Error>(&mut map, "session_id", &self.session_id)?;
        insert_json_option::<_, S::Error>(&mut map, "cache_control", &self.cache_control)?;
        insert_json_option::<_, S::Error>(&mut map, "trace", &self.trace)?;
        insert_json_option::<_, S::Error>(&mut map, "provider", &self.provider)?;
        insert_json_option::<_, S::Error>(&mut map, "metadata", &self.metadata)?;
        insert_json_option::<_, S::Error>(&mut map, "plugins", &self.plugins)?;
        insert_json_option::<_, S::Error>(&mut map, "modalities", &self.modalities)?;
        insert_json_option::<_, S::Error>(&mut map, "image_config", &self.image_config)?;
        insert_json_option::<_, S::Error>(&mut map, "response_format", &self.response_format)?;
        insert_json_option::<_, S::Error>(&mut map, "reasoning", &self.reasoning)?;
        insert_json_option::<_, S::Error>(&mut map, "include_reasoning", &self.include_reasoning)?;
        insert_json_option::<_, S::Error>(&mut map, "stop", &self.stop)?;
        insert_json_option::<_, S::Error>(&mut map, "stream_options", &self.stream_options)?;
        insert_json_option::<_, S::Error>(&mut map, "debug", &self.debug)?;

        let function_tools = self.tools.as_deref().unwrap_or_default();
        let server_tools = self.server_tools.as_deref().unwrap_or_default();
        if self.tools.is_some() || self.server_tools.is_some() {
            let mut tools = Vec::with_capacity(function_tools.len() + server_tools.len());
            for tool in function_tools {
                tools.push(serde_json::to_value(tool).map_err(serde::ser::Error::custom)?);
            }
            for tool in server_tools {
                tools.push(serde_json::to_value(tool).map_err(serde::ser::Error::custom)?);
            }
            map.insert("tools".to_string(), Value::Array(tools));
        }

        insert_json_option::<_, S::Error>(&mut map, "tool_choice", &self.tool_choice)?;
        insert_json_option::<_, S::Error>(
            &mut map,
            "parallel_tool_calls",
            &self.parallel_tool_calls,
        )?;

        Value::Object(map).serialize(serializer)
    }
}

impl ChatCompletionRequestBuilder {
    strip_option_vec_setter!(models, String);
    strip_option_map_setter!(logit_bias, String, f64);
    strip_option_vec_setter!(transforms, String);
    strip_option_map_setter!(metadata, String, String);
    strip_option_map_setter!(image_config, String, Value);
    strip_option_vec_setter!(plugins, Plugin);
    strip_option_vec_setter!(modalities, Modality);
    strip_option_vec_setter!(tools, crate::types::Tool);
    strip_option_vec_setter!(server_tools, crate::types::ServerTool);

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

    /// Add a single OpenRouter server tool to the request.
    pub fn server_tool(&mut self, tool: crate::types::ServerTool) -> &mut Self {
        if let Some(Some(ref mut existing_tools)) = self.server_tools {
            existing_tools.push(tool);
        } else {
            self.server_tools = Some(Some(vec![tool]));
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

    /// Force the model to use a specific OpenRouter server tool.
    pub fn force_server_tool(&mut self, tool_type: impl Into<String>) -> &mut Self {
        self.tool_choice = Some(Some(crate::types::ToolChoice::force_server_tool(tool_type)));
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
    /// use openrouter_rs::api::chat::{ChatCompletionRequest, Message};
    /// use openrouter_rs::types::Role;
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
    ///     .messages(vec![Message::new(Role::User, "What is the weather in Paris?")])
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
    /// # use openrouter_rs::api::chat::{ChatCompletionRequest, Message};
    /// # use openrouter_rs::types::Role;
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
    ///     .messages(vec![Message::new(Role::User, "Need weather and calculator help")])
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
    /// # use openrouter_rs::api::chat::{ChatCompletionRequest, Message};
    /// # use openrouter_rs::types::Role;
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
    ///     .messages(vec![Message::new(Role::User, "What is the weather in Paris?")])
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
    pub fn tools(&self) -> Option<&[crate::types::Tool]> {
        self.tools.as_deref()
    }

    /// Get the OpenRouter server tools defined in this request.
    pub fn server_tools(&self) -> Option<&[crate::types::ServerTool]> {
        self.server_tools.as_deref()
    }

    pub(crate) fn requires_openrouter_files_tool_header(&self) -> bool {
        self.server_tools
            .as_deref()
            .is_some_and(|tools| tools.iter().any(crate::types::ServerTool::is_files_tool))
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
    pub fn messages(&self) -> &[Message] {
        &self.messages
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
    app_categories: &Option<Vec<String>>,
    request: &ChatCompletionRequest,
) -> Result<CompletionsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    send_chat_completion_with_client(
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

pub(crate) async fn send_chat_completion_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &ChatCompletionRequest,
) -> Result<CompletionsResponse, OpenRouterError> {
    let url = format!("{base_url}/chat/completions");

    // Ensure that the request is not streaming to get a single response
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
        transport_response::parse_json_response(response, "chat completion").await
    } else {
        transport_response::handle_error(response).await?;
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
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &ChatCompletionRequest,
) -> Result<BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    stream_chat_completion_with_client(
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

pub(crate) async fn stream_chat_completion_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    request: &ChatCompletionRequest,
) -> Result<BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>, OpenRouterError> {
    let url = format!("{base_url}/chat/completions");

    // Ensure that the request is streaming to get a continuous response
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
        let lines = parse_sse_frames(response_lines(response))
            .filter_map(async |line| match line {
                Ok(frame) if frame.data == "[DONE]" => None,
                Ok(frame) => Some(
                    serde_json::from_str::<CompletionsResponse>(&frame.data)
                        .map_err(OpenRouterError::Serialization),
                ),
                Err(error) => Some(Err(error)),
            })
            .boxed();

        Ok(lines)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
