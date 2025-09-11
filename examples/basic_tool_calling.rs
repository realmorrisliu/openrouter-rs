//! # Basic Tool Calling Example
//!
//! This example demonstrates the fundamental tool calling capabilities of the OpenRouter SDK.
//! It shows how to define tools, send requests with tool support, and handle tool call responses.
//!
//! ## Features Demonstrated
//!
//! - Tool definition with JSON schema parameters
//! - Chat completion request with tools
//! - Tool call response handling
//! - Multi-turn conversation with tool results
//!
//! ## Tools Used
//!
//! - **Calculator**: Performs basic arithmetic operations
//! - **Weather**: Gets weather information for a location
//!
//! ## Usage
//!
//! ```bash
//! OPENROUTER_API_KEY=your_key cargo run --example basic_tool_calling
//! ```
//!
//! ## Key Features Shown
//!
//! - Proper conversation flow with tool calls
//! - Assistant messages include tool_calls array
//! - Tool responses reference tool_call_id correctly
//! - Multi-turn conversation handling

use std::env;
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::{Role, Tool},
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable not set");
    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs tool calling example")
        .build()?;

    // Define a calculator tool
    let calculator_tool = Tool::builder()
        .name("calculator")
        .description("Perform basic arithmetic operations")
        .parameters(json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "The arithmetic operation to perform"
                },
                "a": {
                    "type": "number",
                    "description": "First number"
                },
                "b": {
                    "type": "number", 
                    "description": "Second number"
                }
            },
            "required": ["operation", "a", "b"]
        }))
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
                    "description": "The city and state or country, e.g. 'San Francisco, CA' or 'London, UK'"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "Temperature unit preference",
                    "default": "fahrenheit"
                }
            },
            "required": ["location"]
        }))
        .build()?;

    println!("🔧 Defined tools:");
    println!("- Calculator: Performs basic arithmetic");
    println!("- Weather: Gets weather for locations");
    println!();

    // Create initial request with tools
    let request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3.1:free")  // Using a free model that supports tools
        .messages(vec![
            Message::new(Role::System, "You are a helpful assistant that can perform calculations and get weather information. Use the available tools when needed."),
            Message::new(Role::User, "What's 15 * 7? Also, what's the weather like in San Francisco?")
        ])
        .tools(vec![calculator_tool, weather_tool])
        .tool_choice_auto()  // Let the model decide when to use tools
        .max_tokens(1000)
        .build()?;

    println!("📤 Sending request with tools...");
    let response = client.send_chat_completion(&request).await?;

    // Check if the model wants to call tools
    if let Some(choice) = response.choices.first() {
        if let Some(tool_calls) = choice.tool_calls() {
            println!("🛠️  Model wants to call {} tool(s):", tool_calls.len());
            
            let mut messages = request.messages().clone();
            
            // Add the assistant's response (with tool calls) to conversation
            let content = choice.content().unwrap_or("");
            let assistant_message = Message::assistant_with_tool_calls(content, tool_calls.to_vec());
            messages.push(assistant_message);

            // Process each tool call
            for (i, tool_call) in tool_calls.iter().enumerate() {
                println!("  {}. Calling '{}' with arguments: {}", 
                    i + 1, 
                    tool_call.function.name, 
                    tool_call.function.arguments
                );

                // Simulate tool execution
                let tool_result = match tool_call.function.name.as_str() {
                    "calculator" => {
                        // Parse arguments and perform calculation
                        let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)?;
                        let operation = args["operation"].as_str().unwrap_or("add");
                        let a = args["a"].as_f64().unwrap_or(0.0);
                        let b = args["b"].as_f64().unwrap_or(0.0);
                        
                        let result = match operation {
                            "add" => a + b,
                            "subtract" => a - b,
                            "multiply" => a * b,
                            "divide" => if b != 0.0 { a / b } else { f64::NAN },
                            _ => f64::NAN,
                        };
                        
                        format!("The result of {} {} {} is {}", a, operation, b, result)
                    },
                    "get_weather" => {
                        // Simulate weather API call
                        let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)?;
                        let location = args["location"].as_str().unwrap_or("Unknown");
                        let unit = args["unit"].as_str().unwrap_or("fahrenheit");
                        
                        // Mock weather data
                        let temp = if unit == "celsius" { "22°C" } else { "72°F" };
                        format!("Current weather in {}: Sunny, {}, light breeze", location, temp)
                    },
                    _ => "Unknown tool".to_string(),
                };

                println!("     ✅ Result: {}", tool_result);

                // Add tool response to messages
                messages.push(Message::tool_response(&tool_call.id, &tool_result));
            }

            // Send follow-up request with tool results - keep tools available in case model needs more
            println!("\n📤 Sending follow-up request with tool results...");
            let follow_up_request = ChatCompletionRequest::builder()
                .model("deepseek/deepseek-chat-v3.1:free")
                .messages(messages.clone())
                .tools(vec![
                    Tool::builder()
                        .name("get_weather")
                        .description("Get current weather information for a location")
                        .parameters(json!({
                            "type": "object",
                            "properties": {
                                "location": {
                                    "type": "string",
                                    "description": "The city and state or country, e.g. 'San Francisco, CA'"
                                }
                            },
                            "required": ["location"]
                        }))
                        .build()?
                ])
                .tool_choice_auto()
                .max_tokens(500)
                .build()?;

            let final_response = client.send_chat_completion(&follow_up_request).await?;
            
            if let Some(final_choice) = final_response.choices.first() {
                // Check if model wants to call more tools
                if let Some(more_tool_calls) = final_choice.tool_calls() {
                    println!("🛠️  Model wants to call {} additional tool(s):", more_tool_calls.len());
                    
                    // Add assistant message with additional tool calls to conversation
                    let content = final_choice.content().unwrap_or("");
                    let assistant_message = Message::assistant_with_tool_calls(content, more_tool_calls.to_vec());
                    messages.push(assistant_message);
                    
                    // Handle additional tool calls
                    for (i, tool_call) in more_tool_calls.iter().enumerate() {
                        println!("  {}. Calling '{}' with arguments: {}", 
                            i + 1, 
                            tool_call.function.name, 
                            tool_call.function.arguments
                        );

                        let tool_result = match tool_call.function.name.as_str() {
                            "get_weather" => {
                                let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)?;
                                let location = args["location"].as_str().unwrap_or("Unknown");
                                let temp = "22°C";
                                format!("Current weather in {}: Sunny, {}, light breeze", location, temp)
                            },
                            _ => "Unknown tool".to_string(),
                        };

                        println!("     ✅ Result: {}", tool_result);
                        messages.push(Message::tool_response(&tool_call.id, &tool_result));
                    }
                    
                    // Final request without tools for summary
                    println!("\n📤 Sending final summary request...");
                    let summary_request = ChatCompletionRequest::builder()
                        .model("deepseek/deepseek-chat-v3.1:free")
                        .messages(messages)
                        .max_tokens(300)
                        .build()?;

                    let summary_response = client.send_chat_completion(&summary_request).await?;
                    if let Some(summary_choice) = summary_response.choices.first() {
                        if let Some(summary_content) = summary_choice.content() {
                            println!("🤖 Final summary:");
                            println!("{}", summary_content);
                        }
                    }
                } else if let Some(final_content) = final_choice.content() {
                    println!("🤖 Final response:");
                    println!("{}", final_content);
                }
            }
        } else {
            // No tool calls, just regular response
            if let Some(content) = choice.content() {
                println!("🤖 Response (no tools used):");
                println!("{}", content);
            }
        }
    }

    println!("\n✨ Tool calling example completed!");
    Ok(())
}