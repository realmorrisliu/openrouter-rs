//! Decoding of chat completion payloads whose shape varies by upstream provider.

use openrouter_rs::types::completion::{
    Choice, CompletionsResponse, ErrorCode, ResponseCostDetails, ToolCall,
};
use serde_json::json;

fn response_with_choice(choice: serde_json::Value) -> serde_json::Value {
    json!({
        "id": "gen-123",
        "choices": [choice],
        "created": 1700000000,
        "model": "openai/gpt-4o",
        "object": "chat.completion"
    })
}

#[test]
fn test_bad_choice_field_reports_its_own_error() {
    let payload = response_with_choice(json!({
        "finish_reason": "stop",
        "index": 0,
        "message": { "role": "assistant", "content": "hi", "tool_calls": "not a list" }
    }));

    let error = serde_json::from_value::<CompletionsResponse>(payload)
        .expect_err("a bad tool_calls field should fail")
        .to_string();

    assert!(error.contains("invalid chat choice"), "{error}");
    assert!(error.contains("invalid type"), "{error}");
    assert!(!error.contains("did not match any variant"), "{error}");
}

#[test]
fn test_choice_without_known_shape_names_the_expected_fields() {
    let error = serde_json::from_value::<Choice>(json!({ "index": 0 }))
        .expect_err("a choice without message, delta or text should fail")
        .to_string();

    assert!(error.contains("`message`, `delta` or `text`"), "{error}");
}

#[test]
fn test_choices_decode_by_shape() {
    let message: Choice =
        serde_json::from_value(json!({ "message": { "role": "assistant", "content": "a" } }))
            .unwrap();
    let delta: Choice = serde_json::from_value(json!({ "delta": { "content": "b" } })).unwrap();
    let text: Choice = serde_json::from_value(json!({ "text": "c" })).unwrap();

    assert!(matches!(message, Choice::NonStreaming(_)));
    assert!(matches!(delta, Choice::Streaming(_)));
    assert!(matches!(text, Choice::NonChat(_)));
    assert_eq!(
        [message.content(), delta.content(), text.content()],
        [Some("a"), Some("b"), Some("c")]
    );
}

#[test]
fn test_choice_error_accepts_numeric_and_string_codes() {
    let payload = json!({
        "id": "gen-123",
        "choices": [
            {
                "finish_reason": "error",
                "index": 0,
                "message": { "role": "assistant", "content": null },
                "error": { "code": 502, "message": "Upstream error" }
            },
            {
                "finish_reason": "error",
                "index": 1,
                "message": { "role": "assistant", "content": null },
                "error": { "code": "rate_limit_exceeded", "message": "Slow down" }
            },
            {
                "finish_reason": "error",
                "index": 2,
                "message": { "role": "assistant", "content": null },
                "error": { "message": "No code" }
            }
        ],
        "created": 1700000000,
        "model": "openai/gpt-4o",
        "object": "chat.completion"
    });

    let response: CompletionsResponse = serde_json::from_value(payload).unwrap();
    let codes: Vec<Option<&ErrorCode>> = response
        .choices
        .iter()
        .map(|choice| choice.error().and_then(|error| error.code.as_ref()))
        .collect();

    assert_eq!(codes[0], Some(&ErrorCode::Number(502)));
    assert_eq!(codes[0].and_then(ErrorCode::as_i64), Some(502));
    assert_eq!(
        codes[1],
        Some(&ErrorCode::Text("rate_limit_exceeded".to_string()))
    );
    assert_eq!(codes[1].and_then(ErrorCode::as_i64), None);
    assert_eq!(codes[1].unwrap().to_string(), "rate_limit_exceeded");
    assert_eq!(codes[2], None);
}

#[test]
fn test_tool_call_without_type_defaults_to_function() {
    let payload = response_with_choice(json!({
        "finish_reason": "tool_calls",
        "index": 0,
        "message": {
            "role": "assistant",
            "content": null,
            "tool_calls": [{
                "id": "call_1",
                "function": { "name": "get_weather", "arguments": "{\"city\":\"Paris\"}" }
            }]
        }
    }));

    let response: CompletionsResponse = serde_json::from_value(payload).unwrap();
    let calls: &[ToolCall] = response.choices[0].tool_calls().unwrap();

    assert_eq!(calls[0].tool_type(), "function");
    assert_eq!(calls[0].name(), "get_weather");
}

#[test]
fn test_cost_details_accept_null_upstream_costs() {
    let payload = json!({
        "id": "gen-123",
        "choices": [{ "finish_reason": "stop", "index": 0, "message": { "role": "assistant", "content": "ok" } }],
        "created": 1700000000,
        "model": "openai/gpt-4o",
        "object": "chat.completion",
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 2,
            "total_tokens": 12,
            "cost": 0.0001,
            "cost_details": {
                "upstream_inference_cost": null,
                "upstream_inference_prompt_cost": null,
                "upstream_inference_completions_cost": null
            }
        }
    });

    let response: CompletionsResponse = serde_json::from_value(payload).unwrap();
    let cost_details = response.usage.unwrap().cost_details.unwrap();

    assert_eq!(cost_details.upstream_inference_prompt_cost, None);
    assert_eq!(cost_details.upstream_inference_completions_cost, None);

    let missing: ResponseCostDetails = serde_json::from_value(json!({})).unwrap();
    assert_eq!(missing.upstream_inference_prompt_cost, None);
    assert_eq!(serde_json::to_value(missing).unwrap(), json!({}));
}

#[test]
fn test_invalid_chat_choice_does_not_fall_back_to_text() {
    let error = serde_json::from_value::<Choice>(json!({
        "message": { "tool_calls": "invalid" }, "text": "fallback"
    }))
    .unwrap_err()
    .to_string();
    assert!(error.contains("invalid chat choice"), "{error}");
}

#[test]
fn test_numeric_string_error_codes_preserve_retry_status() {
    use http::StatusCode;
    use openrouter_rs::{api::errors::parse_api_error, error::OpenRouterError};

    let error = parse_api_error(
        StatusCode::OK,
        None,
        r#"{"error":{"code":" 429 ","message":"Slow down"}}"#,
    );
    let OpenRouterError::Api(error) = error else {
        panic!("expected API error")
    };
    assert_eq!(error.api_code, Some(429));
    assert_eq!(error.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(error.is_retryable());
    assert_eq!(ErrorCode::Text("429".into()).as_i64(), Some(429));
}
