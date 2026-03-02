use std::{env, time::Duration};

use futures_util::StreamExt;
use openrouter_rs::{
    api::messages::{AnthropicMessage, AnthropicMessagesRequest},
    error::OpenRouterError,
    types::stream::{UnifiedStreamEvent, UnifiedStreamSource},
};

use super::{
    model_pool::test_chat_model,
    test_utils::{create_test_client, rate_limit_delay},
};

fn test_messages_model() -> String {
    env::var("OPENROUTER_TEST_MESSAGES_MODEL")
        .ok()
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
        .unwrap_or_else(test_chat_model)
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_create_message_non_streaming() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let model = test_messages_model();
    let request = AnthropicMessagesRequest::builder()
        .model(model.clone())
        .max_tokens(120)
        .messages(vec![AnthropicMessage::user(
            "Please reply with a short greeting that includes the word Rust.",
        )])
        .build()?;

    let response = client.messages().create(&request).await?;

    assert!(
        response
            .id
            .as_deref()
            .is_some_and(|id| !id.trim().is_empty()),
        "response.id should be present"
    );
    assert!(
        response
            .object_type
            .as_deref()
            .is_some_and(|kind| !kind.trim().is_empty()),
        "response.type should be present"
    );
    assert!(
        response
            .role
            .as_deref()
            .is_some_and(|role| !role.trim().is_empty()),
        "response.role should be present"
    );
    assert!(
        !response.content.is_empty(),
        "response.content should be non-empty"
    );

    println!(
        "Messages non-stream test passed (model={model}, id={})",
        response.id.as_deref().unwrap_or("<missing>")
    );
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_stream_messages_unified_done_semantics() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let model = test_messages_model();
    let request = AnthropicMessagesRequest::builder()
        .model(model.clone())
        .max_tokens(120)
        .messages(vec![AnthropicMessage::user(
            "Give a brief greeting in English.",
        )])
        .build()?;

    let mut stream = client.messages().stream_unified(&request).await?;

    let (saw_payload_event, saw_done) = tokio::time::timeout(Duration::from_secs(90), async move {
        let mut saw_payload_event = false;
        let mut saw_done = false;

        while let Some(event) = stream.next().await {
            match event {
                UnifiedStreamEvent::Error(err) => return Err(err),
                UnifiedStreamEvent::Done { source, .. } => {
                    assert_eq!(
                        source,
                        UnifiedStreamSource::Messages,
                        "terminal event source should be messages"
                    );
                    saw_done = true;
                    break;
                }
                UnifiedStreamEvent::ContentDelta(delta) => {
                    if !delta.trim().is_empty() {
                        saw_payload_event = true;
                    }
                }
                UnifiedStreamEvent::ReasoningDelta(delta) => {
                    if !delta.trim().is_empty() {
                        saw_payload_event = true;
                    }
                }
                UnifiedStreamEvent::ReasoningDetailsDelta(details) => {
                    if !details.is_empty() {
                        saw_payload_event = true;
                    }
                }
                UnifiedStreamEvent::ToolDelta(_) => {
                    saw_payload_event = true;
                }
                UnifiedStreamEvent::Raw { source, .. } => {
                    assert_eq!(
                        source,
                        UnifiedStreamSource::Messages,
                        "raw event source should be messages"
                    );
                    saw_payload_event = true;
                }
            }
        }

        Ok::<(bool, bool), OpenRouterError>((saw_payload_event, saw_done))
    })
    .await
    .expect("timed out waiting for messages stream completion")?;

    assert!(
        saw_payload_event,
        "stream should emit at least one payload event"
    );
    assert!(saw_done, "stream should emit terminal done event");

    println!("Messages stream test passed (model={model})");
    Ok(())
}
