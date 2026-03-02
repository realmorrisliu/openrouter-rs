use std::{collections::HashSet, env};

use openrouter_rs::{
    api::embeddings::{EmbeddingRequest, EmbeddingVector},
    error::OpenRouterError,
};

use super::test_utils::{create_test_client, rate_limit_delay};

const MAX_EMBEDDING_MODEL_PROBES: usize = 12;

fn configured_embeddings_model() -> Option<String> {
    env::var("OPENROUTER_TEST_EMBEDDINGS_MODEL")
        .ok()
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_list_embedding_models_live() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let models = client.models().list_embedding_models().await?;
    assert!(
        !models.is_empty(),
        "embedding model list should not be empty"
    );
    assert!(
        models
            .iter()
            .any(|model| !model.id.trim().is_empty() && !model.name.trim().is_empty()),
        "at least one embedding model should include id and name"
    );

    println!("Embeddings models test passed: {} models", models.len());
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_create_embedding_live() -> Result<(), OpenRouterError> {
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
            "Return a deterministic embedding for this short integration sentence.",
        );

        match client.models().create_embedding(&request).await {
            Ok(response) => {
                assert!(
                    !response.object.trim().is_empty(),
                    "response.object should exist"
                );
                assert!(
                    !response.model.trim().is_empty(),
                    "response.model should exist"
                );
                assert!(
                    !response.data.is_empty(),
                    "response.data should not be empty"
                );

                assert!(
                    response.data.iter().any(|item| match &item.embedding {
                        EmbeddingVector::Float(values) => !values.is_empty(),
                        EmbeddingVector::Base64(value) => !value.trim().is_empty(),
                    }),
                    "at least one embedding item should contain non-empty vector payload"
                );

                println!(
                    "Embeddings create test passed (model={}, vectors={})",
                    response.model,
                    response.data.len()
                );
                return Ok(());
            }
            Err(error) => {
                last_failure = Some(format!("{model} => {error}"));
            }
        }
    }

    panic!(
        "failed to create embedding after trying {attempted}/{MAX_EMBEDDING_MODEL_PROBES} candidate models; last failure: {}",
        last_failure.unwrap_or_else(|| "no candidate models were available".to_string())
    );
}
