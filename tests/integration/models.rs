use openrouter_rs::error::OpenRouterError;

use super::test_utils::{create_test_client, rate_limit_delay};

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_list_models() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;

    rate_limit_delay().await;

    let models = client.list_models().await?;

    assert!(!models.is_empty(), "model list should not be empty");
    assert!(
        models.iter().any(|m| !m.id.is_empty()),
        "each model should have an ID"
    );

    println!("first model: {:?}", models[0]);

    Ok(())
}
