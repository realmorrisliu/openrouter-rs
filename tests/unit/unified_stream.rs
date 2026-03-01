use std::collections::HashMap;

use futures_util::{StreamExt, stream};
use openrouter_rs::{
    api::{
        messages::{
            AnthropicContentPart, AnthropicMessagesResponse, AnthropicMessagesSseEvent,
            AnthropicMessagesStreamEvent, AnthropicMessagesUsage,
        },
        responses::ResponsesStreamEvent,
    },
    error::OpenRouterError,
    types::{
        completion::{
            Choice, CompletionsResponse, Delta, FinishReason, ObjectType, PartialFunctionCall,
            PartialToolCall, ResponseUsage, StreamingChoice,
        },
        stream::{
            UnifiedStreamEvent, UnifiedStreamSource, adapt_chat_stream, adapt_messages_stream,
            adapt_responses_stream,
        },
    },
};
use serde_json::json;

fn chat_chunk(
    id: &str,
    model: &str,
    content: Option<&str>,
    reasoning: Option<&str>,
    partial_tool: Option<PartialToolCall>,
    finish_reason: Option<FinishReason>,
    usage: Option<ResponseUsage>,
) -> CompletionsResponse {
    CompletionsResponse {
        id: id.to_string(),
        choices: vec![Choice::Streaming(StreamingChoice {
            finish_reason,
            native_finish_reason: None,
            delta: Delta {
                content: content.map(ToOwned::to_owned),
                role: None,
                tool_calls: partial_tool.map(|p| vec![p]),
                reasoning: reasoning.map(ToOwned::to_owned),
                reasoning_details: None,
                audio: None,
                refusal: None,
            },
            error: None,
            index: Some(0),
            logprobs: None,
        })],
        created: 1_700_000_000,
        model: model.to_string(),
        object_type: ObjectType::ChatCompletionChunk,
        provider: None,
        system_fingerprint: None,
        usage,
    }
}

fn responses_event(event_type: &str, data: serde_json::Value) -> ResponsesStreamEvent {
    let mut map = HashMap::new();
    if let Some(obj) = data.as_object() {
        for (k, v) in obj {
            map.insert(k.clone(), v.clone());
        }
    }
    ResponsesStreamEvent {
        event_type: event_type.to_string(),
        sequence_number: None,
        data: map,
    }
}

#[tokio::test]
async fn test_unified_chat_stream_mixed_sequence() {
    let chunks = vec![
        Ok(chat_chunk(
            "gen_1",
            "test-model",
            Some("Hello "),
            Some("thinking"),
            Some(PartialToolCall {
                id: Some("call_1".to_string()),
                type_: Some("function".to_string()),
                function: Some(PartialFunctionCall {
                    name: Some("get_weather".to_string()),
                    arguments: Some("{\"location\":\"SF\"}".to_string()),
                }),
                index: Some(0),
            }),
            None,
            None,
        )),
        Ok(chat_chunk(
            "gen_1",
            "test-model",
            None,
            None,
            None,
            Some(FinishReason::Stop),
            Some(ResponseUsage {
                prompt_tokens: 5,
                completion_tokens: 7,
                total_tokens: 12,
            }),
        )),
    ];

    let mut stream = adapt_chat_stream(stream::iter(chunks).boxed());
    let events: Vec<UnifiedStreamEvent> = stream.by_ref().collect().await;

    assert_eq!(events.len(), 4);
    assert!(matches!(events[0], UnifiedStreamEvent::ContentDelta(_)));
    assert!(matches!(events[1], UnifiedStreamEvent::ReasoningDelta(_)));
    assert!(matches!(events[2], UnifiedStreamEvent::ToolDelta(_)));
    match &events[3] {
        UnifiedStreamEvent::Done {
            source,
            id,
            model,
            finish_reason,
            usage,
        } => {
            assert_eq!(*source, UnifiedStreamSource::Chat);
            assert_eq!(id.as_deref(), Some("gen_1"));
            assert_eq!(model.as_deref(), Some("test-model"));
            assert_eq!(finish_reason.as_deref(), Some("stop"));
            assert!(usage.is_some());
        }
        other => panic!("expected Done event, got {other:?}"),
    }
}

