use serde::{Deserialize, Serialize};

use crate::{error::OpenRouterError, utils::handle_error};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthRequest {
    code: String,
    code_verifier: Option<String>,
    code_challenge_method: Option<CodeChallengeMethod>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CodeChallengeMethod {
    S256,

    #[serde(rename_all = "lowercase")]
    Plain,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub key: String,
    pub user_id: Option<String>,
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
