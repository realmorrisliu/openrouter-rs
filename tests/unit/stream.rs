use futures_util::{StreamExt, stream};
use openrouter_rs::error::OpenRouterError;
use openrouter_rs::types::completion::{
    CompletionsResponse, PartialFunctionCall, PartialToolCall, ResponseUsage,
};
use openrouter_rs::types::stream::{StreamEvent, ToolAwareStream};
use serde_json::json;

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
        partials[0].function.as_ref().unwrap().arguments.as_deref(),
        Some("{\"loc")
    );
}

// =============================================
// ToolAwareStream accumulation tests
// =============================================

/// Helper: create a CompletionsResponse with a streaming content delta.
fn content_chunk(id: &str, model: &str, content: &str) -> CompletionsResponse {
    serde_json::from_value(json!({
        "id": id,
        "choices": [{
            "finish_reason": null,
            "native_finish_reason": null,
            "delta": {
                "content": content,
                "role": null,
                "tool_calls": null,
                "reasoning": null,
                "reasoning_details": null,
                "audio": null,
                "refusal": null
            },
            "error": null,
            "index": 0,
            "logprobs": null
        }],
        "created": 1700000000_u64,
        "model": model,
        "object": "chat.completion.chunk",
        "provider": null,
        "system_fingerprint": null,
        "usage": null
    }))
    .expect("content chunk should deserialize")
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
        json!({
            "name": func_name,
            "arguments": func_args
        })
    } else {
        serde_json::Value::Null
    };

    serde_json::from_value(json!({
        "id": id,
        "choices": [{
            "finish_reason": null,
            "native_finish_reason": null,
            "delta": {
                "content": null,
                "role": null,
                "tool_calls": [{
                    "id": tool_id,
                    "type": tool_type,
                    "function": function,
                    "index": tool_call_index
                }],
                "reasoning": null,
                "reasoning_details": null,
                "audio": null,
                "refusal": null
            },
            "error": null,
            "index": 0,
            "logprobs": null
        }],
        "created": 1700000000_u64,
        "model": model,
        "object": "chat.completion.chunk",
        "provider": null,
        "system_fingerprint": null,
        "usage": null
    }))
    .expect("tool call chunk should deserialize")
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

    serde_json::from_value(json!({
        "id": id,
        "choices": [{
            "finish_reason": reason,
            "native_finish_reason": finish_reason,
            "delta": {
                "content": null,
                "role": null,
                "tool_calls": null,
                "reasoning": null,
                "reasoning_details": null,
                "audio": null,
                "refusal": null
            },
            "error": null,
            "index": 0,
            "logprobs": null
        }],
        "created": 1700000000_u64,
        "model": model,
        "object": "chat.completion.chunk",
        "provider": null,
        "system_fingerprint": null,
        "usage": usage
    }))
    .expect("done chunk should deserialize")
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
            Some(ResponseUsage::new(100, 20, 120)),
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
            assert_eq!(tool_calls[0].function.arguments, "{\"location\": \"NYC\"}");
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
            assert_eq!(tool_calls[1].function.arguments, "{\"timezone\": \"EST\"}");
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
    let reasoning_chunk: CompletionsResponse = serde_json::from_value(json!({
        "id": "gen-1",
        "choices": [{
            "finish_reason": null,
            "native_finish_reason": null,
            "delta": {
                "content": null,
                "role": null,
                "tool_calls": null,
                "reasoning": "Let me think...",
                "reasoning_details": null,
                "audio": null,
                "refusal": null
            },
            "error": null,
            "index": 0,
            "logprobs": null
        }],
        "created": 1700000000_u64,
        "model": "test-model",
        "object": "chat.completion.chunk",
        "provider": null,
        "system_fingerprint": null,
        "usage": null
    }))
    .expect("reasoning chunk should deserialize");

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
