use openrouter_rs::types::completion::{Choice, CompletionsResponse};

/// Test deserialization of a standard non-streaming response
#[test]
fn test_non_streaming_response_deserialization() {
    let json = r#"{
        "id": "gen-123",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello from Rust!",
                "model": "openai/gpt-4o"
            }
        }],
        "created": 1700000000,
        "model": "deepseek/deepseek-chat-v3-0324:free",
        "object": "chat.completion"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(response.id, "gen-123");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.model, "deepseek/deepseek-chat-v3-0324:free");

    let choice = &response.choices[0];
    assert_eq!(choice.content(), Some("Hello from Rust!"));
    assert_eq!(choice.role(), Some("assistant"));
    assert_eq!(choice.index(), Some(0));
    let Choice::NonStreaming(choice) = choice else {
        panic!("expected non-streaming choice");
    };
    assert_eq!(choice.message.model.as_deref(), Some("openai/gpt-4o"));
}

/// Test deserialization of response with index field (Grok model format)
#[test]
fn test_response_with_index_field() {
    let json = r#"{
        "id": "gen-grok-456",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "native_finish_reason": "stop",
            "message": {
                "role": "assistant",
                "content": "Hello from Grok!"
            }
        }],
        "created": 1700000000,
        "model": "x-ai/grok-4.3",
        "object": "chat.completion"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(response.id, "gen-grok-456");
    assert_eq!(response.choices.len(), 1);

    let choice = &response.choices[0];
    assert_eq!(choice.content(), Some("Hello from Grok!"));
    assert_eq!(choice.index(), Some(0));
}

/// Test deserialization of response with logprobs field
#[test]
fn test_response_with_logprobs() {
    let json = r#"{
        "id": "gen-789",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Test"
            },
            "logprobs": {
                "content": [{"token": "Test", "logprob": -0.5}]
            }
        }],
        "created": 1700000000,
        "model": "test-model",
        "object": "chat.completion"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");

    let choice = &response.choices[0];
    assert!(choice.logprobs().is_some());
}

/// Test deserialization of response with reasoning details (reasoning models)
#[test]
fn test_response_with_reasoning_details() {
    let json = r#"{
        "id": "gen-reasoning-001",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "The answer is 42.",
                "reasoning": "Let me think step by step...",
                "reasoning_details": [
                    {
                        "type": "reasoning.text",
                        "text": "First, I need to consider..."
                    },
                    {
                        "type": "reasoning.summary",
                        "text": "Summary of reasoning"
                    }
                ]
            }
        }],
        "created": 1700000000,
        "model": "x-ai/grok-4",
        "object": "chat.completion"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");

    let choice = &response.choices[0];
    assert_eq!(choice.content(), Some("The answer is 42."));
    assert_eq!(choice.reasoning(), Some("Let me think step by step..."));

    let reasoning_details = choice
        .reasoning_details()
        .expect("Should have reasoning_details");
    assert_eq!(reasoning_details.len(), 2);
    assert_eq!(
        reasoning_details[0].content(),
        Some("First, I need to consider...")
    );
    assert_eq!(reasoning_details[0].reasoning_type(), "reasoning.text");
}

#[test]
fn test_server_tool_reasoning_detail_deserializes() {
    let detail: openrouter_rs::types::completion::ReasoningDetail = serde_json::from_str(
        r#"{"type":"reasoning.server_tool_call","index":0,"tool_name":"web_search","arguments":"{}","result":"ok","tool_call_id":"call_1"}"#,
    )
    .expect("server tool reasoning detail should deserialize");

    assert_eq!(detail.tool_name.as_deref(), Some("web_search"));
    assert_eq!(detail.arguments.as_deref(), Some("{}"));
    assert_eq!(detail.result.as_deref(), Some("ok"));
    assert_eq!(detail.tool_call_id.as_deref(), Some("call_1"));
}

