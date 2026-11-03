use openrouter_rs::{
    OpenRouterClient,
    api::responses::{AccessPrograms, ResponsesRequest},
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;

    let mut builder = ResponsesRequest::builder();
    builder.model("openai/gpt-5").input(json!([{
        "role": "user",
        "content": "Say hello in one sentence."
    }]));

    // OpenAI Daybreak programs (`standard`, `daybreak_blue`, `daybreak_red`) are
    // only accepted on BYOK requests running on your own enrolled OpenAI key, so
    // this stays unset for ordinary OpenRouter OpenAI credentials.
    if let Ok(program) = std::env::var("OPENROUTER_OPENAI_ACCESS_PROGRAM") {
        builder.access_programs(AccessPrograms::builder().cyber(program).build()?);
    }

    let request = builder.build()?;

    let response = client.responses().create(&request).await?;
    println!("response id: {:?}", response.id);
    println!("status: {:?}", response.status);

    Ok(())
}
