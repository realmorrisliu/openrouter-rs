use crate::{error::OpenRouterError, utils::handle_error};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CoinbaseChargeRequest {
    amount: f64,
    sender: String,
    chain_id: u32,
}

impl CoinbaseChargeRequest {
    pub fn new(amount: f64, sender: &str, chain_id: u32) -> Self {
        Self {
            amount,
            sender: sender.to_string(),
            chain_id,
        }
    }

    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = amount;
        self
    }

    pub fn sender(mut self, sender: &str) -> Self {
        self.sender = sender.to_string();
        self
    }

    pub fn chain_id(mut self, chain_id: u32) -> Self {
        self.chain_id = chain_id;
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoinbaseChargeResponse {
    data: CoinbaseChargeData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoinbaseChargeData {
    addresses: std::collections::HashMap<String, String>,
    calldata: std::collections::HashMap<String, String>,
    chain_id: u32,
    sender: String,
    id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreditsResponse {
    data: CreditsData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreditsData {
    total_credits: f64,
    total_usage: f64,
}

/// Creates and hydrates a Coinbase Commerce charge for cryptocurrency payments
///
/// # Arguments
///
/// * `client` - The HTTP client to use for the request.
/// * `api_key` - The API key for authentication.
/// * `request` - The request data for creating a Coinbase charge.
///
/// # Returns
///
/// * `Result<CoinbaseChargeResponse, OpenRouterError>` - The response data containing the charge details.
pub async fn create_coinbase_charge(
    client: &Client,
    api_key: &str,
    request: &CoinbaseChargeRequest,
) -> Result<CoinbaseChargeResponse, OpenRouterError> {
    let url = "https://openrouter.ai/api/v1/credits/coinbase";

    let response = client
        .post(url)
        .bearer_auth(api_key)
        .json(request)
        .send()
        .await?;

    if response.status().is_success() {
        let charge_response = response.json::<CoinbaseChargeResponse>().await?;
        Ok(charge_response)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

/// Returns the total credits purchased and used for the authenticated user
///
/// # Arguments
///
/// * `client` - The HTTP client to use for the request.
/// * `api_key` - The API key for authentication.
///
/// # Returns
///
/// * `Result<CreditsData, OpenRouterError>` - The response data containing the total credits and usage.
pub async fn get_credits(client: &Client, api_key: &str) -> Result<CreditsData, OpenRouterError> {
    let url = "https://openrouter.ai/api/v1/credits";

    let response = client.get(url).bearer_auth(api_key).send().await?;

    if response.status().is_success() {
        let credits_response = response.json::<CreditsResponse>().await?;
        Ok(credits_response.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