/// Test deserialization of streaming response chunk
#[test]
fn test_streaming_response_deserialization() {
    let json = r#"{
        "id": "gen-stream-001",
        "choices": [{
            "finish_reason": null,
            "index": 0,
            "delta": {
                "role": "assistant",
                "content": "Hello"
            }
        }],
        "created": 1700000000,
        "model": "test-model",
        "object": "chat.completion.chunk"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(response.choices.len(), 1);

    let choice = &response.choices[0];
    assert_eq!(choice.content(), Some("Hello"));
    assert_eq!(choice.index(), Some(0));
}

/// OpenRouter's schema marks `finish_reason` as allowing unknown values, so a
/// provider may stream a value this SDK does not model. The whole SSE frame
/// must still deserialize, capturing the raw value instead of erroring.
#[test]
fn test_unknown_finish_reason_does_not_fail_full_frame() {
    let json = r#"{
        "id": "gen-stream-002",
        "choices": [{
            "finish_reason": "function_call",
            "index": 0,
            "delta": {
                "role": "assistant",
                "content": null
            }
        }],
        "created": 1700000000,
        "model": "test-model",
        "object": "chat.completion.chunk"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(response.choices.len(), 1);
    let choice = &response.choices[0];
    assert!(matches!(
        choice.finish_reason(),
        Some(openrouter_rs::types::completion::FinishReason::Other(value))
            if value == "function_call"
    ));
}

/// Unknown finish reasons survive a serialize round-trip and known variants
/// keep their snake_case wire form.
#[test]
fn test_finish_reason_round_trip() {
    use openrouter_rs::types::completion::FinishReason;

    assert_eq!(
        serde_json::to_string(&FinishReason::ToolCalls).unwrap(),
        r#""tool_calls""#
    );
    assert_eq!(
        serde_json::to_string(&FinishReason::Stop).unwrap(),
        r#""stop""#
    );
    assert_eq!(
        serde_json::to_string(&FinishReason::Other("max_tokens".to_string())).unwrap(),
        r#""max_tokens""#
    );

    let parsed: FinishReason = serde_json::from_str(r#""function_call""#).unwrap();
    assert!(matches!(parsed, FinishReason::Other(value) if value == "function_call"));
    let known: FinishReason = serde_json::from_str(r#""length""#).unwrap();
    assert!(matches!(known, FinishReason::Length));
}

/// Test deserialization with refusal field
#[test]
fn test_response_with_refusal() {
    let json = r#"{
        "id": "gen-refusal-001",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "refusal": "I cannot help with that request."
            }
        }],
        "created": 1700000000,
        "model": "test-model",
        "object": "chat.completion"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");

    let choice = &response.choices[0];
    assert!(choice.content().is_none());
    // Note: refusal is stored in Message but not exposed via Choice::refusal() yet
}

/// Test non-chat (text completion) response
#[test]
fn test_non_chat_response_deserialization() {
    let json = r#"{
        "id": "gen-text-001",
        "choices": [{
            "finish_reason": "stop",
            "text": "Completed text here",
            "index": 0
        }],
        "created": 1700000000,
        "model": "text-model",
        "object": "chat.completion"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");

    let choice = &response.choices[0];
    assert_eq!(choice.content(), Some("Completed text here"));
}

/// Test that optional fields can be omitted
#[test]
fn test_minimal_response() {
    let json = r#"{
        "id": "gen-minimal",
        "choices": [{
            "message": {
                "content": "Hello"
            }
        }],
        "created": 1700000000,
        "model": "test-model",
        "object": "chat.completion"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(response.choices.len(), 1);
    let choice = &response.choices[0];
    assert_eq!(choice.content(), Some("Hello"));
    assert!(choice.finish_reason().is_none());
    assert!(choice.index().is_none());
}

