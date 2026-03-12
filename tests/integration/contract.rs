use std::{collections::HashSet, env, time::Duration};

use futures_util::StreamExt;
use openrouter_rs::{
    api::{
        chat::{ChatCompletionRequestBuilder, Message},
        embeddings::{EmbeddingRequest, EmbeddingVector},
        messages::{AnthropicMessage, AnthropicMessagesRequest},
        responses::ResponsesRequest,
    },
    error::OpenRouterError,
    types::{
        Role,
        stream::{UnifiedStream, UnifiedStreamEvent, UnifiedStreamSource},
    },
};
use serde_json::json;

use super::{
    model_pool::test_chat_model,
    test_utils::{create_test_client, rate_limit_delay},
};

const DEFAULT_RESPONSES_MODEL: &str = "openai/gpt-4o-mini";
const MAX_MODEL_ENDPOINT_PROBES: usize = 8;
const MAX_EMBEDDING_MODEL_PROBES: usize = 8;

fn test_responses_model() -> String {
    env::var("OPENROUTER_TEST_RESPONSES_MODEL")
        .ok()
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
        .unwrap_or_else(|| DEFAULT_RESPONSES_MODEL.to_string())
}

fn test_messages_model() -> String {
    env::var("OPENROUTER_TEST_MESSAGES_MODEL")
        .ok()
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
        .unwrap_or_else(test_chat_model)
}

fn configured_embeddings_model() -> Option<String> {
    env::var("OPENROUTER_TEST_EMBEDDINGS_MODEL")
        .ok()
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
}

