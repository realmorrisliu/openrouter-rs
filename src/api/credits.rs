use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::transport::RequestContext;
use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ApiResponse,
};

#[derive(Serialize, Deserialize, Debug, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
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
#[non_exhaustive]
pub struct CoinbaseChargeData {
    pub addresses: HashMap<String, String>,
    pub calldata: HashMap<String, String>,
    pub chain_id: u32,
    pub sender: String,
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct CreditsData {
    pub total_credits: f64,
    pub total_usage: f64,
}

pub(crate) async fn create_coinbase_charge_with_client(
    ctx: &RequestContext<'_>,
    request: &CoinbaseChargeRequest,
) -> Result<CoinbaseChargeData, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/credits/coinbase");

    let req =
        transport_request::with_bearer_auth(transport_request::post(http_client, &url), api_key)
            .json(request);
    let charge_response: ApiResponse<CoinbaseChargeData> =
        transport_response::send_json(req, "coinbase charge").await?;
    Ok(charge_response.data)
}

pub(crate) async fn get_credits_with_client(
    ctx: &RequestContext<'_>,
) -> Result<CreditsData, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/credits");

    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let credits_response: ApiResponse<_> = transport_response::send_json(req, "credits").await?;
    Ok(credits_response.data)
}
