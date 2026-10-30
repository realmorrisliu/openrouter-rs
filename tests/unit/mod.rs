#[test]
fn dummy_test() {
    // This is a placeholder test
}

pub mod analytics;
pub mod api_keys;
pub mod audio;
pub mod auth;
pub mod byok;
pub mod chat_api;
pub mod chat_request;
pub mod client_domains;
#[cfg(feature = "legacy-completions")]
pub mod client_legacy;
pub mod client_management_key;
pub mod completion;
pub mod completion_leniency;
pub mod credits;
pub mod custom_http_client;
pub mod default_headers;
pub mod discovery;
pub mod embeddings;
pub mod error_model;
pub mod files;
pub mod generation;
pub mod guardrails;
pub mod images;
pub mod messages;
pub mod models;
pub mod observability;
pub mod organization;
pub mod pagination;
pub mod presets;
pub mod provider;
pub mod reasoning_fields;
pub mod rerank;
pub mod response_format;
pub mod responses;
pub mod scim;
pub mod stream;
pub mod tool_builder;
pub mod unified_stream;
pub mod upstream_drift_surface;
pub mod videos;
pub mod workspaces;

pub mod openapi_issue_251;
pub mod openapi_issue_273;
pub mod openapi_september;

/// Client bound to a local test server.
///
/// Sets both credentials so a transport test can exercise either an API-key
/// endpoint or a management-key endpoint through the same helper.
pub fn test_client(base_url: &str, credential: &str) -> openrouter_rs::OpenRouterClient {
    openrouter_rs::OpenRouterClient::builder()
        .base_url(base_url)
        .api_key(credential)
        .management_key(credential)
        .build()
        .expect("test client should build")
}

/// Client bound to a local test server with per-request attribution metadata.
pub fn test_client_with_attribution(
    base_url: &str,
    credential: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
) -> openrouter_rs::OpenRouterClient {
    let mut builder = openrouter_rs::OpenRouterClient::builder();
    builder
        .base_url(base_url)
        .api_key(credential)
        .management_key(credential);
    if let Some(value) = x_title {
        builder.x_title(value.clone());
    }
    if let Some(value) = http_referer {
        builder.http_referer(value.clone());
    }
    if let Some(values) = app_categories {
        builder.app_categories(values.iter().cloned());
    }
    builder.build().expect("test client should build")
}
