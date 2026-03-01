use super::test_utils::{create_test_client, rate_limit_delay};
use openrouter_rs::error::OpenRouterError;

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_get_current_api_key_info() -> Result<(), OpenRouterError> {
    let client = create_test_client()?;
    rate_limit_delay().await;

    let key_info = client.get_current_api_key_info().await?;

    assert!(!key_info.label.is_empty(), "API key should have a label");
    assert!(key_info.usage >= 0.0, "Usage should be non-negative");
    assert!(
        !key_info.rate_limit.requests.is_nan(),
        "Request limit should be a valid number"
    );

    println!("Current API key info: {key_info:?}");
    Ok(())
}
