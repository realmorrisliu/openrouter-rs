use futures_util::{StreamExt, stream};
use openrouter_rs::error::OpenRouterError;
use openrouter_rs::types::completion::{
    Choice, CompletionsResponse, Delta, ObjectType, PartialFunctionCall, PartialToolCall,
    ResponseUsage, StreamingChoice,
};
use openrouter_rs::types::stream::{StreamEvent, ToolAwareStream};

// =============================================
// PartialToolCall deserialization tests
// =============================================

#[test]
fn test_partial_tool_call_full_first_chunk() {
    // First chunk typically has id, type, and function name
    let json = r#"{
        "index": 0,
        "id": "call_abc123",
        "type": "function",
        "function": {
            "name": "get_weather",
            "arguments": ""
        }
    }"#;

    let partial: PartialToolCall = serde_json::from_str(json).unwrap();
    assert_eq!(partial.index, Some(0));
    assert_eq!(partial.id.as_deref(), Some("call_abc123"));
    assert_eq!(partial.type_.as_deref(), Some("function"));

    let func = partial.function.unwrap();
    assert_eq!(func.name.as_deref(), Some("get_weather"));
    assert_eq!(func.arguments.as_deref(), Some(""));
}

#[test]
fn test_partial_tool_call_arguments_only_chunk() {
    // Subsequent chunks only have index + arguments fragment
    let json = r#"{
        "index": 0,
        "function": {
            "arguments": "{\"location"
        }
    }"#;

    let partial: PartialToolCall = serde_json::from_str(json).unwrap();
    assert_eq!(partial.index, Some(0));
    assert!(partial.id.is_none());
    assert!(partial.type_.is_none());

    let func = partial.function.unwrap();
    assert!(func.name.is_none());
    assert_eq!(func.arguments.as_deref(), Some("{\"location"));
}

#[test]
fn test_partial_tool_call_minimal_chunk() {
    // Minimal chunk with only index
    let json = r#"{"index": 1}"#;

    let partial: PartialToolCall = serde_json::from_str(json).unwrap();
    assert_eq!(partial.index, Some(1));
    assert!(partial.id.is_none());
    assert!(partial.type_.is_none());
    assert!(partial.function.is_none());
}

#[test]
fn test_partial_tool_call_empty_object() {
    let json = r#"{}"#;

    let partial: PartialToolCall = serde_json::from_str(json).unwrap();
    assert!(partial.index.is_none());
    assert!(partial.id.is_none());
    assert!(partial.type_.is_none());
    assert!(partial.function.is_none());
}

#[test]
fn test_partial_function_call_default() {
    let partial = PartialFunctionCall::default();
    assert!(partial.name.is_none());
    assert!(partial.arguments.is_none());
}

#[test]
fn test_streaming_chunk_with_partial_tool_calls() {
    // Full streaming SSE chunk with tool_calls in delta
    let json = r#"{
        "id": "gen-123",
        "choices": [{
            "finish_reason": null,
            "index": 0,
            "delta": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "index": 0,
                    "id": "call_abc",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": ""
                    }
                }]
            }
        }],
        "created": 1700000000,
        "model": "test-model",
        "object": "chat.completion.chunk"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.choices.len(), 1);

    let choice = &response.choices[0];
    // tool_calls() should return None for streaming
    assert!(choice.tool_calls().is_none());

    // partial_tool_calls() should return the fragments
    let partials = choice.partial_tool_calls().unwrap();
    assert_eq!(partials.len(), 1);
    assert_eq!(partials[0].id.as_deref(), Some("call_abc"));
    assert_eq!(
        partials[0].function.as_ref().unwrap().name.as_deref(),
        Some("get_weather")
    );
}

#[test]
fn test_streaming_chunk_arguments_fragment() {
    let json = r#"{
        "id": "gen-123",
        "choices": [{
            "finish_reason": null,
            "index": 0,
            "delta": {
                "tool_calls": [{
                    "index": 0,
                    "function": {
                        "arguments": "{\"loc"
                    }
                }]
            }
        }],
        "created": 1700000000,
        "model": "test-model",
        "object": "chat.completion.chunk"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).unwrap();
    let partials = response.choices[0].partial_tool_calls().unwrap();
    assert_eq!(partials.len(), 1);
    assert_eq!(
        partials[0]
            .function
            .as_ref()
            .unwrap()
            .arguments
            .as_deref(),
        Some("{\"loc")
    );
}

// =============================================
// ToolAwareStream accumulation tests
// =============================================

