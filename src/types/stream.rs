//! # Tool-Aware Streaming
//!
//! This module provides [`ToolAwareStream`], a wrapper around the raw SSE
//! stream that automatically accumulates partial tool call fragments into
//! complete [`ToolCall`] objects while still yielding text and reasoning
//! content deltas in real time.
//!
//! ## Problem
//!
//! When the OpenRouter API streams a response that includes tool calls,
//! the tool call data arrives incrementally across many SSE chunks:
//!
//! - Chunk 1: `{index: 0, id: "call_abc", type: "function", function: {name: "get_weather", arguments: ""}}`
//! - Chunk 2: `{index: 0, function: {arguments: "{\"loc"}}`
//! - Chunk 3: `{index: 0, function: {arguments: "ation\":"}}`
//! - Chunk N: `{index: 0, function: {arguments: " \"NYC\"}"}}`
//!
//! The raw stream yields these as [`PartialToolCall`] fragments that cannot
//! be used directly. `ToolAwareStream` handles merging them by `index`.
//!
//! ## Solution
//!
//! Wrap the raw stream in a `ToolAwareStream` to get a stream of
//! [`StreamEvent`] values:
//!
//! ```rust,no_run
//! use futures_util::StreamExt;
//! use openrouter_rs::types::stream::{ToolAwareStream, StreamEvent};
//!
//! # async fn example(client: openrouter_rs::OpenRouterClient, request: openrouter_rs::api::chat::ChatCompletionRequest) -> Result<(), Box<dyn std::error::Error>> {
//! let raw_stream = client.chat().stream(&request).await?;
//! let mut stream = ToolAwareStream::new(raw_stream);
//!
//! while let Some(event) = stream.next().await {
//!     match event {
//!         StreamEvent::ContentDelta(text) => print!("{}", text),
//!         StreamEvent::ReasoningDelta(text) => { /* reasoning content */ },
//!         StreamEvent::Done { tool_calls, .. } => {
//!             for tc in &tool_calls {
//!                 println!("Tool: {} args: {}", tc.name(), tc.arguments_json());
//!             }
//!         },
//!         StreamEvent::Error(e) => eprintln!("Error: {}", e),
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::collections::{BTreeMap, VecDeque};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::stream::BoxStream;
use futures_util::{Stream, StreamExt};
use serde_json::Value;

use crate::error::OpenRouterError;
use crate::types::completion::{
    CompletionsResponse, FunctionCall, PartialToolCall, ReasoningDetail, ResponseUsage, ToolCall,
};
use crate::{
    api::{
        messages::{AnthropicContentPart, AnthropicMessagesSseEvent, AnthropicMessagesStreamEvent},
        responses::ResponsesStreamEvent,
    },
    types::completion::FinishReason,
};

/// Events emitted by [`ToolAwareStream`].
///
/// Content and reasoning deltas are yielded immediately as they arrive.
/// Tool calls are accumulated internally and emitted as complete objects
/// only once in the final [`StreamEvent::Done`] event.
#[derive(Debug)]
pub enum StreamEvent {
    /// A fragment of text content from the assistant's response.
    ContentDelta(String),

    /// A fragment of reasoning/chain-of-thought content.
    ReasoningDelta(String),

    /// Structured reasoning detail blocks (e.g., encrypted reasoning).
    ReasoningDetailsDelta(Vec<ReasoningDetail>),

    /// The stream has finished. Contains all accumulated data.
    ///
    /// `tool_calls` will be empty if the model did not invoke any tools.
    /// `usage` is typically only present in the final SSE chunk.
    Done {
        /// Fully assembled tool calls (empty if none were requested).
        tool_calls: Vec<ToolCall>,
        /// The reason the model stopped generating.
        finish_reason: Option<FinishReason>,
        /// Token usage statistics (if provided by the API).
        usage: Option<ResponseUsage>,
        /// The response ID from the API.
        id: String,
        /// The model that generated the response.
        model: String,
    },

    /// An error occurred while processing the stream.
    Error(OpenRouterError),
}

