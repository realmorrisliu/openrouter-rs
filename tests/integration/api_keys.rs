use super::test_utils::{create_test_client, rate_limit_delay};
use openrouter_rs::error::OpenRouterError;

#[tokio::test]
async fn test_get_current_api_key_info() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let key_info = client.get_current_api_key_info().await?;

    assert!(!key_info.label.is_empty(), "API key should have a label");
    assert!(key_info.usage >= 0.0, "Usage should be non-negative");
    assert!(
        key_info.rate_limit.requests > 0.0,
        "Should have positive request limit"
    );

    println!("Current API key info: {:?}", key_info);
    Ok(())
}