/// Helper: create a CompletionsResponse with a streaming content delta.
fn content_chunk(id: &str, model: &str, content: &str) -> CompletionsResponse {
    CompletionsResponse {
        id: id.to_string(),
        choices: vec![Choice::Streaming(StreamingChoice {
            finish_reason: None,
            native_finish_reason: None,
            delta: Delta {
                content: Some(content.to_string()),
                role: None,
                tool_calls: None,
                reasoning: None,
                reasoning_details: None,
                refusal: None,
            },
            error: None,
            index: Some(0),
            logprobs: None,
        })],
        created: 1700000000,
        model: model.to_string(),
        object_type: ObjectType::ChatCompletionChunk,
        provider: None,
        system_fingerprint: None,
        usage: None,
    }
}

/// Helper: create a CompletionsResponse with a partial tool call delta.
fn tool_call_chunk(
    id: &str,
    model: &str,
    tool_call_index: u32,
    tool_id: Option<&str>,
    tool_type: Option<&str>,
    func_name: Option<&str>,
    func_args: Option<&str>,
) -> CompletionsResponse {
    let function = if func_name.is_some() || func_args.is_some() {
        Some(PartialFunctionCall {
            name: func_name.map(|s| s.to_string()),
            arguments: func_args.map(|s| s.to_string()),
        })
    } else {
        None
    };

    CompletionsResponse {
        id: id.to_string(),
        choices: vec![Choice::Streaming(StreamingChoice {
            finish_reason: None,
            native_finish_reason: None,
            delta: Delta {
                content: None,
                role: None,
                tool_calls: Some(vec![PartialToolCall {
                    id: tool_id.map(|s| s.to_string()),
                    type_: tool_type.map(|s| s.to_string()),
                    function,
                    index: Some(tool_call_index),
                }]),
                reasoning: None,
                reasoning_details: None,
                refusal: None,
            },
            error: None,
            index: Some(0),
            logprobs: None,
        })],
        created: 1700000000,
        model: model.to_string(),
        object_type: ObjectType::ChatCompletionChunk,
        provider: None,
        system_fingerprint: None,
        usage: None,
    }
}

/// Helper: create a final chunk with finish_reason and usage.
fn done_chunk(
    id: &str,
    model: &str,
    finish_reason: &str,
    usage: Option<ResponseUsage>,
) -> CompletionsResponse {
    use openrouter_rs::types::completion::FinishReason;

    let reason = match finish_reason {
        "tool_calls" => Some(FinishReason::ToolCalls),
        "stop" => Some(FinishReason::Stop),
        _ => None,
    };

    CompletionsResponse {
        id: id.to_string(),
        choices: vec![Choice::Streaming(StreamingChoice {
            finish_reason: reason,
            native_finish_reason: Some(finish_reason.to_string()),
            delta: Delta {
                content: None,
                role: None,
                tool_calls: None,
                reasoning: None,
                reasoning_details: None,
                refusal: None,
            },
            error: None,
            index: Some(0),
            logprobs: None,
        })],
        created: 1700000000,
        model: model.to_string(),
        object_type: ObjectType::ChatCompletionChunk,
        provider: None,
        system_fingerprint: None,
        usage,
    }
}

#[tokio::test]
async fn test_tool_aware_stream_content_only() {
    // Stream with only text content, no tool calls
    let chunks: Vec<Result<CompletionsResponse, OpenRouterError>> = vec![
        Ok(content_chunk("gen-1", "test-model", "Hello")),
        Ok(content_chunk("gen-1", "test-model", " world")),
        Ok(done_chunk("gen-1", "test-model", "stop", None)),
    ];

    let raw_stream = stream::iter(chunks).boxed();
    let mut stream = ToolAwareStream::new(raw_stream);

    let mut events: Vec<StreamEvent> = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    // Should have: ContentDelta("Hello"), ContentDelta(" world"), Done
    assert_eq!(events.len(), 3);

    match &events[0] {
        StreamEvent::ContentDelta(text) => assert_eq!(text, "Hello"),
        other => panic!("Expected ContentDelta, got {:?}", other),
    }

    match &events[1] {
        StreamEvent::ContentDelta(text) => assert_eq!(text, " world"),
        other => panic!("Expected ContentDelta, got {:?}", other),
    }

    match &events[2] {
        StreamEvent::Done {
            tool_calls,
            finish_reason,
            model,
            ..
        } => {
            assert!(tool_calls.is_empty());
            assert!(matches!(
                finish_reason,
                Some(openrouter_rs::types::completion::FinishReason::Stop)
            ));
            assert_eq!(model, "test-model");
        }
        other => panic!("Expected Done, got {:?}", other),
    }
}