async fn assert_stream_completes(
    mut stream: UnifiedStream,
    source: UnifiedStreamSource,
) -> Result<(), OpenRouterError> {
    let (saw_payload, saw_done) = tokio::time::timeout(Duration::from_secs(90), async move {
        let mut saw_payload = false;
        let mut saw_done = false;

        while let Some(event) = stream.next().await {
            match event {
                UnifiedStreamEvent::Error(err) => return Err(err),
                UnifiedStreamEvent::Done {
                    source: done_source,
                    ..
                } => {
                    assert_eq!(done_source, source, "terminal event source should match");
                    saw_done = true;
                    break;
                }
                UnifiedStreamEvent::ContentDelta(delta) => {
                    if !delta.trim().is_empty() {
                        saw_payload = true;
                    }
                }
                UnifiedStreamEvent::ReasoningDelta(delta) => {
                    if !delta.trim().is_empty() {
                        saw_payload = true;
                    }
                }
                UnifiedStreamEvent::ReasoningDetailsDelta(details) => {
                    if !details.is_empty() {
                        saw_payload = true;
                    }
                }
                UnifiedStreamEvent::ToolDelta(_) | UnifiedStreamEvent::Raw { .. } => {
                    saw_payload = true;
                }
            }
        }

        Ok::<(bool, bool), OpenRouterError>((saw_payload, saw_done))
    })
    .await
    .expect("timed out waiting for unified stream completion")?;

    assert!(saw_payload, "stream should emit at least one payload event");
    assert!(saw_done, "stream should emit a terminal done event");
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_read_only_contract_metadata_surface() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;

    rate_limit_delay().await;
    let key_info = client.get_current_api_key_info().await?;
    assert!(
        !key_info.label.trim().is_empty(),
        "API key label should be present"
    );
    assert!(key_info.usage >= 0.0, "usage should be non-negative");

    rate_limit_delay().await;
    let models = client.models().list().await?;
    assert!(!models.is_empty(), "model list should not be empty");

    rate_limit_delay().await;
    let user_models = client.models().list_user_models().await?;
    assert!(
        !user_models.is_empty(),
        "user models list should not be empty"
    );

    rate_limit_delay().await;
    let count = client.models().get_model_count().await?;
    assert!(count.count > 0, "model count should be positive");

    rate_limit_delay().await;
    let providers = client.models().list_providers().await?;
    assert!(!providers.is_empty(), "provider list should not be empty");
    assert!(
        providers
            .iter()
            .any(|provider| !provider.slug.trim().is_empty()),
        "at least one provider should include a slug"
    );

    let mut last_failure = None;
    let mut attempted = 0usize;
    for model in &models {
        let Some((author, slug)) = model.id.split_once('/') else {
            continue;
        };
        if author.trim().is_empty() || slug.trim().is_empty() {
            continue;
        }
        if attempted >= MAX_MODEL_ENDPOINT_PROBES {
            break;
        }

        attempted += 1;
        rate_limit_delay().await;
        match client.models().list_endpoints(author, slug).await {
            Ok(endpoint_data) => {
                assert!(
                    !endpoint_data.id.trim().is_empty(),
                    "endpoint metadata id should not be empty"
                );
                assert!(
                    !endpoint_data.endpoints.is_empty(),
                    "endpoint metadata should include at least one provider endpoint"
                );
                return Ok(());
            }
            Err(err) => {
                last_failure = Some(format!("{} => {err}", model.id));
            }
        }
    }

    panic!(
        "failed to validate model endpoints after trying {attempted}/{MAX_MODEL_ENDPOINT_PROBES} models; last failure: {}",
        last_failure.unwrap_or_else(|| "no parseable model ids were available".to_string()),
    );
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_read_only_contract_chat_surface() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    let model = test_chat_model();

    let request = ChatCompletionRequestBuilder::default()
        .model(model.clone())
        .messages(vec![Message::new(
            Role::User,
            "Reply with a short greeting that includes the word Rust.",
        )])
        .max_tokens(64)
        .temperature(0.0)
        .build()?;

    rate_limit_delay().await;
    let response = client.chat().create(&request).await?;
    assert!(
        !response.id.trim().is_empty(),
        "chat response id should be present"
    );
    assert!(
        !response.model.trim().is_empty(),
        "chat response model should be present"
    );
    assert!(
        !response.choices.is_empty(),
        "chat response should include at least one choice"
    );

    rate_limit_delay().await;
    let stream = client.chat().stream_unified(&request).await?;
    assert_stream_completes(stream, UnifiedStreamSource::Chat).await?;
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_read_only_contract_responses_and_messages_surface() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;

    let responses_model = test_responses_model();
    let responses_request = ResponsesRequest::builder()
        .model(responses_model.clone())
        .input(json!([{
            "role": "user",
            "content": "Reply with a short sentence that includes the word Rust."
        }]))
        .max_output_tokens(80)
        .temperature(0.0)
        .build()?;

    rate_limit_delay().await;
    let response = client.responses().create(&responses_request).await?;
    assert!(
        response
            .id
            .as_deref()
            .is_some_and(|id| !id.trim().is_empty()),
        "responses API should return an id"
    );

    rate_limit_delay().await;
    let responses_stream = client
        .responses()
        .stream_unified(&responses_request)
        .await?;
    assert_stream_completes(responses_stream, UnifiedStreamSource::Responses).await?;

    let messages_model = test_messages_model();
    let messages_request = AnthropicMessagesRequest::builder()
        .model(messages_model.clone())
        .max_tokens(120)
        .messages(vec![AnthropicMessage::user(
            "Reply with a short greeting that includes the word Rust.",
        )])
        .build()?;

    rate_limit_delay().await;
    let message = client.messages().create(&messages_request).await?;
    assert!(
        message
            .id
            .as_deref()
            .is_some_and(|id| !id.trim().is_empty()),
        "messages API should return an id"
    );
    assert!(
        !message.content.is_empty(),
        "messages content should not be empty"
    );

    rate_limit_delay().await;
    let messages_stream = client.messages().stream_unified(&messages_request).await?;
    assert_stream_completes(messages_stream, UnifiedStreamSource::Messages).await?;

    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_read_only_contract_embeddings_surface() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;

    rate_limit_delay().await;
    let models = client.models().list_embedding_models().await?;
    assert!(
        !models.is_empty(),
        "embedding model list should not be empty"
    );

    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    if let Some(model) = configured_embeddings_model() {
        seen.insert(model.clone());
        candidates.push(model);
    }

    for model in &models {
        let id = model.id.trim();
        if id.is_empty() {
            continue;
        }
        if seen.insert(id.to_string()) {
            candidates.push(id.to_string());
        }
        if candidates.len() >= MAX_EMBEDDING_MODEL_PROBES {
            break;
        }
    }

    let mut last_failure = None;
    let mut attempted = 0usize;
    for model in candidates {
        attempted += 1;
        rate_limit_delay().await;
        let request = EmbeddingRequest::new(
            model.clone(),
            "Return a deterministic embedding for this short contract test sentence.",
        );

        match client.models().create_embedding(&request).await {
            Ok(response) => {
                assert!(
                    !response.object.trim().is_empty(),
                    "embedding response object should be present"
                );
                assert!(
                    response.data.iter().any(|item| match &item.embedding {
                        EmbeddingVector::Float(values) => !values.is_empty(),
                        EmbeddingVector::Base64(value) => !value.trim().is_empty(),
                    }),
                    "embedding response should include at least one non-empty vector"
                );
                return Ok(());
            }
            Err(err) => {
                last_failure = Some(format!("{model} => {err}"));
            }
        }
    }

    panic!(
        "failed to validate embeddings after trying {attempted}/{MAX_EMBEDDING_MODEL_PROBES} candidate models; last failure: {}",
        last_failure.unwrap_or_else(|| "no candidate embedding models were available".to_string()),
    );
}
