use std::collections::HashMap;

use derive_builder::Builder;
use futures_util::{StreamExt, stream::BoxStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::transport::RequestContext;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing: Option<String>,
    /// URL of the input video.
    pub url: String,
}

impl VideoUrl {
    pub fn processing(mut self, processing: impl Into<String>) -> Self {
        self.processing = Some(processing.into());
        self
    }
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            processing: None,
        }
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

/// Explicit prompt-cache mode used by OpenAI-style cache controls.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum PromptCacheMode {
    Explicit,
}

/// Marks an explicit prompt-cache boundary on a text content part.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PromptCacheBreakpoint {
    pub mode: PromptCacheMode,
}

impl PromptCacheBreakpoint {
    pub fn explicit() -> Self {
        Self {
            mode: PromptCacheMode::Explicit,
        }
    }
}

/// Request-level controls for explicit prompt caching.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PromptCacheOptions {
    pub mode: PromptCacheMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
}

impl PromptCacheOptions {
    pub fn explicit() -> Self {
        Self {
            mode: PromptCacheMode::Explicit,
            ttl: None,
        }
    }

    pub fn explicit_with_ttl(ttl: impl Into<String>) -> Self {
        Self {
            mode: PromptCacheMode::Explicit,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        prompt_cache_breakpoint: Option<PromptCacheBreakpoint>,
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
            prompt_cache_breakpoint: None,
        }
    }

    pub fn text_with_cache_control(text: impl Into<String>, cache_control: CacheControl) -> Self {
        Self::Text {
            text: text.into(),
            cache_control: Some(cache_control),
            prompt_cache_breakpoint: None,
        }
    }

    pub fn text_with_prompt_cache_breakpoint(
        text: impl Into<String>,
        breakpoint: PromptCacheBreakpoint,
    ) -> Self {
        Self::Text {
            text: text.into(),
            cache_control: None,
            prompt_cache_breakpoint: Some(breakpoint),
        }
    }

    pub fn cache_breakpoint_text(text: impl Into<String>) -> Self {
        Self::text_with_prompt_cache_breakpoint(text, PromptCacheBreakpoint::explicit())
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
    /// No text content, as commonly returned with assistant tool calls.
    Null,
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

/// One text part in a static predicted output.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PredictionContentText {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl PredictionContentText {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            content_type: "text".to_string(),
            text: text.into(),
        }
    }
}

/// Static predicted output content.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
#[serde(untagged)]
pub enum PredictionContent {
    Text(String),
    Parts(Vec<PredictionContentText>),
}

/// Static predicted output used to reduce latency when most output is known.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Prediction {
    #[serde(rename = "type")]
    pub prediction_type: String,
    pub content: PredictionContent,
}

