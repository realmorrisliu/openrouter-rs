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

/// One agent credit purchase made by an organization.
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
    pub failure_reason: Option<String>,
    pub created_at: String,
}

/// Result of an unpaid request or its paid retry.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreditPurchaseOutcome {
    PaymentRequired {
        /// Each WWW-Authenticate value is a separate payment offer.
        challenges: Vec<String>,
        problem: CreditPurchaseProblem,
        /// Encoded x402 challenge from PAYMENT-REQUIRED.
        payment_required: Option<String>,
        /// Purchase URL to save before signing an x402 challenge.
        location: Option<String>,
    },
    Accepted {
        purchase: CreditPurchase,
        location: Option<String>,
        payment_receipt: Option<String>,
        /// Encoded x402 settlement receipt, if returned.
        payment_response: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct CreditPurchaseProblem {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub setup: String,
    pub detail: Option<String>,
    #[serde(rename = "challengeId")]
    pub challenge_id: Option<String>,
}

/// Public Markdown guides for agent credit purchases.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum CreditPurchaseGuide {
    Overview,
    Setup,
    Skill,
}

pub(crate) async fn get_credit_purchase_guide(
    ctx: &RequestContext<'_>,
    guide: CreditPurchaseGuide,
) -> Result<String, OpenRouterError> {
    let suffix = match guide {
        CreditPurchaseGuide::Overview => "",
        CreditPurchaseGuide::Setup => "/setup",
        CreditPurchaseGuide::Skill => "/SKILL.md",
    };
    let url = format!("{}/credits/purchases{suffix}", ctx.base_url);
    let response = transport_request::get(ctx.http_client, &url).send().await?;
    if response.status().is_success() {
        Ok(response.text().await?)
    } else {
        Err(transport_response::error_from_response(response).await)
    }
}

/// Request body for `POST /credits/purchases`.
#[derive(Serialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateCreditPurchaseRequest {
    /// Amount in USD as a decimal string, so the amount stays exact.
    #[builder(setter(into))]
    pub credit_amount_usd: String,
    /// Payment protocol. Omitting it uses the service default (MPP).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub protocol: Option<String>,
}

impl CreateCreditPurchaseRequest {
    pub fn builder() -> CreateCreditPurchaseRequestBuilder {
        CreateCreditPurchaseRequestBuilder::default()
    }

    pub fn new(credit_amount_usd: impl Into<String>) -> Self {
        Self {
            credit_amount_usd: credit_amount_usd.into(),
            protocol: None,
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

pub(crate) async fn create_credit_purchase_with_client(
    ctx: &RequestContext<'_>,
    request: &CreateCreditPurchaseRequest,
    idempotency_key: &str,
    payment_credential: Option<&str>,
) -> Result<CreditPurchaseOutcome, OpenRouterError> {
    let url = format!("{}/credits/purchases", ctx.base_url);
    let mut req = transport_request::with_bearer_auth(
        transport_request::post(ctx.http_client, &url),
        ctx.credential,
    )
    .header("Idempotency-Key", idempotency_key)
    .json(request);
    if let Some(payment) = payment_credential {
        let name = match request.protocol.as_deref() {
            None | Some("mpp") => "Payment-Authorization",
            Some("x402") => "Payment-Signature",
            Some(_) => {
                return Err(OpenRouterError::ConfigError(
                    "Cannot select a payment credential header for an unknown protocol".into(),
                ));
            }
        };
        let mut header = http::HeaderValue::from_str(payment)
            .map_err(|_| OpenRouterError::ConfigError(format!("Invalid {name} header")))?;
        header.set_sensitive(true);
        req = req.header(name, header);
    }
    let response = req.send().await?;
    let header = |name: &str| {
        response
            .headers()
            .get(name)?
            .to_str()
            .ok()
            .map(str::to_owned)
    };
    let location = header("location");
    if response.status() == http::StatusCode::PAYMENT_REQUIRED {
        let payment_required = header("payment-required");
        let challenges = response
            .headers()
            .get_all(http::header::WWW_AUTHENTICATE)
            .iter()
            .filter_map(|value| value.to_str().ok().map(str::to_owned))
            .collect();
        let problem =
            transport_response::parse_json_response(response, "credit purchase challenge").await?;
        return Ok(CreditPurchaseOutcome::PaymentRequired {
            challenges,
            problem,
            payment_required,
            location,
        });
    }
    let payment_response = header("payment-response");
    let payment_receipt = header("payment-receipt");
    let purchase: ApiResponse<CreditPurchase> =
        transport_response::parse_or_error(response, "credit purchase").await?;
    Ok(CreditPurchaseOutcome::Accepted {
        purchase: purchase.data,
        location,
        payment_receipt,
        payment_response,
    })
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
