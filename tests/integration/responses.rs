use std::{env, time::Duration};

use futures_util::StreamExt;
use openrouter_rs::{
    api::responses::{ResponsesRequest, ResponsesResponse},
    error::OpenRouterError,
    types::stream::{UnifiedStreamEvent, UnifiedStreamSource},
};
use serde_json::{Value, json};

use super::{
    model_pool::{hot_responses_models, integration_tier_name, should_run_hot_responses_sweep},
    test_utils::{create_test_client, rate_limit_delay},
};

const DEFAULT_RESPONSES_MODEL: &str = "openai/gpt-4o-mini";

fn test_responses_model() -> String {
    env::var("OPENROUTER_TEST_RESPONSES_MODEL")
        .ok()
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
        .unwrap_or_else(|| DEFAULT_RESPONSES_MODEL.to_string())
}

fn hot_responses_request(model: &str) -> Result<ResponsesRequest, OpenRouterError> {
    let mut builder = ResponsesRequest::builder();
    builder.model(model.to_string());
    builder.instructions(
        "Return a plain-text final answer only. Do not call tools or use external actions.",
    );
    builder.input(json!([{
        "role": "user",
        "content": "Reply with exactly: hot-model-check"
    }]));
    builder.max_output_tokens(64);
    builder.temperature(0.0);
    builder.parallel_tool_calls(false);
    builder.store(false);

    if model.starts_with("x-ai/grok-4.20") {
        builder.reasoning(json!({ "enabled": false }));
    }

    builder.build()
}

fn collect_responses_output_text(value: &Value, buffer: &mut String) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_responses_output_text(item, buffer);
            }
        }
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str).is_some_and(|kind| {
                matches!(
                    kind,
                    "output_text" | "text" | "reasoning" | "reasoning_text"
                )
            }) {
                if let Some(text) = map.get("text").and_then(Value::as_str) {
                    buffer.push_str(text);
                }
                if let Some(text) = map.get("content").and_then(Value::as_str) {
                    buffer.push_str(text);
                }
                if let Some(text) = map.get("reasoning").and_then(Value::as_str) {
                    buffer.push_str(text);
                }
            }

            for value in map.values() {
                if value.is_array() || value.is_object() {
                    collect_responses_output_text(value, buffer);
                }
            }
        }
        _ => {}
    }
}

fn validate_responses_output_for_model(response: &ResponsesResponse) -> Result<(), String> {
    let id = response.id.as_deref().unwrap_or_default().trim();
    if id.is_empty() {
        return Err("missing response ID".to_string());
    }

    let status = response.status.as_deref().unwrap_or_default().trim();
    if matches!(status, "failed" | "cancelled" | "incomplete") {
        return Err(format!("responses status {status}"));
    }

    let output = response
        .output
        .as_ref()
        .ok_or_else(|| "missing response output".to_string())?;
    if output.is_empty() {
        return Err("empty response output".to_string());
    }

    let mut text = String::new();
    for item in output {
        collect_responses_output_text(item, &mut text);
    }

    if text.trim().is_empty() {
        return Err("no output_text or reasoning text in response output".to_string());
    }

    Ok(())
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

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_hot_responses_model_sweep() -> Result<(), OpenRouterError> {
    if !should_run_hot_responses_sweep() {
        println!(
            "Skipping hot responses model sweep because OPENROUTER_INTEGRATION_TIER={} (expected: hot)",
            integration_tier_name()
        );
        return Ok(());
    }

    let models = hot_responses_models();
    assert!(
        !models.is_empty(),
        "hot responses model list should not be empty when tier=hot"
    );

    let client = create_test_client()?;
    let mut failures = Vec::new();

    println!(
        "Running hot responses model sweep across {} models",
        models.len()
    );

    for model in models {
        rate_limit_delay().await;
        let request = hot_responses_request(&model)?;

        match client.responses().create(&request).await {
            Ok(response) => {
                if let Err(reason) = validate_responses_output_for_model(&response) {
                    failures.push(format!("{model}: {reason}"));
                } else {
                    println!("Hot responses model check passed: {model}");
                }
            }
            Err(err) => failures.push(format!("{model}: {err}")),
        }
    }

    assert!(
        failures.is_empty(),
        "hot responses model sweep had failures:\n{}",
        failures.join("\n")
    );

    Ok(())
}
