pub(crate) mod request;
pub(crate) mod response;
pub(crate) mod sse;

use reqwest::Client;

use crate::error::OpenRouterError;

pub(crate) fn new_client() -> Result<Client, OpenRouterError> {
    reqwest::Client::builder()
        .build()
        .map_err(OpenRouterError::from)
}
