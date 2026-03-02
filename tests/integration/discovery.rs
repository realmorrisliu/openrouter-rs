use openrouter_rs::error::OpenRouterError;

use super::test_utils::{create_test_client, rate_limit_delay};

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_list_providers_live() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let providers = client.models().list_providers().await?;
    assert!(!providers.is_empty(), "provider list should not be empty");
    assert!(
        providers
            .iter()
            .any(|provider| !provider.slug.trim().is_empty()),
        "at least one provider should have a slug"
    );

    println!(
        "Discovery providers test passed: {} providers",
        providers.len()
    );
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_list_user_models_live() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let models = client.models().list_user_models().await?;
    assert!(!models.is_empty(), "models/user should not be empty");
    assert!(
        models
            .iter()
            .any(|model| !model.id.trim().is_empty() && !model.canonical_slug.trim().is_empty()),
        "at least one user model should include id and canonical_slug"
    );

    println!("Discovery user-models test passed: {} models", models.len());
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_count_models_live() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let count = client.models().get_model_count().await?;
    assert!(count.count > 0, "models/count should be > 0");

    println!("Discovery models/count test passed: {}", count.count);
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_list_zdr_endpoints_live() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let endpoints = client.models().list_zdr_endpoints().await?;
    assert!(!endpoints.is_empty(), "endpoints/zdr should not be empty");
    assert!(
        endpoints.iter().any(|endpoint| {
            !endpoint.model_id.trim().is_empty()
                && !endpoint.model_name.trim().is_empty()
                && !endpoint.provider_name.trim().is_empty()
        }),
        "at least one ZDR endpoint should include model/provider identifiers"
    );

    println!(
        "Discovery endpoints/zdr test passed: {} endpoints",
        endpoints.len()
    );
    Ok(())
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_list_model_endpoints_live() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let models = client.models().list().await?;
    assert!(!models.is_empty(), "models list should not be empty");

    let mut last_failure = None;
    let mut attempted = 0usize;

    for model in &models {
        let Some((author, slug)) = model.id.split_once('/') else {
            continue;
        };
        if author.trim().is_empty() || slug.trim().is_empty() {
            continue;
        }

        attempted += 1;
        rate_limit_delay().await;
        match client.models().list_endpoints(author, slug).await {
            Ok(endpoint_data) => {
                assert!(
                    !endpoint_data.id.trim().is_empty(),
                    "endpoint data id should not be empty"
                );
                assert!(
                    !endpoint_data.name.trim().is_empty(),
                    "endpoint data name should not be empty"
                );
                assert!(
                    !endpoint_data.endpoints.is_empty(),
                    "endpoint data should contain at least one endpoint"
                );
                assert!(
                    endpoint_data.endpoints.iter().any(|endpoint| {
                        !endpoint.name.trim().is_empty()
                            && !endpoint.provider_name.trim().is_empty()
                    }),
                    "at least one endpoint entry should include name/provider_name"
                );

                println!(
                    "Discovery model-endpoints test passed for model {} ({} endpoints)",
                    model.id,
                    endpoint_data.endpoints.len()
                );
                return Ok(());
            }
            Err(error) => {
                last_failure = Some(format!("{} => {error}", model.id));
            }
        }
    }

    panic!(
        "failed to validate model endpoints after trying {attempted} models; last failure: {}",
        last_failure.unwrap_or_else(|| "no parseable model ids were found".to_string()),
    );
}
