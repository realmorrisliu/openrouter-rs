use openrouter_rs::OpenRouterClient;
use std::time::Duration;

#[test]
fn test_builder_accepts_custom_http_client() {
    let custom = reqwest::Client::builder()
        .timeout(Duration::from_secs(7))
        .build()
        .expect("custom reqwest::Client should build");

    let client = OpenRouterClient::builder()
        .api_key("test-api-key")
        .http_client(custom)
        .build();

    assert!(client.is_ok(), "builder should accept a custom http_client");
}

#[test]
fn test_builder_falls_back_to_default_http_client_when_omitted() {
    let client = OpenRouterClient::builder().api_key("test-api-key").build();

    assert!(
        client.is_ok(),
        "builder should use default http_client when none provided"
    );
}