#[tokio::test]
async fn test_unified_chat_stream_error_propagation() {
    let chunks = vec![
        Ok(chat_chunk(
            "gen_2",
            "test-model",
            Some("hello"),
            None,
            None,
            None,
            None,
        )),
        Err(OpenRouterError::Unknown("stream-failed".to_string())),
    ];

    let mut stream = adapt_chat_stream(stream::iter(chunks).boxed());
    let events: Vec<UnifiedStreamEvent> = stream.by_ref().collect().await;

    assert_eq!(events.len(), 3);
    assert!(matches!(events[0], UnifiedStreamEvent::ContentDelta(_)));
    assert!(matches!(events[1], UnifiedStreamEvent::Error(_)));
    assert!(matches!(events[2], UnifiedStreamEvent::Done { .. }));
}

#[tokio::test]
async fn test_unified_responses_stream_mixed_sequence() {
    let events = vec![
        Ok(responses_event(
            "response.output_text.delta",
            json!({ "delta": "Hi" }),
        )),
        Ok(responses_event(
            "response.reasoning.delta",
            json!({ "delta": "step-by-step" }),
        )),
        Ok(responses_event(
            "response.output_tool_call.delta",
            json!({ "delta": { "arguments": "{\"city\":\"SF\"}" } }),
        )),
        Ok(responses_event(
            "response.completed",
            json!({
                "response": {
                    "id": "resp_1",
                    "model": "openai/gpt-5",
                    "status": "completed",
                    "usage": { "total_tokens": 10 }
                }
            }),
        )),
    ];

    let mut stream = adapt_responses_stream(stream::iter(events).boxed());
    let unified: Vec<UnifiedStreamEvent> = stream.by_ref().collect().await;

    assert_eq!(unified.len(), 4);
    assert!(matches!(unified[0], UnifiedStreamEvent::ContentDelta(_)));
    assert!(matches!(unified[1], UnifiedStreamEvent::ReasoningDelta(_)));
    assert!(matches!(unified[2], UnifiedStreamEvent::ToolDelta(_)));
    match &unified[3] {
        UnifiedStreamEvent::Done {
            source,
            id,
            model,
            finish_reason,
            usage,
        } => {
            assert_eq!(*source, UnifiedStreamSource::Responses);
            assert_eq!(id.as_deref(), Some("resp_1"));
            assert_eq!(model.as_deref(), Some("openai/gpt-5"));
            assert_eq!(finish_reason.as_deref(), Some("completed"));
            assert_eq!(
                usage
                    .as_ref()
                    .and_then(|u| u.get("total_tokens"))
                    .and_then(|v| v.as_u64()),
                Some(10)
            );
        }
        other => panic!("expected Done event, got {other:?}"),
    }
}

#[tokio::test]
async fn test_unified_responses_stream_non_terminal_completed_suffix_stays_open() {
    let events = vec![
        Ok(responses_event(
            "response.output_text.delta",
            json!({ "delta": "Hello " }),
        )),
        Ok(responses_event(
            "response.tool.completed",
            json!({ "tool": "get_weather" }),
        )),
        Ok(responses_event(
            "response.output_text.delta",
            json!({ "delta": "world" }),
        )),
        Ok(responses_event(
            "response.completed",
            json!({
                "response": {
                    "id": "resp_terminal",
                    "model": "openai/gpt-5",
                    "status": "completed"
                }
            }),
        )),
    ];

    let mut stream = adapt_responses_stream(stream::iter(events).boxed());
    let unified: Vec<UnifiedStreamEvent> = stream.by_ref().collect().await;

    assert_eq!(unified.len(), 4);
    assert!(matches!(unified[0], UnifiedStreamEvent::ContentDelta(_)));
    match &unified[1] {
        UnifiedStreamEvent::ToolDelta(data) => {
            assert_eq!(
                data.get("tool").and_then(|value| value.as_str()),
                Some("get_weather")
            );
        }
        other => panic!("expected ToolDelta event, got {other:?}"),
    }
    assert!(matches!(unified[2], UnifiedStreamEvent::ContentDelta(_)));
    match &unified[3] {
        UnifiedStreamEvent::Done {
            source,
            id,
            model,
            finish_reason,
            ..
        } => {
            assert_eq!(*source, UnifiedStreamSource::Responses);
            assert_eq!(id.as_deref(), Some("resp_terminal"));
            assert_eq!(model.as_deref(), Some("openai/gpt-5"));
            assert_eq!(finish_reason.as_deref(), Some("completed"));
        }
        other => panic!("expected Done event, got {other:?}"),
    }
}

