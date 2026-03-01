use openrouter_rs::{OpenRouterClient, error::OpenRouterError};

#[tokio::test]
async fn test_create_api_key_requires_management_key() {
    let client = OpenRouterClient::builder()
        .api_key("user-api-key")
        .build()
        .expect("client should build");

    let result = client.create_api_key("new-key", Some(10.0)).await;
    assert!(
        matches!(result, Err(OpenRouterError::KeyNotConfigured)),
        "create_api_key should require management_key"
    );
}

#[tokio::test]
async fn test_guardrails_endpoints_require_management_key() {
    let client = OpenRouterClient::builder()
        .api_key("user-api-key")
        .build()
        .expect("client should build");

    let result = client.list_guardrails(None).await;
    assert!(
        matches!(result, Err(OpenRouterError::KeyNotConfigured)),
        "list_guardrails should require management_key"
    );
}

#[tokio::test]
async fn test_activity_endpoint_requires_management_key() {
    let client = OpenRouterClient::builder()
        .api_key("user-api-key")
        .build()
        .expect("client should build");

    let result = client.get_activity(None).await;
    assert!(
        matches!(result, Err(OpenRouterError::KeyNotConfigured)),
        "get_activity should require management_key"
    );
}

#[tokio::test]
async fn test_clear_management_key_removes_management_access() {
    let mut client = OpenRouterClient::builder()
        .management_key("mgmt-key")
        .build()
        .expect("client should build");
    client.clear_management_key();

    let result = client.delete_api_key("hash").await;
    assert!(
        matches!(result, Err(OpenRouterError::KeyNotConfigured)),
        "delete_api_key should fail after clearing management_key"
    );
}