/// Internal accumulator for a single tool call being assembled from
/// streaming fragments.
#[derive(Debug, Clone, Default)]
struct ToolCallAccumulator {
    id: Option<String>,
    type_: Option<String>,
    name: Option<String>,
    arguments: String,
}

impl ToolCallAccumulator {
    /// Merge a partial tool call fragment into this accumulator.
    fn merge(&mut self, partial: &PartialToolCall) {
        if let Some(id) = &partial.id {
            self.id = Some(id.clone());
        }
        if let Some(type_) = &partial.type_ {
            self.type_ = Some(type_.clone());
        }
        if let Some(func) = &partial.function {
            if let Some(name) = &func.name {
                self.name = Some(name.clone());
            }
            if let Some(args) = &func.arguments {
                self.arguments.push_str(args);
            }
        }
    }

    /// Try to convert this accumulator into a complete [`ToolCall`].
    ///
    /// Returns `None` if required fields (`id`, `name`) are still missing,
    /// which would indicate an incomplete stream.
    fn into_tool_call(self) -> Option<ToolCall> {
        Some(ToolCall {
            id: self.id?,
            type_: self.type_.unwrap_or_else(|| "function".to_string()),
            function: FunctionCall {
                name: self.name?,
                arguments: self.arguments,
            },
            index: None,
        })
    }
}

/// A stream wrapper that accumulates partial tool call fragments and
/// yields [`StreamEvent`] values.
///
/// Text content and reasoning deltas are forwarded immediately. Tool call
/// chunks are buffered internally and assembled into complete [`ToolCall`]
/// objects, which are emitted in the final [`StreamEvent::Done`] event.
///
/// # Construction
///
/// Wrap any raw streaming response from
/// [`stream_chat_completion`](crate::api::chat::stream_chat_completion):
///
/// ```rust,no_run
/// # async fn example(client: openrouter_rs::OpenRouterClient, request: openrouter_rs::api::chat::ChatCompletionRequest) -> Result<(), Box<dyn std::error::Error>> {
/// use openrouter_rs::types::stream::ToolAwareStream;
///
/// let raw = client.chat().stream(&request).await?;
/// let stream = ToolAwareStream::new(raw);
/// # Ok(())
/// # }
/// ```
///
/// Or use the convenience method on the client:
///
/// ```rust,no_run
/// # async fn example(client: openrouter_rs::OpenRouterClient, request: openrouter_rs::api::chat::ChatCompletionRequest) -> Result<(), Box<dyn std::error::Error>> {
/// let stream = client.stream_chat_completion_tool_aware(&request).await?;
/// # Ok(())
/// # }
/// ```
pub struct ToolAwareStream {
    inner: BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>,
    /// Tool call fragments accumulated by tool call index.
    tool_accumulators: BTreeMap<u32, ToolCallAccumulator>,
    /// Buffered events ready to be yielded.
    pending_events: Vec<StreamEvent>,
    /// Last seen response ID.
    last_id: String,
    /// Last seen model name.
    last_model: String,
    /// Last seen usage stats.
    last_usage: Option<ResponseUsage>,
    /// Last seen finish reason.
    last_finish_reason: Option<FinishReason>,
    /// Whether the stream has completed.
    finished: bool,
}

impl ToolAwareStream {
    /// Create a new `ToolAwareStream` wrapping a raw SSE stream.
    pub fn new(inner: BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>) -> Self {
        Self {
            inner,
            tool_accumulators: BTreeMap::new(),
            pending_events: Vec::new(),
            last_id: String::new(),
            last_model: String::new(),
            last_usage: None,
            last_finish_reason: None,
            finished: false,
        }
    }

