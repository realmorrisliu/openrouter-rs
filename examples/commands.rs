use dotenvy_macro::dotenv;
use futures_util::StreamExt;
use openrouter_rs::api::chat::{ChatCompletionRequest, Message};
use openrouter_rs::api::completion::CompletionRequest;
use openrouter_rs::command::*;
use openrouter_rs::types::Role;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");

    // Example: Set API Key
    set_api_key(api_key).await?;

    // Example: Send Chat Completion
    let chat_request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat:free")
        .messages(vec![Message::new(
            Role::User,
            "What is the meaning of life?",
        )])
        .max_tokens(100)
        .temperature(0.7)
        .build()?;

    let chat_response = send_chat_completion(chat_request).await?;
    println!("Chat Response: {:?}", chat_response);

    // Example: Stream Chat Completion
    let chat_request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat:free")
        .messages(vec![Message::new(
            Role::User,
            "What is the meaning of life?",
        )])
        .temperature(0.7)
        .build()?;

    let mut response = stream_chat_completion(chat_request).await?;

    while let Some(event) = response.next().await {
        match event {
            Ok(event) => println!("Stream Event: {:?}", event),
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    // Example: Send Completion Request
    let completion_request = CompletionRequest::builder()
        .model("deepseek/deepseek-chat:free")
        .prompt("Once upon a time")
        .max_tokens(100)
        .temperature(0.7)
        .build()?;

    let completion_response = send_completion(completion_request).await?;
    println!("Completion Response: {:?}", completion_response);

    // Example: Get Credits
    let credits = get_credits().await?;
    println!("Credits: {:?}", credits);

    // Example: Get Generation
    let generation_id = completion_response.id.unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    println!("Wait for completion");
    let generation_data = get_generation(&generation_id).await?;
    println!("Generation Data: {:?}", generation_data);

    // Example: List Models
    let models = list_models().await?;
    println!("Models: {:?}", models);

    // Example: List Model Endpoints
    let endpoints = list_model_endpoints("deepseek", "deepseek-chat:free").await?;
    println!("Endpoints: {:?}", endpoints);

    // Example: Check if Model is Enabled
    let is_enabled = is_model_enabled("deepseek/deepseek-chat:free").await?;
    println!("Is Model Enabled: {:?}", is_enabled);

    Ok(())
}
