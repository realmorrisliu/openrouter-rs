use dotenvy_macro::dotenv;
use openrouter_rs::OpenRouterClient;
use openrouter_rs::api::auth::CodeChallengeMethod;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder().api_key(api_key).build();

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
