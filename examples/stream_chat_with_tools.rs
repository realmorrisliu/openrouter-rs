//! # Streaming with Tool Calls Example
//!
//! This example demonstrates how to use `ToolAwareStream` to handle streaming
//! responses that include tool calls. Content is printed in real time as it
//! arrives, while tool call fragments are automatically accumulated and
//! delivered as complete objects once the stream finishes.
//!
//! ## Usage
//!
//! ```bash
//! OPENROUTER_API_KEY=your_key cargo run --example stream_chat_with_tools
//! ```

use std::env;

use futures_util::StreamExt;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Content, Message},
    types::stream::StreamEvent,
    types::{Role, Tool, completion::FinishReason},
};
use serde_json::json;

// Keep reasoning objects in arrival order; do not merge or deduplicate indices.
// https://openrouter.ai/docs/guides/best-practices/reasoning-tokens#streaming-response
fn record_assistant_delta(message: &mut Message, event: &StreamEvent) {
    match event {
        StreamEvent::ContentDelta(text) => {
            if let Content::Text(content) = &mut message.content {
                content.push_str(text);
            }
        }
        StreamEvent::ReasoningDelta(text) => {
            message
                .reasoning
                .get_or_insert_with(String::new)
                .push_str(text);
        }
        StreamEvent::ReasoningDetailsDelta(details) => {
            message
                .reasoning_details
                .get_or_insert_with(Vec::new)
                .extend_from_slice(details);
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY environment variable not set");

    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs streaming tool calls example")
        .build()?;

    // Define a weather tool
    let weather_tool = Tool::builder()
        .name("get_weather")
        .description("Get current weather information for a location")
        .parameters(json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name, e.g. 'San Francisco, CA'"
                }
            },
            "required": ["location"]
        }))
        .build()?;

    let request = ChatCompletionRequest::builder()
        .model("openai/gpt-4o-mini")
        .messages(vec![
            Message::new(
                Role::System,
                "You are a helpful assistant. Use the get_weather tool when asked about weather.",
            ),
            Message::new(Role::User, "What's the weather in San Francisco and Tokyo?"),
        ])
        .tool(weather_tool)
        .tool_choice_auto()
        .max_tokens(500)
        .build()?;

    println!("--- Streaming with tool-aware processing ---\n");

    // Use the tool-aware stream wrapper
    let mut stream = client.chat().stream_tool_aware(&request).await?;

    let mut assistant_message = Message::new(Role::Assistant, "");
    while let Some(event) = stream.next().await {
        record_assistant_delta(&mut assistant_message, &event);
        match event {
            StreamEvent::ContentDelta(text) => {
                // Content is printed immediately as it streams in
                print!("{}", text);
            }
            StreamEvent::ReasoningDelta(text) => {
                print!("[reasoning] {}", text);
            }
            StreamEvent::ReasoningDetailsDelta(details) => {
                for detail in &details {
                    if let Some(content) = detail.content() {
                        print!("[detail] {}", content);
                    }
                }
            }
            StreamEvent::Done {
                tool_calls,
                finish_reason,
                usage,
                id,
                model,
            } => {
                println!();
                println!("\n--- Stream complete ---");
                println!("  ID: {}", id);
                println!("  Model: {}", model);

                if let Some(reason) = &finish_reason {
                    println!("  Finish reason: {:?}", reason);
                }

                if let Some(usage) = &usage {
                    println!(
                        "  Tokens: {} prompt + {} completion = {} total",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                    );
                }

                if !tool_calls.is_empty() {
                    println!("\n  Tool calls ({}):", tool_calls.len());
                    for (i, tc) in tool_calls.iter().enumerate() {
                        println!("    {}. {} (id: {})", i + 1, tc.name(), tc.id());
                        println!("       Arguments: {}", tc.arguments_json());
                    }

                    // If tools were called, simulate execution and send results back
                    if matches!(finish_reason, Some(FinishReason::ToolCalls)) {
                        println!("\n--- Simulating tool execution ---\n");

                        let mut messages = request.messages().to_vec();
                        // Add the assistant's response with tool calls
                        assistant_message.tool_calls = Some(tool_calls.clone());
                        messages.push(assistant_message.clone());

                        for tc in &tool_calls {
                            let result = format!("Weather in {}: Sunny, 22C", tc.arguments_json());
                            println!("  {} -> {}", tc.name(), result);
                            messages.push(Message::tool_response(tc.id(), result));
                        }

                        // Second round: stream the final response
                        println!("\n--- Streaming final response ---\n");

                        let follow_up = ChatCompletionRequest::builder()
                            .model("openai/gpt-4o-mini")
                            .messages(messages)
                            .max_tokens(500)
                            .build()?;

                        let mut stream2 = client.chat().stream_tool_aware(&follow_up).await?;

                        while let Some(event2) = stream2.next().await {
                            match event2 {
                                StreamEvent::ContentDelta(text) => print!("{}", text),
                                StreamEvent::Done { .. } => println!("\n\n--- Done ---"),
                                StreamEvent::Error(e) => eprintln!("\nError: {}", e),
                                _ => {}
                            }
                        }
                    }
                }
            }
            StreamEvent::Error(e) => {
                eprintln!("\nStream error: {}", e);
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_text_and_reasoning_in_arrival_order() {
        let first = json!([
            {"type": "reasoning.summary", "summary": "First", "index": 0},
            {"type": "reasoning.text", "text": "Thinking", "index": 1}
        ]);
        let second = json!([
            {"type": "reasoning.text", "text": " more", "signature": "signed", "index": 1},
            {"type": "reasoning.encrypted", "data": "encrypted", "index": 2}
        ]);
        let mut message = Message::new(Role::Assistant, "");
        for event in [
            StreamEvent::ContentDelta("Let me ".into()),
            StreamEvent::ReasoningDelta("Check ".into()),
            StreamEvent::ReasoningDetailsDelta(serde_json::from_value(first.clone()).unwrap()),
            StreamEvent::ContentDelta("check.".into()),
            StreamEvent::ReasoningDelta("weather".into()),
            StreamEvent::ReasoningDetailsDelta(serde_json::from_value(second.clone()).unwrap()),
        ] {
            record_assistant_delta(&mut message, &event);
        }
        message.tool_calls = Some(vec![openrouter_rs::types::ToolCall::new(
            "call-1", "weather", "{}",
        )]);
        let request = ChatCompletionRequest::builder()
            .model("test/model")
            .messages(vec![message, Message::tool_response("call-1", "Sunny")])
            .build()
            .unwrap();
        let payload = serde_json::to_value(request).unwrap();
        let replay = &payload["messages"][0];
        assert_eq!(replay["content"], "Let me check.");
        assert_eq!(replay["reasoning"], "Check weather");
        assert_eq!(
            replay["reasoning_details"],
            json!([first[0], first[1], second[0], second[1]])
        );
        assert_eq!(replay["tool_calls"][0]["id"], "call-1");
    }
}