#[tokio::test]
async fn test_unified_messages_stream_mixed_sequence() {
    let events = vec![
        Ok(AnthropicMessagesSseEvent {
            event: "message_start".to_string(),
            data: AnthropicMessagesStreamEvent::MessageStart {
                message: Box::new(AnthropicMessagesResponse {
                    id: Some("msg_1".to_string()),
                    object_type: Some("message".to_string()),
                    role: Some("assistant".to_string()),
                    content: Vec::new(),
                    model: Some("anthropic/claude-sonnet-4".to_string()),
                    stop_reason: None,
                    stop_sequence: None,
                    usage: Some(AnthropicMessagesUsage {
                        input_tokens: Some(5),
                        output_tokens: Some(0),
                        cache_creation_input_tokens: None,
                        cache_read_input_tokens: None,
                        service_tier: None,
                        extra: HashMap::new(),
                    }),
                    extra: HashMap::new(),
                }),
            },
        }),
        Ok(AnthropicMessagesSseEvent {
            event: "content_block_delta".to_string(),
            data: AnthropicMessagesStreamEvent::ContentBlockDelta {
                index: 0,
                delta: json!({ "type": "text_delta", "text": "Hello" }),
            },
        }),
        Ok(AnthropicMessagesSseEvent {
            event: "content_block_delta".to_string(),
            data: AnthropicMessagesStreamEvent::ContentBlockDelta {
                index: 0,
                delta: json!({ "type": "thinking_delta", "thinking": "plan" }),
            },
        }),
        Ok(AnthropicMessagesSseEvent {
            event: "content_block_start".to_string(),
            data: AnthropicMessagesStreamEvent::ContentBlockStart {
                index: 1,
                content_block: Box::new(AnthropicContentPart::ToolUse {
                    id: "toolu_1".to_string(),
                    name: "get_weather".to_string(),
                    input: Some(json!({ "city": "SF" })),
                    cache_control: None,
                }),
            },
        }),
        Ok(AnthropicMessagesSseEvent {
            event: "message_stop".to_string(),
            data: AnthropicMessagesStreamEvent::MessageStop,
        }),
    ];

    let mut stream = adapt_messages_stream(stream::iter(events).boxed());
    let unified: Vec<UnifiedStreamEvent> = stream.by_ref().collect().await;

    assert_eq!(unified.len(), 4);
    assert!(matches!(unified[0], UnifiedStreamEvent::ContentDelta(_)));
    assert!(matches!(unified[1], UnifiedStreamEvent::ReasoningDelta(_)));
    assert!(matches!(unified[2], UnifiedStreamEvent::ToolDelta(_)));
    match &unified[3] {
        UnifiedStreamEvent::Done {
            source, id, model, ..
        } => {
            assert_eq!(*source, UnifiedStreamSource::Messages);
            assert_eq!(id.as_deref(), Some("msg_1"));
            assert_eq!(model.as_deref(), Some("anthropic/claude-sonnet-4"));
        }
        other => panic!("expected Done event, got {other:?}"),
    }
}

