pub(crate) mod client;
pub(crate) mod multipart;
pub(crate) mod request;
#[cfg(feature = "reqwest")]
mod reqwest_backend;
pub(crate) mod response;
pub(crate) mod sse;
#[cfg(all(feature = "wasip3", target_os = "wasi"))]
mod wasip3_backend;

pub(crate) use client::{HttpClient, RequestBuilder, Response};
#[cfg(feature = "reqwest")]
pub(crate) use reqwest_backend::transport_error as reqwest_error;

use crate::error::OpenRouterError;

/// Builds the default HTTP client: the host's `wasi:http` when the `wasip3` feature targets WASI,
/// otherwise reqwest.
pub(crate) fn new_client() -> Result<HttpClient, OpenRouterError> {
    #[cfg(all(feature = "wasip3", target_os = "wasi"))]
    {
        Ok(HttpClient::Wasip3)
    }
    #[cfg(all(feature = "reqwest", not(all(feature = "wasip3", target_os = "wasi"))))]
    {
        reqwest::Client::builder()
            .build()
            .map(HttpClient::Reqwest)
            .map_err(OpenRouterError::from)
    }
}

/// Per-request client configuration.
///
/// Bundles the values every endpoint function needs so signatures do not repeat
/// the same argument clump, and so a new configuration field does not force a
/// signature change everywhere.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RequestContext<'a> {
    pub http_client: &'a HttpClient,
    pub base_url: &'a str,
    pub credential: &'a str,
    pub x_title: &'a Option<String>,
    pub http_referer: &'a Option<String>,
    pub app_categories: &'a Option<Vec<String>>,
}
