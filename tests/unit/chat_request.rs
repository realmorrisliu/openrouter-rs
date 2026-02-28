use openrouter_rs::{
    api::chat::{
        CacheControl, CacheControlType, ChatCompletionRequest, ContentPart, DebugOptions, Message,
        Modality, Plugin, StopSequence, StreamOptions, TraceOptions,
    },
    types::{Effort, Role},
};
use serde_json::json;

#[test]
fn test_reasoning_effort_extended_values_serialize() {
    let efforts = vec![
        (Effort::Xhigh, "xhigh"),
        (Effort::High, "high"),
        (Effort::Medium, "medium"),
        (Effort::Low, "low"),
        (Effort::Minimal, "minimal"),
        (Effort::None, "none"),
    ];

    for (effort, expected) in efforts {
        let request = ChatCompletionRequest::builder()
            .model("openai/gpt-5")
            .messages(vec![Message::new(Role::User, "test")])
            .reasoning_effort(effort)
            .build()
            .expect("request should build");

        let json = serde_json::to_value(&request).expect("request should serialize");
        assert_eq!(json["reasoning"]["effort"], expected);
    }
}

#[test]
fn test_text_content_part_cache_control_serialization() {
    let request = ChatCompletionRequest::builder()
        .model("anthropic/claude-sonnet-4.5")
        .messages(vec![Message::with_parts(
            Role::User,
            vec![
                ContentPart::text("prefix"),
                ContentPart::text_with_cache_control(
                    "HUGE TEXT BODY",
                    CacheControl::ephemeral_with_ttl("1h"),
                ),
                ContentPart::cacheable_text("another block"),
            ],
        )])
        .build()
        .expect("request should build");

    let json = serde_json::to_value(&request).expect("request should serialize");
    let parts = json["messages"][0]["content"]
        .as_array()
        .expect("content should be multipart");

    assert!(parts[0].get("cache_control").is_none());
    assert_eq!(parts[1]["cache_control"]["type"], "ephemeral");
    assert_eq!(parts[1]["cache_control"]["ttl"], "1h");
    assert_eq!(parts[2]["cache_control"]["type"], "ephemeral");
    assert!(parts[2]["cache_control"].get("ttl").is_none());
}

#[test]
fn test_text_content_part_cache_control_deserialization() {
    let json = r#"{
        "type": "text",
        "text": "cached",
        "cache_control": {
            "type": "ephemeral",
            "ttl": "1h"
        }
    }"#;

    let part: ContentPart = serde_json::from_str(json).expect("content part should deserialize");

    match part {
        ContentPart::Text {
            text,
            cache_control,
        } => {
            assert_eq!(text, "cached");
            let cache_control = cache_control.expect("cache control should be present");
            assert!(matches!(cache_control.kind, CacheControlType::Ephemeral));
            assert_eq!(cache_control.ttl.as_deref(), Some("1h"));
        }
        _ => panic!("expected text content part"),
    }
}

#[test]
fn test_chat_request_extended_control_fields_serialize() {
    let trace = TraceOptions {
        trace_id: Some("trace-1".to_string()),
        span_name: Some("sdk.chat".to_string()),
        generation_name: None,
        trace_name: None,
        parent_span_id: None,
        extra: [("team".to_string(), json!("rust-sdk"))]
            .into_iter()
            .collect(),
    };

    let request = ChatCompletionRequest::builder()
        .model("openai/gpt-5")
        .messages(vec![Message::new(Role::User, "ping")])
        .user("user-123")
        .session_id("session-abc")
        .metadata([("env", "test"), ("feature", "chat-parity")])
        .trace(trace)
        .stop(StopSequence::Multiple(vec![
            "END".to_string(),
            "DONE".to_string(),
        ]))
        .build()
        .expect("request should build");

    let json = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(json["user"], "user-123");
    assert_eq!(json["session_id"], "session-abc");
    assert_eq!(json["metadata"]["env"], "test");
    assert_eq!(json["metadata"]["feature"], "chat-parity");
    assert_eq!(json["trace"]["trace_id"], "trace-1");
    assert_eq!(json["trace"]["span_name"], "sdk.chat");
    assert_eq!(json["trace"]["team"], "rust-sdk");
    assert_eq!(json["stop"][0], "END");
    assert_eq!(json["stop"][1], "DONE");
}

#[test]
fn test_chat_request_extended_generation_fields_serialize() {
    let request = ChatCompletionRequest::builder()
        .model("openai/gpt-5")
        .messages(vec![Message::new(Role::User, "generate")])
        .max_completion_tokens(512)
        .logprobs(true)
        .modalities(vec![Modality::Text, Modality::Image])
        .image_config([("aspect_ratio", json!("16:9")), ("n", json!(1))])
        .build()
        .expect("request should build");

    let json = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(json["max_completion_tokens"], 512);
    assert_eq!(json["logprobs"], true);
    assert_eq!(json["modalities"][0], "text");
    assert_eq!(json["modalities"][1], "image");
    assert_eq!(json["image_config"]["aspect_ratio"], "16:9");
    assert_eq!(json["image_config"]["n"], 1);
}

#[test]
fn test_chat_request_plugins_and_stream_options_serialize() {
    let request = ChatCompletionRequest::builder()
        .model("openai/gpt-5")
        .messages(vec![Message::new(Role::User, "search the web")])
        .plugins(vec![
            Plugin::new("web")
                .option("max_results", 3)
                .option("search_prompt", "latest rust release"),
        ])
        .stream_options(StreamOptions {
            include_usage: Some(true),
        })
        .debug(DebugOptions {
            echo_upstream_body: Some(true),
        })
        .build()
        .expect("request should build");

    let json = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(json["plugins"][0]["id"], "web");
    assert_eq!(json["plugins"][0]["max_results"], 3);
    assert_eq!(json["plugins"][0]["search_prompt"], "latest rust release");
    assert_eq!(json["stream_options"]["include_usage"], true);
    assert_eq!(json["debug"]["echo_upstream_body"], true);
}
