use openrouter_rs::OpenRouterClient;
use openrouter_rs::api::auth::CodeChallengeMethod;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");
    let client = OpenRouterClient::new(api_key);

    let auth_response = client
        .exchange_code_for_api_key(
            "your_authorization_code",
            Some("your_code_verifier"),
            Some(CodeChallengeMethod::S256),
        )
        .await?;
    println!("{:?}", auth_response);

    Ok(())
}