#[tokio::test]
async fn test_tool_aware_stream_single_tool_call() {
    // Simulate streaming a single tool call
    let chunks: Vec<Result<CompletionsResponse, OpenRouterError>> = vec![
        // First chunk: tool call header
        Ok(tool_call_chunk(
            "gen-1",
            "gpt-4",
            0,
            Some("call_abc"),
            Some("function"),
            Some("get_weather"),
            Some(""),
        )),
        // Arguments fragments
        Ok(tool_call_chunk(
            "gen-1",
            "gpt-4",
            0,
            None,
            None,
            None,
            Some("{\"location"),
        )),
        Ok(tool_call_chunk(
            "gen-1",
            "gpt-4",
            0,
            None,
            None,
            None,
            Some("\": \"NYC\"}"),
        )),
        // Final chunk
        Ok(done_chunk(
            "gen-1",
            "gpt-4",
            "tool_calls",
            Some(ResponseUsage {
                prompt_tokens: 100,
                completion_tokens: 20,
                total_tokens: 120,
            }),
        )),
    ];

    let raw_stream = stream::iter(chunks).boxed();
    let mut stream = ToolAwareStream::new(raw_stream);

    let mut events: Vec<StreamEvent> = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    // Should have exactly 1 event: Done (no content deltas)
    assert_eq!(events.len(), 1);

    match &events[0] {
        StreamEvent::Done {
            tool_calls,
            finish_reason,
            usage,
            id,
            model,
        } => {
            assert_eq!(tool_calls.len(), 1);
            assert_eq!(tool_calls[0].id, "call_abc");
            assert_eq!(tool_calls[0].type_, "function");
            assert_eq!(tool_calls[0].function.name, "get_weather");
            assert_eq!(
                tool_calls[0].function.arguments,
                "{\"location\": \"NYC\"}"
            );
            assert!(matches!(
                finish_reason,
                Some(openrouter_rs::types::completion::FinishReason::ToolCalls)
            ));
            assert!(usage.is_some());
            assert_eq!(usage.as_ref().unwrap().total_tokens, 120);
            assert_eq!(id, "gen-1");
            assert_eq!(model, "gpt-4");
        }
        other => panic!("Expected Done, got {:?}", other),
    }
}

#[tokio::test]
async fn test_tool_aware_stream_parallel_tool_calls() {
    // Simulate two tool calls arriving in parallel (interleaved by index)
    let chunks: Vec<Result<CompletionsResponse, OpenRouterError>> = vec![
        // Tool call 0 header
        Ok(tool_call_chunk(
            "gen-1",
            "gpt-4",
            0,
            Some("call_1"),
            Some("function"),
            Some("get_weather"),
            Some(""),
        )),
        // Tool call 1 header
        Ok(tool_call_chunk(
            "gen-1",
            "gpt-4",
            1,
            Some("call_2"),
            Some("function"),
            Some("get_time"),
            Some(""),
        )),
        // Tool call 0 arguments
        Ok(tool_call_chunk(
            "gen-1",
            "gpt-4",
            0,
            None,
            None,
            None,
            Some("{\"city\": \"NYC\"}"),
        )),
        // Tool call 1 arguments
        Ok(tool_call_chunk(
            "gen-1",
            "gpt-4",
            1,
            None,
            None,
            None,
            Some("{\"timezone\": \"EST\"}"),
        )),
        // Done
        Ok(done_chunk("gen-1", "gpt-4", "tool_calls", None)),
    ];

    let raw_stream = stream::iter(chunks).boxed();
    let mut stream = ToolAwareStream::new(raw_stream);

    let mut events: Vec<StreamEvent> = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    assert_eq!(events.len(), 1);

    match &events[0] {
        StreamEvent::Done { tool_calls, .. } => {
            assert_eq!(tool_calls.len(), 2);

            // BTreeMap preserves order by key (index), so index 0 comes first
            assert_eq!(tool_calls[0].id, "call_1");
            assert_eq!(tool_calls[0].function.name, "get_weather");
            assert_eq!(tool_calls[0].function.arguments, "{\"city\": \"NYC\"}");

            assert_eq!(tool_calls[1].id, "call_2");
            assert_eq!(tool_calls[1].function.name, "get_time");
            assert_eq!(
                tool_calls[1].function.arguments,
                "{\"timezone\": \"EST\"}"
            );
        }
        other => panic!("Expected Done, got {:?}", other),
    }
}

