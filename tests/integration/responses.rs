use std::{env, time::Duration};

use futures_util::StreamExt;
use openrouter_rs::{
    error::OpenRouterError,
    types::stream::{UnifiedStreamEvent, UnifiedStreamSource},
};
use serde_json::json;

use super::test_utils::{create_test_client, rate_limit_delay};

const DEFAULT_RESPONSES_MODEL: &str = "openai/gpt-4o-mini";

fn test_responses_model() -> String {
    env::var("OPENROUTER_TEST_RESPONSES_MODEL")
        .ok()
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
        .unwrap_or_else(|| DEFAULT_RESPONSES_MODEL.to_string())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_create_response_non_streaming() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let model = test_responses_model();
    let request = openrouter_rs::api::responses::ResponsesRequest::builder()
        .model(model.clone())
        .input(json!([{
            "role": "user",
            "content": "Reply with a short sentence that includes the word Rust."
        }]))
        .max_output_tokens(80)
        .temperature(0.0)
        .build()?;

    let response = client.responses().create(&request).await?;

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
        "response.object should be present"
    );
    assert!(
        response
            .status
            .as_deref()
            .is_some_and(|status| !status.trim().is_empty()),
        "response.status should be present"
    );
    assert!(
        response
            .output
            .as_ref()
            .is_some_and(|output| !output.is_empty()),
        "response.output should be non-empty"
    );

    println!(
        "Responses non-stream test passed (model={model}, id={})",
        response.id.as_deref().unwrap_or("<missing>")
    );
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_stream_response_unified_done_semantics() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let model = test_responses_model();
    let request = openrouter_rs::api::responses::ResponsesRequest::builder()
        .model(model.clone())
        .input(json!([{
            "role": "user",
            "content": "Give a brief greeting."
        }]))
        .max_output_tokens(60)
        .temperature(0.0)
        .build()?;

    let mut stream = client.responses().stream_unified(&request).await?;

    let (saw_payload_event, saw_done) = tokio::time::timeout(Duration::from_secs(90), async move {
        let mut saw_payload_event = false;
        let mut saw_done = false;

        while let Some(event) = stream.next().await {
            match event {
                UnifiedStreamEvent::Error(err) => return Err(err),
                UnifiedStreamEvent::Done { source, .. } => {
                    assert_eq!(
                        source,
                        UnifiedStreamSource::Responses,
                        "terminal event source should be responses"
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
                UnifiedStreamEvent::Raw { .. } => {
                    saw_payload_event = true;
                }
            }
        }

        Ok::<(bool, bool), OpenRouterError>((saw_payload_event, saw_done))
    })
    .await
    .expect("timed out waiting for responses stream completion")?;

    assert!(
        saw_payload_event,
        "stream should emit at least one payload event"
    );
    assert!(saw_done, "stream should emit terminal done event");

    println!("Responses stream test passed (model={model})");
    Ok(())
}
