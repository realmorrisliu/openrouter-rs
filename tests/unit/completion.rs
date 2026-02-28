use openrouter_rs::types::completion::CompletionsResponse;

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
                "content": "Hello from Rust!"
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
        "model": "x-ai/grok-code-fast-1",
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
