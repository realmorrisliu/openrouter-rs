use dotenvy_macro::dotenv;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::{Choice, ResponseFormat, Role},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = dotenv!("OPENROUTER_API_KEY");
    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs")
        .build();

    let format = ResponseFormat::json_schema(
        "character_info",
        true,
        serde_json::json!({
          "type": "object",
          "properties": {
            "name": {
              "type": "string",
              "description": "Name of the character",
            },
            "school": {
              "type": "string",
              "description": "School of the character",
            },
            "hair_color": {
              "type": "string",
              "description": "Hair color of the character",
            },
          },
        }),
    );

    let chat_request = ChatCompletionRequest::builder()
        .model("google/gemini-2.5-pro-exp-03-25:free")
        .messages(vec![Message::new(Role::User, "Who is Harry Potter?")])
        .response_format(format)
        .build()?;

    let chat_response = client.send_chat_completion(&chat_request).await?;

    println!("=== Chat Response:");
    println!("{:?}", chat_response);

    println!("=== Structured Response:");
    for choice in chat_response.choices {
        if let Choice::NonStreaming(non_streaming_choice) = choice {
            if let Some(content) = non_streaming_choice.message.content {
                println!("{}", content);
            }
        }
    }

    Ok(())
}
