use openrouter_rs::{
    api::chat::{CacheControl, CacheControlType, ChatCompletionRequest, ContentPart, Message},
    types::{Effort, Role},
};

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