/// Test Gemini tool call response with annotations field
#[test]
fn test_gemini_tool_call_response() {
    let json = r#"{"id":"gen-123","provider":"Google AI Studio","model":"google/gemini-3-flash-preview","object":"chat.completion","created":1767358919,"choices":[{"logprobs":null,"finish_reason":"tool_calls","native_finish_reason":"STOP","index":0,"message":{"role":"assistant","content":"","refusal":null,"reasoning":null,"tool_calls":[{"type":"function","index":0,"id":"tool_123","function":{"name":"test","arguments":"{}"}}],"reasoning_details":[{"id":"tool_123","format":"google-gemini-v1","index":0,"type":"reasoning.encrypted","data":"abc123"}],"annotations":[]}}],"usage":{"prompt_tokens":100,"completion_tokens":10,"total_tokens":110}}"#;

    let result = serde_json::from_str::<CompletionsResponse>(json);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_usage_deserializes_openrouter_cost_fields() {
    let json = r#"{
        "id": "gen-usage-cost",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello"
            }
        }],
        "created": 1700000000,
        "model": "openai/gpt-4o-mini",
        "object": "chat.completion",
        "usage": {
            "prompt_tokens": 100,
            "completion_tokens": 10,
            "total_tokens": 110,
            "cost": 0.00025,
            "cost_details": {
                "upstream_inference_prompt_cost": 0.0001,
                "upstream_inference_completions_cost": 0.00015,
                "upstream_inference_cost": null
            },
            "is_byok": false,
            "server_tool_use_details": {
                "tool_calls_requested": 2,
                "tool_calls_executed": 1,
                "web_search_requests": 1
            }
        }
    }"#;

    let response: CompletionsResponse =
        serde_json::from_str(json).expect("response should deserialize");
    let usage = response.usage.expect("usage should be present");
    let cost_details = usage.cost_details.expect("cost details should be present");

    assert_eq!(usage.cost, Some(0.00025));
    assert_eq!(usage.is_byok, Some(false));
    assert_eq!(cost_details.upstream_inference_prompt_cost, 0.0001);
    assert_eq!(cost_details.upstream_inference_completions_cost, 0.00015);
    assert_eq!(cost_details.upstream_inference_cost, None);
    let server_tool_use = usage
        .server_tool_use_details
        .expect("server tool details should be present");
    assert_eq!(server_tool_use.tool_calls_requested, Some(2));
    assert_eq!(server_tool_use.tool_calls_executed, Some(1));
    assert_eq!(server_tool_use.web_search_requests, Some(1));
}

/// Test deserialization of multimodal assistant content parts
#[test]
fn test_response_with_multimodal_content_parts() {
    let json = r#"{
        "id": "gen-multi-001",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": [
                    {"type":"output_text","text":"Caption: beach sunset"},
                    {"type":"image_url","image_url":{"url":"https://example.com/out.png"}}
                ]
            }
        }],
        "created": 1700000000,
        "model": "test-model",
        "object": "chat.completion"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");
    let choice = &response.choices[0];

    assert_eq!(choice.content(), Some("Caption: beach sunset"));
}

#[test]
fn test_response_with_text_content_object() {
    let json = r#"{
        "id": "gen-multi-002",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": {
                    "type":"output_text",
                    "text":"Hello from object content"
                }
            }
        }],
        "created": 1700000000,
        "model": "test-model",
        "object": "chat.completion"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");
    let choice = &response.choices[0];

    assert_eq!(choice.content(), Some("Hello from object content"));
}

/// Test deserialization of assistant images/audio fields
#[test]
fn test_response_with_assistant_media_fields() {
    let json = r#"{
        "id": "gen-media-001",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Here is your result",
                "images": [{"url":"https://example.com/generated.png"}],
                "audio": {"id":"audio_123","expires_at":1700001000}
            }
        }],
        "created": 1700000000,
        "model": "test-model",
        "object": "chat.completion"
    }"#;

    let response: CompletionsResponse = serde_json::from_str(json).expect("Failed to deserialize");
    let choice = &response.choices[0];
    assert_eq!(choice.content(), Some("Here is your result"));
}
