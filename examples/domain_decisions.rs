use openrouter_rs::{OpenRouterClient, api::decisions::DecisionsRequest};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenRouterClient::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY")?)
        .build()?;
    let request = DecisionsRequest::builder()
        .model("typesafe/jev-1.13")
        .state(json!({"issue": "A user reports that a request fails."}))
        .questions([(
            "is_bug",
            json!({
                "type": "noul",
                "instructions": "Is the report describing a software defect?",
                "criteria": {
                    "true": "The user describes broken or unexpected behavior.",
                    "false": "The user asks a question or requests a feature."
                }
            }),
        )])
        .build()?;
    let response = client.decisions().create(&request).await?;
    println!("{:?}", response.answers);
    Ok(())
}