impl Prediction {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            prediction_type: "content".to_string(),
            content: PredictionContent::Text(text.into()),
        }
    }

    pub fn parts(parts: Vec<PredictionContentText>) -> Self {
        Self {
            prediction_type: "content".to_string(),
            content: PredictionContent::Parts(parts),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_update: Option<Value>,
    pub role: Role,
    pub content: Content,
    /// Model that generated an assistant message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional name for tool messages or function calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Tool call ID for tool response messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Tool calls made by assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::types::ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    /// Reasoning text in the field some OpenAI-compatible upstreams use and expect replayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    /// Reasoning text in the field some OpenAI-compatible upstreams use and expect replayed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<Vec<crate::types::completion::ReasoningDetail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Value>,
}

impl Message {
    pub fn new(role: Role, content: impl Into<Content>) -> Self {
        Self {
            reasoning: None,
            reasoning_content: None,
            reasoning_text: None,
            reasoning_details: None,
            refusal: None,
            annotations: None,
            images: None,
            audio: None,
            configuration_update: None,
            role,
            content: content.into(),
            model: None,
            name: None,
            tool_call_id: None,
            tool_calls: None,
        }
    }

    /// Create a message with multi-part content (text and images).
    pub fn with_parts(role: Role, parts: Vec<ContentPart>) -> Self {
        Self::new(role, parts)
    }

    /// Create a tool response message
    pub fn tool_response(tool_call_id: &str, content: impl Into<Content>) -> Self {
        Self {
            tool_call_id: Some(tool_call_id.to_string()),
            ..Self::new(Role::Tool, content)
        }
    }

    /// Create a tool response message with a specific tool name
    pub fn tool_response_named(
        tool_call_id: &str,
        tool_name: &str,
        content: impl Into<Content>,
    ) -> Self {
        Self {
            name: Some(tool_name.to_string()),
            ..Self::tool_response(tool_call_id, content)
        }
    }

    /// Create a message with a specific name
    pub fn named(role: Role, name: &str, content: impl Into<Content>) -> Self {
        Self {
            name: Some(name.to_string()),
            ..Self::new(role, content)
        }
    }

    /// Create an assistant message with tool calls
    pub fn assistant_with_tool_calls(
        content: impl Into<Content>,
        tool_calls: Vec<crate::types::ToolCall>,
    ) -> Self {
        Self {
            tool_calls: Some(tool_calls),
            ..Self::new(Role::Assistant, content)
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_reasoning_content(mut self, reasoning_content: impl Into<String>) -> Self {
        self.reasoning_content = Some(reasoning_content.into());
        self
    }

    pub fn with_reasoning_text(mut self, reasoning_text: impl Into<String>) -> Self {
        self.reasoning_text = Some(reasoning_text.into());
        self
    }
}

impl TryFrom<&crate::types::completion::Message> for Message {
    type Error = serde_json::Error;

    /// Convert a response message into a conversation message, preserving all
    /// modeled fields, including null content and signed reasoning details.
    /// Response content parts have already been normalized to text by the SDK.
    /// Missing or unsupported roles are rejected rather than silently changed.
    fn try_from(message: &crate::types::completion::Message) -> Result<Self, Self::Error> {
        serde_json::from_value(serde_json::to_value(message)?)
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

/// One entry in a chat completion request's wire `tools` array.
///
/// Function tools are executed by the caller; server tools are executed by
/// OpenRouter. Both travel in the same array, so the request stores one list.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
#[serde(untagged)]
pub enum ChatTool {
    /// Caller-defined function tool.
    Function(crate::types::Tool),
    /// OpenRouter-operated server tool.
    Server(crate::types::ServerTool),
}

impl ChatTool {
    /// Borrow this entry as a function tool.
    pub fn as_function(&self) -> Option<&crate::types::Tool> {
        match self {
            ChatTool::Function(tool) => Some(tool),
            ChatTool::Server(_) => None,
        }
    }

    /// Borrow this entry as a server tool.
    pub fn as_server(&self) -> Option<&crate::types::ServerTool> {
        match self {
            ChatTool::Server(tool) => Some(tool),
            ChatTool::Function(_) => None,
        }
    }
}

impl From<crate::types::Tool> for ChatTool {
    fn from(tool: crate::types::Tool) -> Self {
        ChatTool::Function(tool)
    }
}

impl From<crate::types::ServerTool> for ChatTool {
    fn from(tool: crate::types::ServerTool) -> Self {
        ChatTool::Server(tool)
    }
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ChatCompletionRequest {
    #[builder(setter(into))]
    model: String,

    messages: Vec<Message>,

    #[builder(setter(skip), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,

    #[builder(setter(strip_option), default)]
    #[serde(skip)]
    experimental_metadata: Option<OpenRouterExperimentalMetadata>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,

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
    logprobs: Option<bool>,

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

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_control: Option<CacheControl>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_cache_key: Option<String>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_cache_options: Option<PromptCacheOptions>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    prediction: Option<Prediction>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<TraceOptions>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<ProviderPreferences>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    plugins: Option<Vec<Plugin>>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    modalities: Option<Vec<Modality>>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    image_config: Option<HashMap<String, Value>>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<ReasoningConfig>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    include_reasoning: Option<bool>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<StopSequence>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    debug: Option<DebugOptions>,

    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ChatTool>>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<crate::types::ToolChoice>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,
    /// Portable deferred-tool discovery configuration; validated by the server.
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    deferred_tools: Option<Value>,
}

impl ChatCompletionRequestBuilder {
    strip_option_vec_setter!(models, String);
    strip_option_map_setter!(logit_bias, String, f64);
    strip_option_vec_setter!(transforms, String);
    strip_option_map_setter!(metadata, String, String);
    strip_option_map_setter!(image_config, String, Value);
    strip_option_vec_setter!(plugins, Plugin);
    strip_option_vec_setter!(modalities, Modality);
    /// Replace the request's function tools, keeping any server tools.
    ///
    /// Function and server tools were independent lists before they were merged
    /// into one field, so replacing one kind must not drop the other.
    pub fn tools<T, S>(&mut self, tools: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<crate::types::Tool>,
    {
        let servers = self.tools.take().flatten().unwrap_or_default();
        self.tools = Some(Some(
            servers
                .into_iter()
                .filter(|tool| tool.as_server().is_some())
                .chain(
                    tools
                        .into_iter()
                        .map(|tool| ChatTool::Function(tool.into())),
                )
                .collect(),
        ));
        self
    }

    /// Replace the request's server tools, keeping any function tools.
    pub fn server_tools<T, S>(&mut self, tools: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<crate::types::ServerTool>,
    {
        let functions = self
            .tools
            .take()
            .flatten()
            .unwrap_or_default()
            .into_iter()
            .filter(|tool| tool.as_function().is_some());
        self.tools = Some(Some(
            functions
                .chain(tools.into_iter().map(|tool| ChatTool::Server(tool.into())))
                .collect(),
        ));
        self
    }

    /// Enable reasoning with the provider's default settings.
    pub fn enable_reasoning(&mut self) -> &mut Self {
        self.reasoning_config().enabled = Some(true);
        self
    }

    /// Set the reasoning effort level.
    ///
    /// Clears any budget set by [`Self::reasoning_max_tokens`]; the API accepts
    /// only one of `effort` and `max_tokens` per request.
    pub fn reasoning_effort(&mut self, effort: crate::types::Effort) -> &mut Self {
        let config = self.reasoning_config();
        config.effort = Some(effort);
        config.max_tokens = None;
        self
    }

    /// Limit the reasoning token budget.
    ///
    /// Clears any level set by [`Self::reasoning_effort`]; the API accepts only
    /// one of `effort` and `max_tokens` per request.
    pub fn reasoning_max_tokens(&mut self, max_tokens: u32) -> &mut Self {
        let config = self.reasoning_config();
        config.max_tokens = Some(max_tokens);
        config.effort = None;
        self
    }

    /// Exclude reasoning from the response (reason internally, return only the answer).
    pub fn exclude_reasoning(&mut self) -> &mut Self {
        self.reasoning_config().exclude = Some(true);
        self
    }

    /// The reasoning configuration, created on first use.
    ///
    /// The setters mutate this value rather than replacing it, so `enabled` and
    /// `exclude` compose with either budget selector.
    fn reasoning_config(&mut self) -> &mut crate::types::ReasoningConfig {
        self.reasoning
            .get_or_insert_with(|| Some(crate::types::ReasoningConfig::default()))
            .get_or_insert_with(crate::types::ReasoningConfig::default)
    }

    /// Add a single tool to the request
    pub fn tool(&mut self, tool: crate::types::Tool) -> &mut Self {
        self.tools = Some(Some(
            self.tools
                .take()
                .flatten()
                .unwrap_or_default()
                .into_iter()
                .chain([ChatTool::Function(tool)])
                .collect(),
        ));
        self
    }

    /// Add a single OpenRouter server tool to the request.
    pub fn server_tool(&mut self, tool: crate::types::ServerTool) -> &mut Self {
        self.tools = Some(Some(
            self.tools
                .take()
                .flatten()
                .unwrap_or_default()
                .into_iter()
                .chain([ChatTool::Server(tool)])
                .collect(),
        ));
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
    pub fn tools(&self) -> impl Iterator<Item = &ChatTool> {
        self.tools.iter().flatten()
    }

    /// Iterate over the caller-defined function tools in this request.
    pub fn function_tools(&self) -> impl Iterator<Item = &crate::types::Tool> {
        self.tools().filter_map(ChatTool::as_function)
    }

    /// Get the OpenRouter server tools defined in this request.
    pub fn server_tools(&self) -> impl Iterator<Item = &crate::types::ServerTool> {
        self.tools().filter_map(ChatTool::as_server)
    }

    pub(crate) fn requires_openrouter_files_tool_header(&self) -> bool {
        self.server_tools()
            .any(crate::types::ServerTool::is_files_tool)
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

pub(crate) async fn send_chat_completion_with_client(
    ctx: &RequestContext<'_>,
    request: &ChatCompletionRequest,
) -> Result<CompletionsResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
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

    let req = request_builder.json(&request);
    transport_response::send_json(req, "chat completion").await
}

/// Parses one SSE payload from a chat stream.
///
/// A provider can fail after the response headers were sent, in which case the
/// stream carries an API error payload instead of a chunk. Surfacing that as
/// [`OpenRouterError::Api`] preserves the upstream code and message instead of
/// reporting a deserialization failure.
fn parse_stream_frame(
    status: http::StatusCode,
    data: &str,
) -> Result<CompletionsResponse, OpenRouterError> {
    serde_json::from_str::<CompletionsResponse>(data).map_err(|error| {
        crate::api::errors::try_parse_api_error(status, None, data)
            .unwrap_or(OpenRouterError::Serialization(error))
    })
}

pub(crate) async fn stream_chat_completion_with_client(
    ctx: &RequestContext<'_>,
    request: &ChatCompletionRequest,
) -> Result<BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
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
    let status = response.status();

    if status.is_success() {
        let lines = parse_sse_frames(response_lines(response))
            .filter_map(move |line| async move {
                match line {
                    Ok(frame) if frame.data == "[DONE]" => None,
                    Ok(frame) => Some(parse_stream_frame(status, &frame.data)),
                    Err(error) => Some(Err(error)),
                }
            })
            .boxed();

        Ok(lines)
    } else {
        Err(transport_response::error_from_response(response).await)
    }
}
