#[test]
fn dummy_test() {
    // This is a placeholder test
}

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
pub mod credits;
pub mod custom_http_client;
pub mod default_headers;
pub mod discovery;
pub mod embeddings;
pub mod error_model;
pub mod generation;
pub mod guardrails;
pub mod messages;
pub mod models;
pub mod observability;
pub mod organization;
pub mod pagination;
pub mod provider;
pub mod rerank;
pub mod response_format;
pub mod responses;
pub mod stream;
pub mod tool_builder;
pub mod unified_stream;
pub mod videos;
pub mod workspaces;
