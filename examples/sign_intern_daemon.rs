//! Requires an OAuth access token from `ori login --oidc` with `vault:read` in its grant.
//! Supply it through OPENROUTER_OIDC_ACCESS_TOKEN; ordinary API keys cannot sign.

use openrouter_rs::{OpenRouterClient, api::interns::SignInternDaemonRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let intern_id = args
        .next()
        .ok_or("provide intern ID and body SHA-256 hex digest")?;
    let digest = args.next().ok_or("provide body SHA-256 hex digest")?;
    let session_token = std::env::var("OPENROUTER_OIDC_ACCESS_TOKEN")?;
    let client = OpenRouterClient::builder()
        // The existing api_key slot forwards this OAuth credential as a bearer token.
        .api_key(session_token)
        .build()?;
    let request = SignInternDaemonRequest::builder()
        .body_sha256(digest)
        .build()?;
    let signed = client.interns().sign_daemon(&intern_id, &request).await?;
    // Forward signed.signature, signed.timestamp and signed.user as x-ori-invoke-* headers.
    println!("Received daemon signature at {}", signed.timestamp);
    Ok(())
}