    /// Process a single `CompletionsResponse` chunk, extracting events
    /// and accumulating tool call fragments.
    fn process_chunk(&mut self, response: CompletionsResponse) {
        // Track metadata from every chunk
        self.last_id.clone_from(&response.id);
        self.last_model.clone_from(&response.model);
        if response.usage.is_some() {
            self.last_usage = response.usage;
        }

        for choice in &response.choices {
            // Track finish reason
            if let Some(reason) = choice.finish_reason() {
                self.last_finish_reason = Some(reason.clone());
            }

            // Extract content delta
            if let Some(content) = choice.content() {
                if !content.is_empty() {
                    self.pending_events
                        .push(StreamEvent::ContentDelta(content.to_string()));
                }
            }

            // Extract reasoning delta
            if let Some(reasoning) = choice.reasoning() {
                if !reasoning.is_empty() {
                    self.pending_events
                        .push(StreamEvent::ReasoningDelta(reasoning.to_string()));
                }
            }

            // Extract reasoning details
            if let Some(details) = choice.reasoning_details() {
                if !details.is_empty() {
                    self.pending_events
                        .push(StreamEvent::ReasoningDetailsDelta(details.to_vec()));
                }
            }

            // Accumulate partial tool calls
            if let Some(partial_tool_calls) = choice.partial_tool_calls() {
                for partial in partial_tool_calls {
                    // Use the index field to identify which tool call this
                    // fragment belongs to. Default to 0 if not specified.
                    let idx = partial.index.unwrap_or(0);
                    let acc = self.tool_accumulators.entry(idx).or_default();
                    acc.merge(partial);
                }
            }
        }
    }

    /// Finalize the stream: assemble complete tool calls and emit `Done`.
    fn finalize(&mut self) {
        let tool_calls: Vec<ToolCall> = self
            .tool_accumulators
            .values()
            .cloned()
            .filter_map(|acc| acc.into_tool_call())
            .collect();

        self.pending_events.push(StreamEvent::Done {
            tool_calls,
            finish_reason: self.last_finish_reason.take(),
            usage: self.last_usage.take(),
            id: std::mem::take(&mut self.last_id),
            model: std::mem::take(&mut self.last_model),
        });

        self.finished = true;
    }
}

impl Stream for ToolAwareStream {
    type Item = StreamEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Drain any buffered events first
        if !self.pending_events.is_empty() {
            return Poll::Ready(Some(self.pending_events.remove(0)));
        }

        if self.finished {
            return Poll::Ready(None);
        }

