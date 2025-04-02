use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;

use crate::{error::OpenRouterError, setter, types::ApiResponse, utils::handle_error};

#[derive(Serialize, Deserialize, Debug)]
pub struct CoinbaseChargeRequest {
    amount: f64,
    sender: String,
    chain_id: u32,
}

#[derive(Default)]
pub struct CoinbaseChargeRequestBuilder {
    amount: Option<f64>,
    sender: Option<String>,
    chain_id: Option<u32>,
}

impl CoinbaseChargeRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    setter!(amount, f64);
    setter!(sender, into String);
    setter!(chain_id, u32);

    pub fn build(self) -> Result<CoinbaseChargeRequest, OpenRouterError> {
        Ok(CoinbaseChargeRequest {
            amount: self
                .amount
                .ok_or(OpenRouterError::Validation("amount is required".into()))?,
            sender: self
                .sender
                .ok_or(OpenRouterError::Validation("sender is required".into()))?,
            chain_id: self
                .chain_id
                .ok_or(OpenRouterError::Validation("chain_id is required".into()))?,
        })
    }
}

impl CoinbaseChargeRequest {
    pub fn builder() -> CoinbaseChargeRequestBuilder {
        CoinbaseChargeRequestBuilder::new()
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
    let url = format!("{}/credits/coinbase", base_url);

    let mut response = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .body_json(request)?
        .await?;

    if response.status().is_success() {
        let charge_response: ApiResponse<CoinbaseChargeData> = response.body_json().await?;
        Ok(charge_response.data)
    } else {
        handle_error(response).await?;
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
    let url = format!("{}/credits", base_url);

    let mut response = surf::get(url)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .await?;

    if response.status().is_success() {
        let credits_response: ApiResponse<_> = response.body_json().await?;
        Ok(credits_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
