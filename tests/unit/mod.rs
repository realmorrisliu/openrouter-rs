#[test]
fn dummy_test() {
    // This is a placeholder test
}

pub mod api_keys;
pub mod auth;
pub mod chat_request;
pub mod client_domains;
#[cfg(feature = "legacy-completions")]
pub mod client_legacy;
pub mod client_management_key;
pub mod completion;
pub mod config;
pub mod discovery;
pub mod embeddings;
pub mod error_model;
pub mod guardrails;
pub mod messages;
pub mod pagination;
pub mod provider;
pub mod response_format;
pub mod responses;
pub mod stream;
pub mod unified_stream;
