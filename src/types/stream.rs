//! # Unified streaming events
//!
//! [`UnifiedStreamEvent`] is the single event model emitted by chat, Responses,
//! and Anthropic-compatible Messages streams, plus the adapters that project
//! each source onto it.
//!
//! Chat streams assemble partial tool-call fragments into complete
//! [`ToolCall`] objects, delivered in the terminal [`UnifiedStreamEvent::Done`]
//! event.
//!
//! ```rust,no_run
//! use futures_util::StreamExt;
//! use openrouter_rs::types::stream::UnifiedStreamEvent;
//!
//! # async fn example(client: openrouter_rs::OpenRouterClient, request: openrouter_rs::api::chat::ChatCompletionRequest) -> Result<(), Box<dyn std::error::Error>> {
//! let mut stream = client.chat().stream_unified(&request).await?;
//!
//! while let Some(event) = stream.next().await {
//!     match event {
//!         UnifiedStreamEvent::ContentDelta(text) => print!("{text}"),
//!         UnifiedStreamEvent::Done { tool_calls, .. } => {
//!             for call in &tool_calls {
//!                 println!("Tool: {} args: {}", call.name(), call.arguments_json());
//!             }
//!         }
//!         UnifiedStreamEvent::Error(error) => eprintln!("Error: {error}"),
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::collections::{BTreeMap, VecDeque};

use futures_util::StreamExt;
use futures_util::stream::BoxStream;
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

/// Internal accumulator for a single tool call being assembled from
/// streaming fragments.
#[derive(Debug, Clone, Default)]
struct ToolCallAccumulator {
    index: Option<u32>,
    id: Option<String>,
    type_: Option<String>,
    name: Option<String>,
    arguments: String,
}

impl ToolCallAccumulator {
    /// Merge a partial tool call fragment into this accumulator.
    fn merge(&mut self, partial: &PartialToolCall) {
        if partial.index.is_some() {
            self.index = partial.index;
        }
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
            index: self.index,
        })
    }
}

/// Source stream family for a [`UnifiedStreamEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnifiedStreamSource {
    Chat,
    Responses,
    Messages,
}

/// Unified stream event model across chat/responses/messages APIs.
///
/// The terminal variant is larger than a delta, but a stream yields it once and
/// events are consumed one at a time, so boxing it would cost an allocation per
/// stream for no practical benefit.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
#[non_exhaustive]
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
        /// Fully assembled tool calls. Empty when the source requested none or
        /// does not expose tool calls.
        tool_calls: Vec<ToolCall>,
        /// Token usage when the source reports it in a shape the SDK can map.
        usage: Option<ResponseUsage>,
    },
    /// Transport/parsing/runtime error from the underlying stream.
    Error(OpenRouterError),
}

/// Projects a source's token counts onto the shared usage model.
///
/// Chat uses `prompt_tokens`/`completion_tokens`, while the Responses and
/// Messages APIs use `input_tokens`/`output_tokens`; neither non-chat shape
/// deserializes into [`ResponseUsage`] directly.
fn usage_from_token_counts(
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    total_tokens: Option<u64>,
) -> Option<ResponseUsage> {
    let prompt_tokens = u32::try_from(input_tokens?).ok()?;
    let completion_tokens = u32::try_from(output_tokens?).ok()?;
    let total_tokens = total_tokens
        .and_then(|total| u32::try_from(total).ok())
        .unwrap_or_else(|| prompt_tokens.saturating_add(completion_tokens));
    Some(ResponseUsage::new(
        prompt_tokens,
        completion_tokens,
        total_tokens,
    ))
}

/// A unified stream type across all streaming APIs.
pub type UnifiedStream = BoxStream<'static, UnifiedStreamEvent>;

#[derive(Debug, Default)]
struct StreamMeta {
    id: Option<String>,
    model: Option<String>,
    finish_reason: Option<String>,
    usage: Option<ResponseUsage>,
    tool_accumulators: BTreeMap<u32, ToolCallAccumulator>,
}

impl StreamMeta {
    /// Assembles accumulated tool-call fragments into complete calls.
    fn take_tool_calls(&mut self) -> Vec<ToolCall> {
        std::mem::take(&mut self.tool_accumulators)
            .into_values()
            .filter_map(ToolCallAccumulator::into_tool_call)
            .collect()
    }
}

