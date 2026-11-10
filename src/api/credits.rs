use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

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

/// One agent credit purchase started from a workspace API key.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CreditPurchase {
    pub id: String,
    pub status: String,
    pub credit_amount_usd: String,
    pub platform_fee_usd: String,
    pub total_charged_usd: String,
    pub protocol: String,
    pub network: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_intent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    pub created_at: String,
}

/// Request body for `POST /credits/purchases`.
#[derive(Serialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateCreditPurchaseRequest {
    /// Amount in USD as a decimal string, so the amount stays exact.
    #[builder(setter(into))]
    pub credit_amount_usd: String,
}

impl CreateCreditPurchaseRequest {
    pub fn builder() -> CreateCreditPurchaseRequestBuilder {
        CreateCreditPurchaseRequestBuilder::default()
    }

    pub fn new(credit_amount_usd: impl Into<String>) -> Self {
        Self {
            credit_amount_usd: credit_amount_usd.into(),
        }
    }
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

/// Start a credit purchase (`POST /credits/purchases`).
///
/// The first call normally answers `402` with a payment challenge; retry with a
/// payment credential to get the `202` purchase this returns.
pub(crate) async fn create_credit_purchase_with_client(
    ctx: &RequestContext<'_>,
    request: &CreateCreditPurchaseRequest,
) -> Result<CreditPurchase, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/credits/purchases");
    let req =
        transport_request::with_bearer_auth(transport_request::post(http_client, &url), api_key)
            .json(request);
    let purchase: ApiResponse<CreditPurchase> =
        transport_response::send_json(req, "credit purchase").await?;
    Ok(purchase.data)
}

/// Poll one credit purchase (`GET /credits/purchases/{id}`).
pub(crate) async fn get_credit_purchase_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
) -> Result<CreditPurchase, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/credits/purchases/{}", encode(id));
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let purchase: ApiResponse<CreditPurchase> =
        transport_response::send_json(req, "credit purchase lookup").await?;
    Ok(purchase.data)
}