        // Poll the inner stream for the next chunk
        match self.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(response))) => {
                self.process_chunk(response);

                // Return the first pending event if any
                if !self.pending_events.is_empty() {
                    Poll::Ready(Some(self.pending_events.remove(0)))
                } else {
                    // No events from this chunk (e.g., empty delta), poll again
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(StreamEvent::Error(e))),
            Poll::Ready(None) => {
                // Inner stream ended -- emit Done with accumulated tool calls
                if !self.finished {
                    self.finalize();
                    // Return the Done event
                    if !self.pending_events.is_empty() {
                        Poll::Ready(Some(self.pending_events.remove(0)))
                    } else {
                        Poll::Ready(None)
                    }
                } else {
                    Poll::Ready(None)
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Source stream family for a [`UnifiedStreamEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnifiedStreamSource {
    Chat,
    Responses,
    Messages,
}

/// Unified stream event model across chat/responses/messages APIs.
#[derive(Debug)]
pub enum UnifiedStreamEvent {
    /// Text content delta from the model.
    ContentDelta(String),
    /// Reasoning/thinking delta.
    ReasoningDelta(String),
    /// Structured reasoning detail blocks (chat stream only).
    ReasoningDetailsDelta(Vec<ReasoningDetail>),
    /// Tool-related delta payload (format depends on source API).
    ToolDelta(Value),
    /// Source-specific event payload when no common projection applies.
    Raw {
        source: UnifiedStreamSource,
        event_type: String,
        data: Value,
    },
    /// Terminal event for a stream source.
    Done {
        source: UnifiedStreamSource,
        id: Option<String>,
        model: Option<String>,
        finish_reason: Option<String>,
        usage: Option<Value>,
    },
    /// Transport/parsing/runtime error from the underlying stream.
    Error(OpenRouterError),
}

/// A unified stream type across all streaming APIs.
pub type UnifiedStream = BoxStream<'static, UnifiedStreamEvent>;

#[derive(Debug, Default)]
struct StreamMeta {
    id: Option<String>,
    model: Option<String>,
    finish_reason: Option<String>,
    usage: Option<Value>,
}

fn finish_reason_to_string(reason: &FinishReason) -> &'static str {
    match reason {
        FinishReason::ToolCalls => "tool_calls",
        FinishReason::Stop => "stop",
        FinishReason::Length => "length",
        FinishReason::ContentFilter => "content_filter",
        FinishReason::Error => "error",
    }
}

/// Adapt a chat-completions SSE stream to [`UnifiedStreamEvent`].
pub fn adapt_chat_stream(
    inner: BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>,
) -> UnifiedStream {
    struct State {
        inner: BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>,
        pending: VecDeque<UnifiedStreamEvent>,
        done_emitted: bool,
        meta: StreamMeta,
    }

    let state = State {
        inner,
        pending: VecDeque::new(),
        done_emitted: false,
        meta: StreamMeta::default(),
    };

    futures_util::stream::unfold(state, |mut state| async move {
        loop {
            if let Some(event) = state.pending.pop_front() {
                return Some((event, state));
            }

            if state.done_emitted {
                return None;
            }

            match state.inner.next().await {
                Some(Ok(response)) => {
                    state.meta.id = Some(response.id.clone());
                    state.meta.model = Some(response.model.clone());
                    if let Some(usage) = response.usage {
                        state.meta.usage = serde_json::to_value(usage).ok();
                    }

                    for choice in &response.choices {
                        if let Some(content) = choice.content() {
                            if !content.is_empty() {
                                state.pending.push_back(UnifiedStreamEvent::ContentDelta(
                                    content.to_string(),
                                ));
                            }
                        }

                        if let Some(reasoning) = choice.reasoning() {
                            if !reasoning.is_empty() {
                                state.pending.push_back(UnifiedStreamEvent::ReasoningDelta(
                                    reasoning.to_string(),
                                ));
                            }
                        }

                        if let Some(reasoning_details) = choice.reasoning_details() {
                            if !reasoning_details.is_empty() {
                                state
                                    .pending
                                    .push_back(UnifiedStreamEvent::ReasoningDetailsDelta(
                                        reasoning_details.to_vec(),
                                    ));
                            }
                        }

                        if let Some(partials) = choice.partial_tool_calls() {
                            for partial in partials {
                                state.pending.push_back(UnifiedStreamEvent::ToolDelta(
                                    serde_json::to_value(partial).unwrap_or(Value::Null),
                                ));
                            }
                        }

                        if let Some(reason) = choice.finish_reason() {
                            state.meta.finish_reason =
                                Some(finish_reason_to_string(reason).to_string());
                        }
                    }
                }
                Some(Err(error)) => {
                    state.pending.push_back(UnifiedStreamEvent::Error(error));
                }
                None => {
                    state.done_emitted = true;
                    state.pending.push_back(UnifiedStreamEvent::Done {
                        source: UnifiedStreamSource::Chat,
                        id: state.meta.id.take(),
                        model: state.meta.model.take(),
                        finish_reason: state.meta.finish_reason.take(),
                        usage: state.meta.usage.take(),
                    });
                }
            }
        }
    })
    .boxed()
}

/// Adapt a Responses API SSE stream to [`UnifiedStreamEvent`].
pub fn adapt_responses_stream(
    inner: BoxStream<'static, Result<ResponsesStreamEvent, OpenRouterError>>,
) -> UnifiedStream {
    struct State {
        inner: BoxStream<'static, Result<ResponsesStreamEvent, OpenRouterError>>,
        pending: VecDeque<UnifiedStreamEvent>,
        done_emitted: bool,
        meta: StreamMeta,
    }

    let state = State {
        inner,
        pending: VecDeque::new(),
        done_emitted: false,
        meta: StreamMeta::default(),
    };

    futures_util::stream::unfold(state, |mut state| async move {
        loop {
            if let Some(event) = state.pending.pop_front() {
                return Some((event, state));
            }

            if state.done_emitted {
                return None;
            }

            match state.inner.next().await {
                Some(Ok(event)) => {
                    let event_type = event.event_type.clone();
                    let data_value = serde_json::to_value(&event.data).unwrap_or(Value::Null);
                    let mut emitted = false;

                    if let Some(response) = event.data.get("response") {
                        if let Some(id) = response.get("id").and_then(Value::as_str) {
                            state.meta.id = Some(id.to_string());
                        }
                        if let Some(model) = response.get("model").and_then(Value::as_str) {
                            state.meta.model = Some(model.to_string());
                        }
                        if let Some(status) = response.get("status").and_then(Value::as_str) {
                            state.meta.finish_reason = Some(status.to_string());
                        }
                        if let Some(usage) = response.get("usage") {
                            state.meta.usage = Some(usage.clone());
                        }
                    }

                    if event_type.contains("output_text.delta") {
                        if let Some(delta) = event.data.get("delta").and_then(Value::as_str) {
                            state
                                .pending
                                .push_back(UnifiedStreamEvent::ContentDelta(delta.to_string()));
                            emitted = true;
                        }
                    }

                    if !emitted && event_type.contains("reasoning") {
                        let reasoning = event
                            .data
                            .get("delta")
                            .and_then(Value::as_str)
                            .or_else(|| event.data.get("text").and_then(Value::as_str))
                            .or_else(|| event.data.get("reasoning").and_then(Value::as_str));
                        if let Some(reasoning) = reasoning {
                            state.pending.push_back(UnifiedStreamEvent::ReasoningDelta(
                                reasoning.to_string(),
                            ));
                            emitted = true;
                        }
                    }

                    if !emitted && event_type.contains("tool") {
                        state
                            .pending
                            .push_back(UnifiedStreamEvent::ToolDelta(data_value.clone()));
                        emitted = true;
                    }

                    if event_type == "response.completed" || event_type.ends_with(".completed") {
                        state.done_emitted = true;
                        state.pending.push_back(UnifiedStreamEvent::Done {
                            source: UnifiedStreamSource::Responses,
                            id: state.meta.id.take(),
                            model: state.meta.model.take(),
                            finish_reason: state.meta.finish_reason.take(),
                            usage: state.meta.usage.take(),
                        });
                        continue;
                    }

                    if !emitted {
                        state.pending.push_back(UnifiedStreamEvent::Raw {
                            source: UnifiedStreamSource::Responses,
                            event_type,
                            data: data_value,
                        });
                    }
                }
                Some(Err(error)) => {
                    state.pending.push_back(UnifiedStreamEvent::Error(error));
                }
                None => {
                    state.done_emitted = true;
                    state.pending.push_back(UnifiedStreamEvent::Done {
                        source: UnifiedStreamSource::Responses,
                        id: state.meta.id.take(),
                        model: state.meta.model.take(),
                        finish_reason: state.meta.finish_reason.take(),
                        usage: state.meta.usage.take(),
                    });
                }
            }
        }
    })
    .boxed()
}

/// Adapt a Messages API SSE stream to [`UnifiedStreamEvent`].
pub fn adapt_messages_stream(
    inner: BoxStream<'static, Result<AnthropicMessagesSseEvent, OpenRouterError>>,
) -> UnifiedStream {
    struct State {
        inner: BoxStream<'static, Result<AnthropicMessagesSseEvent, OpenRouterError>>,
        pending: VecDeque<UnifiedStreamEvent>,
        done_emitted: bool,
        meta: StreamMeta,
    }

    let state = State {
        inner,
        pending: VecDeque::new(),
        done_emitted: false,
        meta: StreamMeta::default(),
    };

    futures_util::stream::unfold(state, |mut state| async move {
        loop {
            if let Some(event) = state.pending.pop_front() {
                return Some((event, state));
            }

            if state.done_emitted {
                return None;
            }

            match state.inner.next().await {
                Some(Ok(event)) => {
                    let event_name = event.event.clone();
                    match event.data {
                        AnthropicMessagesStreamEvent::MessageStart { message } => {
                            state.meta.id = message.id.clone();
                            state.meta.model = message.model.clone();
                            if let Some(usage) = message.usage {
                                state.meta.usage = serde_json::to_value(usage).ok();
                            }
                        }
                        AnthropicMessagesStreamEvent::MessageDelta { delta, usage } => {
                            state.meta.usage = Some(usage);
                            if let Some(reason) = delta.get("stop_reason").and_then(Value::as_str) {
                                state.meta.finish_reason = Some(reason.to_string());
                            }
                            let text = delta
                                .get("text")
                                .and_then(Value::as_str)
                                .or_else(|| delta.get("output_text").and_then(Value::as_str));
                            if let Some(text) = text {
                                state
                                    .pending
                                    .push_back(UnifiedStreamEvent::ContentDelta(text.to_string()));
                            }
                        }
                        AnthropicMessagesStreamEvent::ContentBlockStart {
                            content_block, ..
                        } => match *content_block {
                            AnthropicContentPart::Thinking { thinking, .. } => {
                                state
                                    .pending
                                    .push_back(UnifiedStreamEvent::ReasoningDelta(thinking));
                            }
                            AnthropicContentPart::ToolUse { .. }
                            | AnthropicContentPart::ServerToolUse { .. } => {
                                state.pending.push_back(UnifiedStreamEvent::ToolDelta(
                                    serde_json::to_value(content_block).unwrap_or(Value::Null),
                                ));
                            }
                            _ => {}
                        },
                        AnthropicMessagesStreamEvent::ContentBlockDelta { delta, .. } => {
                            let delta_type = delta
                                .get("type")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            if delta_type.contains("text_delta") {
                                if let Some(text) = delta.get("text").and_then(Value::as_str) {
                                    state.pending.push_back(UnifiedStreamEvent::ContentDelta(
                                        text.to_string(),
                                    ));
                                }
                            } else if delta_type.contains("thinking") {
                                let reasoning = delta
                                    .get("thinking")
                                    .and_then(Value::as_str)
                                    .or_else(|| delta.get("text").and_then(Value::as_str));
                                if let Some(reasoning) = reasoning {
                                    state.pending.push_back(UnifiedStreamEvent::ReasoningDelta(
                                        reasoning.to_string(),
                                    ));
                                }
                            } else if delta_type.contains("tool")
                                || delta_type.contains("json")
                                || delta.get("partial_json").is_some()
                            {
                                state
                                    .pending
                                    .push_back(UnifiedStreamEvent::ToolDelta(delta));
                            } else {
                                state.pending.push_back(UnifiedStreamEvent::Raw {
                                    source: UnifiedStreamSource::Messages,
                                    event_type: event_name,
                                    data: delta,
                                });
                            }
                        }
                        AnthropicMessagesStreamEvent::MessageStop => {
                            state.done_emitted = true;
                            state.pending.push_back(UnifiedStreamEvent::Done {
                                source: UnifiedStreamSource::Messages,
                                id: state.meta.id.take(),
                                model: state.meta.model.take(),
                                finish_reason: state.meta.finish_reason.take(),
                                usage: state.meta.usage.take(),
                            });
                        }
                        AnthropicMessagesStreamEvent::Error { error } => {
                            let message = error
                                .get("message")
                                .and_then(Value::as_str)
                                .map(ToOwned::to_owned)
                                .unwrap_or_else(|| error.to_string());
                            state.pending.push_back(UnifiedStreamEvent::Error(
                                OpenRouterError::Unknown(format!(
                                    "messages stream error event: {message}"
                                )),
                            ));
                        }
                        AnthropicMessagesStreamEvent::ContentBlockStop { .. }
                        | AnthropicMessagesStreamEvent::Ping => {}
                    }
                }
                Some(Err(error)) => {
                    state.pending.push_back(UnifiedStreamEvent::Error(error));
                }
                None => {
                    state.done_emitted = true;
                    state.pending.push_back(UnifiedStreamEvent::Done {
                        source: UnifiedStreamSource::Messages,
                        id: state.meta.id.take(),
                        model: state.meta.model.take(),
                        finish_reason: state.meta.finish_reason.take(),
                        usage: state.meta.usage.take(),
                    });
                }
            }
        }
    })
    .boxed()
}
