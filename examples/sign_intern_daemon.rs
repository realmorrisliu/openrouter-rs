use openrouter_rs::{OpenRouterClient, api::interns::SignInternDaemonRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let intern_id = args
        .next()
        .ok_or("provide intern ID and body SHA-256 hex digest")?;
    let digest = args.next().ok_or("provide body SHA-256 hex digest")?;
    let client = OpenRouterClient::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY")?)
        .build()?;
    let request = SignInternDaemonRequest::builder()
        .body_sha256(digest)
        .build()?;
    let signed = client.interns().sign_daemon(&intern_id, &request).await?;
    // Forward signed.signature, signed.timestamp and signed.user as x-ori-invoke-* headers.
    println!("Received daemon signature at {}", signed.timestamp);
    Ok(())
}