fn finish_reason_to_string(reason: &FinishReason) -> String {
    match reason {
        FinishReason::ToolCalls => "tool_calls".to_string(),
        FinishReason::Stop => "stop".to_string(),
        FinishReason::Length => "length".to_string(),
        FinishReason::ContentFilter => "content_filter".to_string(),
        FinishReason::Error => "error".to_string(),
        FinishReason::Other(value) => value.clone(),
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
                        state.meta.usage = Some(usage);
                    }

                    for choice in &response.choices {
                        if let Some(content) = choice.content()
                            && !content.is_empty()
                        {
                            state
                                .pending
                                .push_back(UnifiedStreamEvent::ContentDelta(content.to_string()));
                        }

                        if let Some(reasoning) = choice.reasoning()
                            && !reasoning.is_empty()
                        {
                            state.pending.push_back(UnifiedStreamEvent::ReasoningDelta(
                                reasoning.to_string(),
                            ));
                        }

                        if let Some(reasoning_details) = choice.reasoning_details()
                            && !reasoning_details.is_empty()
                        {
                            state
                                .pending
                                .push_back(UnifiedStreamEvent::ReasoningDetailsDelta(
                                    reasoning_details.to_vec(),
                                ));
                        }

                        if let Some(partials) = choice.partial_tool_calls() {
                            for partial in partials {
                                state
                                    .meta
                                    .tool_accumulators
                                    .entry(partial.index.unwrap_or(0))
                                    .or_default()
                                    .merge(partial);
                                state.pending.push_back(UnifiedStreamEvent::ToolDelta(
                                    serde_json::to_value(partial).unwrap_or(Value::Null),
                                ));
                            }
                        }

                        if let Some(reason) = choice.finish_reason() {
                            state.meta.finish_reason = Some(finish_reason_to_string(reason));
                        }
                    }
                }
                Some(Err(error)) => {
                    state.pending.push_back(UnifiedStreamEvent::Error(error));
                }
                None => {
                    state.done_emitted = true;
                    let tool_calls = state.meta.take_tool_calls();
                    state.pending.push_back(UnifiedStreamEvent::Done {
                        source: UnifiedStreamSource::Chat,
                        id: state.meta.id.take(),
                        model: state.meta.model.take(),
                        finish_reason: state.meta.finish_reason.take(),
                        tool_calls,
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
                            state.meta.usage = usage_from_token_counts(
                                usage.get("input_tokens").and_then(Value::as_u64),
                                usage.get("output_tokens").and_then(Value::as_u64),
                                usage.get("total_tokens").and_then(Value::as_u64),
                            );
                        }
                    }

                    if event_type.contains("output_text.delta")
                        && let Some(delta) = event.data.get("delta").and_then(Value::as_str)
                    {
                        state
                            .pending
                            .push_back(UnifiedStreamEvent::ContentDelta(delta.to_string()));
                        emitted = true;
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

                    if event_type == "response.completed" {
                        state.done_emitted = true;
                        state.pending.push_back(UnifiedStreamEvent::Done {
                            source: UnifiedStreamSource::Responses,
                            id: state.meta.id.take(),
                            model: state.meta.model.take(),
                            finish_reason: state.meta.finish_reason.take(),
                            tool_calls: Vec::new(),
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
                        tool_calls: Vec::new(),
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
                                state.meta.usage = usage_from_token_counts(
                                    usage.input_tokens,
                                    usage.output_tokens,
                                    None,
                                );
                            }
                        }
                        AnthropicMessagesStreamEvent::MessageDelta { delta, usage } => {
                            state.meta.usage = serde_json::from_value::<
                                crate::api::messages::AnthropicMessagesUsage,
                            >(usage)
                            .ok()
                            .and_then(|usage| {
                                usage_from_token_counts(
                                    usage.input_tokens,
                                    usage.output_tokens,
                                    None,
                                )
                            });
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
                            index,
                            content_block,
                        } => match *content_block {
                            AnthropicContentPart::Thinking { thinking, .. } => {
                                state
                                    .pending
                                    .push_back(UnifiedStreamEvent::ReasoningDelta(thinking));
                            }
                            AnthropicContentPart::ToolUse { .. }
                            | AnthropicContentPart::ServerToolUse { .. } => {
                                let content_block_value =
                                    serde_json::to_value(content_block).unwrap_or(Value::Null);
                                state.pending.push_back(UnifiedStreamEvent::ToolDelta(
                                    serde_json::json!({
                                        "index": index,
                                        "content_block": content_block_value,
                                    }),
                                ));
                            }
                            _ => {}
                        },
                        AnthropicMessagesStreamEvent::ContentBlockDelta { index, delta } => {
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
                                state.pending.push_back(UnifiedStreamEvent::ToolDelta(
                                    serde_json::json!({
                                        "index": index,
                                        "delta": delta
                                    }),
                                ));
                            } else {
                                state.pending.push_back(UnifiedStreamEvent::Raw {
                                    source: UnifiedStreamSource::Messages,
                                    event_type: event_name,
                                    data: delta,
                                });
                            }
                        }
                        AnthropicMessagesStreamEvent::MessageStop { .. } => {
                            state.done_emitted = true;
                            state.pending.push_back(UnifiedStreamEvent::Done {
                                source: UnifiedStreamSource::Messages,
                                id: state.meta.id.take(),
                                model: state.meta.model.take(),
                                finish_reason: state.meta.finish_reason.take(),
                                tool_calls: Vec::new(),
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
                        tool_calls: Vec::new(),
                        usage: state.meta.usage.take(),
                    });
                }
            }
        }
    })
    .boxed()
}