#[tokio::test]
async fn test_unified_messages_stream_error_then_done() {
    let events = vec![Err(OpenRouterError::Unknown("io boom".to_string()))];
    let mut stream = adapt_messages_stream(stream::iter(events).boxed());
    let unified: Vec<UnifiedStreamEvent> = stream.by_ref().collect().await;

    assert_eq!(unified.len(), 2);
    assert!(matches!(unified[0], UnifiedStreamEvent::Error(_)));
    assert!(matches!(unified[1], UnifiedStreamEvent::Done { .. }));
}

#[tokio::test]
async fn test_unified_messages_tool_delta_preserves_content_block_index() {
    let events = vec![
        Ok(AnthropicMessagesSseEvent {
            event: "content_block_delta".to_string(),
            data: AnthropicMessagesStreamEvent::ContentBlockDelta {
                index: 2,
                delta: json!({
                    "type": "input_json_delta",
                    "partial_json": "{\"city\":\"S"
                }),
            },
        }),
        Ok(AnthropicMessagesSseEvent {
            event: "message_stop".to_string(),
            data: AnthropicMessagesStreamEvent::MessageStop,
        }),
    ];

    let mut stream = adapt_messages_stream(stream::iter(events).boxed());
    let unified: Vec<UnifiedStreamEvent> = stream.by_ref().collect().await;

    assert_eq!(unified.len(), 2);
    match &unified[0] {
        UnifiedStreamEvent::ToolDelta(payload) => {
            assert_eq!(
                payload.get("index").and_then(|value| value.as_u64()),
                Some(2)
            );
            assert_eq!(
                payload
                    .get("delta")
                    .and_then(|delta| delta.get("type"))
                    .and_then(|value| value.as_str()),
                Some("input_json_delta")
            );
            assert_eq!(
                payload
                    .get("delta")
                    .and_then(|delta| delta.get("partial_json"))
                    .and_then(|value| value.as_str()),
                Some("{\"city\":\"S")
            );
        }
        other => panic!("expected ToolDelta event, got {other:?}"),
    }
    assert!(matches!(
        unified[1],
        UnifiedStreamEvent::Done {
            source: UnifiedStreamSource::Messages,
            ..
        }
    ));
}

#[tokio::test]
async fn test_unified_messages_tool_start_preserves_content_block_index() {
    let events = vec![
        Ok(AnthropicMessagesSseEvent {
            event: "content_block_start".to_string(),
            data: AnthropicMessagesStreamEvent::ContentBlockStart {
                index: 3,
                content_block: Box::new(AnthropicContentPart::ToolUse {
                    id: "toolu_2".to_string(),
                    name: "get_weather".to_string(),
                    input: Some(json!({ "city": "SF" })),
                    cache_control: None,
                }),
            },
        }),
        Ok(AnthropicMessagesSseEvent {
            event: "message_stop".to_string(),
            data: AnthropicMessagesStreamEvent::MessageStop,
        }),
    ];

    let mut stream = adapt_messages_stream(stream::iter(events).boxed());
    let unified: Vec<UnifiedStreamEvent> = stream.by_ref().collect().await;

    assert_eq!(unified.len(), 2);
    match &unified[0] {
        UnifiedStreamEvent::ToolDelta(payload) => {
            assert_eq!(
                payload.get("index").and_then(|value| value.as_u64()),
                Some(3)
            );
            assert_eq!(
                payload
                    .get("content_block")
                    .and_then(|block| block.get("id"))
                    .and_then(|value| value.as_str()),
                Some("toolu_2")
            );
            assert_eq!(
                payload
                    .get("content_block")
                    .and_then(|block| block.get("name"))
                    .and_then(|value| value.as_str()),
                Some("get_weather")
            );
        }
        other => panic!("expected ToolDelta event, got {other:?}"),
    }
    assert!(matches!(
        unified[1],
        UnifiedStreamEvent::Done {
            source: UnifiedStreamSource::Messages,
            ..
        }
    ));
}
