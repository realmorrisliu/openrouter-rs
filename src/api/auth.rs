use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use surf::http::headers::AUTHORIZATION;

use crate::{error::OpenRouterError, types::ApiResponse, utils::handle_error};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthRequest {
    code: String,
    code_verifier: Option<String>,
    code_challenge_method: Option<CodeChallengeMethod>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CodeChallengeMethod {
    #[serde(rename = "S256")]
    S256,
    Plain,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub key: String,
    pub user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UsageLimitType {
    Daily,
    Weekly,
    Monthly,
}

/// Request payload for `POST /auth/keys/code`.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
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
}

impl CreateAuthCodeRequest {
    pub fn builder() -> CreateAuthCodeRequestBuilder {
        CreateAuthCodeRequestBuilder::default()
    }
}

/// Response payload for `POST /auth/keys/code`.
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    let url = format!("{base_url}/auth/keys");
    let request = AuthRequest {
        code: code.to_string(),
        code_verifier: code_verifier.map(|s| s.to_string()),
        code_challenge_method,
    };

    let mut response = surf::post(url).body_json(&request)?.await?;

    if response.status().is_success() {
        let auth_response = response.body_json().await?;
        Ok(auth_response)
    } else {
        handle_error(response).await?;
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
    let url = format!("{base_url}/auth/keys/code");
    let mut response = surf::post(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .body_json(request)?
        .await?;

    if response.status().is_success() {
        let payload: ApiResponse<AuthCodeData> = response.body_json().await?;
        Ok(payload.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