#[tokio::test]
async fn test_tool_aware_stream_content_then_tool_calls() {
    // Model generates some content text AND then makes tool calls
    let chunks: Vec<Result<CompletionsResponse, OpenRouterError>> = vec![
        Ok(content_chunk("gen-1", "gpt-4", "Let me check ")),
        Ok(content_chunk("gen-1", "gpt-4", "the weather.")),
        Ok(tool_call_chunk(
            "gen-1",
            "gpt-4",
            0,
            Some("call_abc"),
            Some("function"),
            Some("get_weather"),
            Some("{\"city\": \"NYC\"}"),
        )),
        Ok(done_chunk("gen-1", "gpt-4", "tool_calls", None)),
    ];

    let raw_stream = stream::iter(chunks).boxed();
    let mut stream = ToolAwareStream::new(raw_stream);

    let mut events: Vec<StreamEvent> = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    // 2 content deltas + 1 Done
    assert_eq!(events.len(), 3);

    match &events[0] {
        StreamEvent::ContentDelta(text) => assert_eq!(text, "Let me check "),
        other => panic!("Expected ContentDelta, got {:?}", other),
    }

    match &events[1] {
        StreamEvent::ContentDelta(text) => assert_eq!(text, "the weather."),
        other => panic!("Expected ContentDelta, got {:?}", other),
    }

    match &events[2] {
        StreamEvent::Done { tool_calls, .. } => {
            assert_eq!(tool_calls.len(), 1);
            assert_eq!(tool_calls[0].function.name, "get_weather");
            assert_eq!(tool_calls[0].function.arguments, "{\"city\": \"NYC\"}");
        }
        other => panic!("Expected Done, got {:?}", other),
    }
}

#[tokio::test]
async fn test_tool_aware_stream_error_handling() {
    let chunks: Vec<Result<CompletionsResponse, OpenRouterError>> = vec![
        Ok(content_chunk("gen-1", "test", "Hello")),
        Err(OpenRouterError::Unknown("stream failed".to_string())),
    ];

    let raw_stream = stream::iter(chunks).boxed();
    let mut stream = ToolAwareStream::new(raw_stream);

    let mut events: Vec<StreamEvent> = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    // ContentDelta, Error, Done (finalize on stream end)
    assert_eq!(events.len(), 3);

    match &events[0] {
        StreamEvent::ContentDelta(text) => assert_eq!(text, "Hello"),
        other => panic!("Expected ContentDelta, got {:?}", other),
    }

    match &events[1] {
        StreamEvent::Error(_) => {} // Expected
        other => panic!("Expected Error, got {:?}", other),
    }

    match &events[2] {
        StreamEvent::Done { tool_calls, .. } => {
            assert!(tool_calls.is_empty());
        }
        other => panic!("Expected Done, got {:?}", other),
    }
}

#[tokio::test]
async fn test_tool_aware_stream_empty_stream() {
    let chunks: Vec<Result<CompletionsResponse, OpenRouterError>> = vec![];

    let raw_stream = stream::iter(chunks).boxed();
    let mut stream = ToolAwareStream::new(raw_stream);

    let mut events: Vec<StreamEvent> = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    // Even an empty stream should emit Done
    assert_eq!(events.len(), 1);

    match &events[0] {
        StreamEvent::Done {
            tool_calls,
            finish_reason,
            usage,
            ..
        } => {
            assert!(tool_calls.is_empty());
            assert!(finish_reason.is_none());
            assert!(usage.is_none());
        }
        other => panic!("Expected Done, got {:?}", other),
    }
}

#[tokio::test]
async fn test_tool_aware_stream_reasoning_deltas() {
    let reasoning_chunk = CompletionsResponse {
        id: "gen-1".to_string(),
        choices: vec![Choice::Streaming(StreamingChoice {
            finish_reason: None,
            native_finish_reason: None,
            delta: Delta {
                content: None,
                role: None,
                tool_calls: None,
                reasoning: Some("Let me think...".to_string()),
                reasoning_details: None,
                refusal: None,
            },
            error: None,
            index: Some(0),
            logprobs: None,
        })],
        created: 1700000000,
        model: "test-model".to_string(),
        object_type: ObjectType::ChatCompletionChunk,
        provider: None,
        system_fingerprint: None,
        usage: None,
    };

    let chunks: Vec<Result<CompletionsResponse, OpenRouterError>> = vec![
        Ok(reasoning_chunk),
        Ok(content_chunk("gen-1", "test-model", "The answer is 42.")),
        Ok(done_chunk("gen-1", "test-model", "stop", None)),
    ];

    let raw_stream = stream::iter(chunks).boxed();
    let mut stream = ToolAwareStream::new(raw_stream);

    let mut events: Vec<StreamEvent> = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    assert_eq!(events.len(), 3);

    match &events[0] {
        StreamEvent::ReasoningDelta(text) => assert_eq!(text, "Let me think..."),
        other => panic!("Expected ReasoningDelta, got {:?}", other),
    }

    match &events[1] {
        StreamEvent::ContentDelta(text) => assert_eq!(text, "The answer is 42."),
        other => panic!("Expected ContentDelta, got {:?}", other),
    }

    match &events[2] {
        StreamEvent::Done { .. } => {}
        other => panic!("Expected Done, got {:?}", other),
    }
}
