use openrouter_rs::OpenRouterClient;
use openrouter_rs::api::auth::CodeChallengeMethod;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");
    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let auth_response = client
        .management()
        .create_api_key_from_auth_code(
            "your_authorization_code",
            Some("your_code_verifier"),
            Some(CodeChallengeMethod::S256),
        )
        .await?;
    println!("{auth_response:?}");

    Ok(())
}
