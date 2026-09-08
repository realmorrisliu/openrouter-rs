use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ApiResponse,
};

#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct AuthRequest {
    code: String,
    code_verifier: Option<String>,
    code_challenge_method: Option<CodeChallengeMethod>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum CodeChallengeMethod {
    #[serde(rename = "S256")]
    S256,
    Plain,
}

#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub struct AuthResponse {
    pub key: String,
    pub user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum UsageLimitType {
    Daily,
    Weekly,
    Monthly,
}

/// Request payload for `POST /auth/keys/code`.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateAuthCodeRequest {
    #[builder(setter(into))]
    callback_url: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    code_challenge: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    code_challenge_method: Option<CodeChallengeMethod>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<f64>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    key_label: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    usage_limit_type: Option<UsageLimitType>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    spawn_agent: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    spawn_cloud: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
}

impl CreateAuthCodeRequest {
    pub fn builder() -> CreateAuthCodeRequestBuilder {
        CreateAuthCodeRequestBuilder::default()
    }
}

/// Response payload for `POST /auth/keys/code`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AuthCodeData {
    pub id: String,
    pub app_id: f64,
    pub created_at: String,
}

/// Exchange an authorization code from the PKCE flow for a user-controlled API key
///
/// # Arguments
///
/// * `base_url` - The base URL of the OpenRouter API.
/// * `code` - The authorization code received from the OAuth redirect.
/// * `code_verifier` - The code verifier if code_challenge was used in the authorization request.
/// * `code_challenge_method` - The method used to generate the code challenge.
///
/// # Returns
///
/// * `Result<AuthResponse, OpenRouterError>` - The API key and user ID associated with the API key.
pub async fn exchange_code_for_api_key(
    base_url: &str,
    code: &str,
    code_verifier: Option<&str>,
    code_challenge_method: Option<CodeChallengeMethod>,
) -> Result<AuthResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    exchange_code_for_api_key_with_client(
        &http_client,
        base_url,
        code,
        code_verifier,
        code_challenge_method,
    )
    .await
}

pub(crate) async fn exchange_code_for_api_key_with_client(
    http_client: &HttpClient,
    base_url: &str,
    code: &str,
    code_verifier: Option<&str>,
    code_challenge_method: Option<CodeChallengeMethod>,
) -> Result<AuthResponse, OpenRouterError> {
    let url = format!("{base_url}/auth/keys");
    let request = AuthRequest {
        code: code.to_string(),
        code_verifier: code_verifier.map(|s| s.to_string()),
        code_challenge_method,
    };

    let response = transport_request::post(http_client, &url)
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let auth_response: AuthResponse =
            transport_response::parse_json_response(response, "auth key exchange").await?;
        Ok(auth_response)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Create an authorization code for PKCE flow (`POST /auth/keys/code`).
///
/// Returns an auth code ID that can be exchanged via [`exchange_code_for_api_key`].
pub async fn create_auth_code(
    base_url: &str,
    api_key: &str,
    request: &CreateAuthCodeRequest,
) -> Result<AuthCodeData, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_auth_code_with_client(&http_client, base_url, api_key, request).await
}

pub(crate) async fn create_auth_code_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    request: &CreateAuthCodeRequest,
) -> Result<AuthCodeData, OpenRouterError> {
    let url = format!("{base_url}/auth/keys/code");
    let response =
        transport_request::with_bearer_auth(transport_request::post(http_client, &url), api_key)
            .json(request)
            .send()
            .await?;

    if response.status().is_success() {
        let payload: ApiResponse<AuthCodeData> =
            transport_response::parse_json_response(response, "auth code creation").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// RFC 8693 workload identity token exchange request.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct OAuthTokenExchangeRequest {
    #[builder(setter(into))]
    pub federation_policy_id: String,
    #[builder(setter(into))]
    pub subject_token: String,
    #[builder(
        default = "String::from(\"urn:ietf:params:oauth:grant-type:token-exchange\")",
        setter(into)
    )]
    pub grant_type: String,
    #[builder(
        default = "String::from(\"urn:ietf:params:oauth:token-type:jwt\")",
        setter(into)
    )]
    pub subject_token_type: String,
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_token_type: Option<String>,
    #[builder(default, setter(into, strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl OAuthTokenExchangeRequest {
    pub fn builder() -> OAuthTokenExchangeRequestBuilder {
        OAuthTokenExchangeRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub issued_token_type: String,
    pub scope: String,
    pub token_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OAuthJwk {
    pub alg: String,
    pub crv: String,
    pub kid: String,
    pub kty: String,
    #[serde(rename = "use")]
    pub key_use: String,
    pub x: String,
    pub y: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OAuthJwksResponse {
    pub keys: Vec<OAuthJwk>,
}

/// Fetch the public signing keys (`GET /oauth/jwks`).
pub async fn get_oauth_jwks(base_url: &str) -> Result<OAuthJwksResponse, OpenRouterError> {
    get_oauth_jwks_with_client(&crate::transport::new_client()?, base_url).await
}

pub(crate) async fn get_oauth_jwks_with_client(
    http_client: &HttpClient,
    base_url: &str,
) -> Result<OAuthJwksResponse, OpenRouterError> {
    let response = transport_request::get(http_client, &format!("{base_url}/oauth/jwks"))
        .send()
        .await?;
    if response.status().is_success() {
        transport_response::parse_json_response(response, "OAuth signing keys").await
    } else {
        Err(transport_response::error_from_response(response).await)
    }
}

/// Exchange a workload JWT for an access token (`POST /oauth/token`).
pub async fn exchange_oauth_token(
    base_url: &str,
    request: &OAuthTokenExchangeRequest,
) -> Result<OAuthTokenResponse, OpenRouterError> {
    exchange_oauth_token_with_client(&crate::transport::new_client()?, base_url, request).await
}

pub(crate) async fn exchange_oauth_token_with_client(
    http_client: &HttpClient,
    base_url: &str,
    request: &OAuthTokenExchangeRequest,
) -> Result<OAuthTokenResponse, OpenRouterError> {
    let response = transport_request::post(http_client, &format!("{base_url}/oauth/token"))
        .form(request)
        .send()
        .await?;
    if response.status().is_success() {
        transport_response::parse_json_response(response, "OAuth token exchange").await
    } else {
        Err(transport_response::error_from_response(response).await)
    }
}
