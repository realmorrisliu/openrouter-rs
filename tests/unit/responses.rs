use openrouter_rs::api::{
    chat::{Plugin, TraceOptions},
    responses::{ResponsesRequest, ResponsesResponse, ResponsesStreamEvent},
};
use serde_json::json;

#[test]
fn test_responses_request_serialization() {
    let request = ResponsesRequest::builder()
        .model("openai/gpt-5")
        .input(json!([{
            "role": "user",
            "content": "Hello from responses API"
        }]))
        .instructions("Be concise")
        .metadata([("env", "test"), ("feature", "responses")])
        .tools(vec![json!({
            "type": "function",
            "name": "get_weather",
            "parameters": { "type": "object" }
        })])
        .tool_choice(json!("auto"))
        .parallel_tool_calls(true)
        .models(vec![
            "openai/gpt-5".to_string(),
            "openai/gpt-4o".to_string(),
        ])
        .max_output_tokens(256)
        .temperature(0.2)
        .top_p(0.9)
        .top_logprobs(5)
        .max_tool_calls(2)
        .presence_penalty(0.0)
        .frequency_penalty(0.0)
        .top_k(40.0)
        .image_config([("aspect_ratio", json!("16:9"))])
        .modalities(vec!["text".to_string(), "image".to_string()])
        .prompt_cache_key("cache-key-1")
        .previous_response_id("resp-prev")
        .include(vec!["reasoning.encrypted_content".to_string()])
        .background(false)
        .safety_identifier("user-123")
        .store(false)
        .service_tier("auto")
        .truncation("auto")
        .user("user-123")
        .session_id("session-abc")
        .trace(TraceOptions {
            trace_id: Some("trace-1".to_string()),
            trace_name: None,
            span_name: Some("responses.unit".to_string()),
            generation_name: None,
            parent_span_id: None,
            extra: Default::default(),
        })
        .plugins(vec![Plugin::new("web").option("max_results", 3)])
        .build()
        .expect("responses request should build");

    let value = serde_json::to_value(&request).expect("responses request should serialize");
    assert_eq!(value["model"], "openai/gpt-5");
    assert_eq!(value["instructions"], "Be concise");
    assert_eq!(value["metadata"]["env"], "test");
    assert_eq!(value["tool_choice"], "auto");
    assert_eq!(value["parallel_tool_calls"], true);
    assert_eq!(value["max_output_tokens"], 256);
    assert_eq!(value["modalities"][1], "image");
    assert_eq!(value["plugins"][0]["id"], "web");
    assert_eq!(value["trace"]["trace_id"], "trace-1");
    assert_eq!(value["trace"]["span_name"], "responses.unit");
}

#[test]
fn test_responses_response_deserialization() {
    let raw = r#"{
        "id": "resp-abc123",
        "object": "response",
        "created_at": 1704067200,
        "model": "gpt-4",
        "status": "completed",
        "output": [{
            "type": "message",
            "id": "msg-abc123",
            "status": "completed",
            "role": "assistant",
            "content": [{
                "type": "output_text",
                "text": "Hello!",
                "annotations": []
            }]
        }],
        "usage": {
            "input_tokens": 10,
            "output_tokens": 25,
            "total_tokens": 35
        }
    }"#;

    let response: ResponsesResponse =
        serde_json::from_str(raw).expect("responses payload should deserialize");
    assert_eq!(response.id.as_deref(), Some("resp-abc123"));
    assert_eq!(response.object_type.as_deref(), Some("response"));
    assert_eq!(response.status.as_deref(), Some("completed"));
    assert!(response.output.is_some());
    assert!(response.usage.is_some());
}

#[test]
fn test_responses_stream_event_deserialization() {
    let raw = r#"{
        "type": "response.output_text.delta",
        "sequence_number": 4,
        "delta": "Hello"
    }"#;

    let event: ResponsesStreamEvent =
        serde_json::from_str(raw).expect("stream event should deserialize");
    assert_eq!(event.event_type, "response.output_text.delta");
    assert_eq!(event.sequence_number, Some(4));
    assert_eq!(
        event.data.get("delta").and_then(|value| value.as_str()),
        Some("Hello")
    );
}

#[test]
fn test_responses_stream_event_with_response_payload() {
    let raw = r#"{
        "type": "response.completed",
        "sequence_number": 10,
        "response": {
            "id": "resp-abc123",
            "status": "completed"
        }
    }"#;

    let event: ResponsesStreamEvent =
        serde_json::from_str(raw).expect("stream event with response should deserialize");
    assert_eq!(event.event_type, "response.completed");
    assert_eq!(event.sequence_number, Some(10));
    assert_eq!(
        event
            .data
            .get("response")
            .and_then(|value| value.get("id"))
            .and_then(|value| value.as_str()),
        Some("resp-abc123")
    );
}
