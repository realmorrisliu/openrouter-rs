use std::collections::HashMap;

use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ApiResponse,
};

#[derive(Serialize, Deserialize, Debug, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct CoinbaseChargeRequest {
    amount: f64,
    #[builder(setter(into))]
    sender: String,
    chain_id: u32,
}

impl CoinbaseChargeRequest {
    pub fn builder() -> CoinbaseChargeRequestBuilder {
        CoinbaseChargeRequestBuilder::default()
    }

    pub fn new(amount: f64, sender: &str, chain_id: u32) -> Self {
        Self::builder()
            .amount(amount)
            .sender(sender)
            .chain_id(chain_id)
            .build()
            .expect("Failed to build CoinbaseChargeRequest")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoinbaseChargeData {
    pub addresses: HashMap<String, String>,
    pub calldata: HashMap<String, String>,
    pub chain_id: u32,
    pub sender: String,
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreditsData {
    pub total_credits: f64,
    pub total_usage: f64,
}

/// Creates and hydrates a Coinbase Commerce charge for cryptocurrency payments
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
/// * `request` - The request data for creating a Coinbase charge.
///
/// # Returns
///
/// * `Result<CoinbaseChargeResponse, OpenRouterError>` - The response data containing the charge details.
pub async fn create_coinbase_charge(
    base_url: &str,
    api_key: &str,
    request: &CoinbaseChargeRequest,
) -> Result<CoinbaseChargeData, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_coinbase_charge_with_client(&http_client, base_url, api_key, request).await
}

pub(crate) async fn create_coinbase_charge_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    request: &CoinbaseChargeRequest,
) -> Result<CoinbaseChargeData, OpenRouterError> {
    let url = format!("{base_url}/credits/coinbase");

    let response =
        transport_request::with_bearer_auth(transport_request::post(http_client, &url), api_key)
            .json(request)
            .send()
            .await?;

    if response.status().is_success() {
        let charge_response: ApiResponse<CoinbaseChargeData> =
            transport_response::parse_json_response(response, "coinbase charge").await?;
        Ok(charge_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Returns the total credits purchased and used for the authenticated user
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `api_key` - The API key for authentication.
///
/// # Returns
///
/// * `Result<CreditsData, OpenRouterError>` - The response data containing the total credits and usage.
pub async fn get_credits(base_url: &str, api_key: &str) -> Result<CreditsData, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_credits_with_client(&http_client, base_url, api_key).await
}

pub(crate) async fn get_credits_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
) -> Result<CreditsData, OpenRouterError> {
    let url = format!("{base_url}/credits");

    let response =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?;

    if response.status().is_success() {
        let credits_response: ApiResponse<_> =
            transport_response::parse_json_response(response, "credits").await?;
        Ok(credits_response.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
