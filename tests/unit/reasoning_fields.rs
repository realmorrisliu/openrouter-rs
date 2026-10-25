//! Reasoning fields used by OpenAI-compatible upstreams, and the completion token breakdown.

use openrouter_rs::{
    api::chat::Message,
    types::{
        Role,
        completion::{Choice, CompletionsResponse},
    },
};
use serde_json::json;

fn reasoning_response() -> CompletionsResponse {
    serde_json::from_value(json!({
        "id": "gen-123",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "4",
                "reasoning_content": "2 + 2 is 4.",
                "reasoning_text": "Adding two and two."
            }
        }],
        "created": 1700000000,
        "model": "qwen/qwen3-32b",
        "object": "chat.completion",
        "usage": {
            "prompt_tokens": 12,
            "completion_tokens": 30,
            "total_tokens": 42,
            "completion_tokens_details": { "reasoning_tokens": 24, "image_tokens": 0 }
        }
    }))
    .expect("a response with reasoning fields should decode")
}

#[test]
fn test_response_message_decodes_reasoning_fields() {
    let response = reasoning_response();
    let message = response.choices[0].message().unwrap();

    assert_eq!(message.reasoning_content.as_deref(), Some("2 + 2 is 4."));
    assert_eq!(
        message.reasoning_text.as_deref(),
        Some("Adding two and two.")
    );
    assert_eq!(message.reasoning, None);
}

#[test]
fn test_usage_decodes_completion_tokens_details() {
    let usage = reasoning_response().usage.unwrap();
    let details = usage.completion_tokens_details.unwrap();

    assert_eq!(details.reasoning_tokens, Some(24));
    assert_eq!(details.image_tokens, Some(0));
    assert_eq!(details.audio_tokens, None);
}

#[test]
fn test_response_reasoning_fields_replay_on_the_next_request() {
    let response = reasoning_response();
    let replayed = Message::try_from(response.choices[0].message().unwrap()).unwrap();

    assert_eq!(replayed.reasoning_content.as_deref(), Some("2 + 2 is 4."));
    let wire = serde_json::to_value(&replayed).unwrap();
    assert_eq!(wire["reasoning_content"], "2 + 2 is 4.");
    assert_eq!(wire["reasoning_text"], "Adding two and two.");
}

#[test]
fn test_request_message_reasoning_fields_serialize_only_when_set() {
    let plain = serde_json::to_value(Message::new(Role::Assistant, "4")).unwrap();
    assert!(plain.get("reasoning_content").is_none());
    assert!(plain.get("reasoning_text").is_none());

    let message = Message::new(Role::Assistant, "4")
        .with_reasoning_content("2 + 2 is 4.")
        .with_reasoning_text("Adding two and two.");
    let wire = serde_json::to_value(message).unwrap();
    assert_eq!(wire["reasoning_content"], "2 + 2 is 4.");
    assert_eq!(wire["reasoning_text"], "Adding two and two.");
}

#[test]
fn test_streaming_delta_decodes_reasoning_content() {
    let choice: Choice = serde_json::from_value(json!({
        "index": 0,
        "delta": { "role": "assistant", "content": "", "reasoning_content": "Thinking", "reasoning_text": "Alternate" }
    }))
    .unwrap();

    assert_eq!(choice.reasoning(), None);
    let Choice::Streaming(choice) = choice else {
        panic!("expected a streaming choice");
    };
    assert_eq!(choice.delta.reasoning_content.as_deref(), Some("Thinking"));
    assert_eq!(choice.delta.reasoning_text.as_deref(), Some("Alternate"));
}
