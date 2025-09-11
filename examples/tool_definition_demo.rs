//! # Tool Definition Demo
//!
//! This example demonstrates how to define tools and build requests with tool support
//! without making actual API calls. It's useful for understanding the tool definition
//! syntax and verifying that serialization works correctly.
//!
//! ## Features Demonstrated
//!
//! - Tool definition with various parameter types
//! - Tool choice options
//! - Request building with tools
//! - JSON serialization of tool-enabled requests
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example tool_definition_demo
//! ```

use openrouter_rs::{
    api::chat::{ChatCompletionRequest, Message},
    types::{Role, Tool, ToolChoice, tool::create_tool},
};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 OpenRouter Tool Definition Demo");
    println!("==================================\n");

    // Demo 1: Basic tool with simple parameters
    println!("1. Basic Calculator Tool:");
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
                "a": {"type": "number", "description": "First number"},
                "b": {"type": "number", "description": "Second number"}
            },
            "required": ["operation", "a", "b"]
        }))
        .build()?;

    println!("   Name: {}", calculator_tool.function.name);
    println!("   Description: {}", calculator_tool.function.description);
    println!("   Parameters: {}\n", serde_json::to_string_pretty(&calculator_tool.function.parameters)?);

    // Demo 2: Tool with complex nested parameters
    println!("2. Weather Tool with Complex Parameters:");
    let weather_tool = Tool::builder()
        .name("get_weather")
        .description("Get detailed weather information for a location")
        .parameters(json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "object",
                    "properties": {
                        "city": {"type": "string", "description": "City name"},
                        "country": {"type": "string", "description": "Country code (e.g., US, UK)"}
                    },
                    "required": ["city"]
                },
                "units": {
                    "type": "string",
                    "enum": ["metric", "imperial", "kelvin"],
                    "default": "metric",
                    "description": "Temperature unit system"
                },
                "include_forecast": {
                    "type": "boolean",
                    "default": false,
                    "description": "Whether to include 5-day forecast"
                }
            },
            "required": ["location"]
        }))
        .build()?;

    println!("   Name: {}", weather_tool.function.name);
    println!("   Description: {}", weather_tool.function.description);
    println!("   Has nested parameters: ✓\n");

    // Demo 3: Tool using the helper function
    println!("3. File Operations Tool (using helper function):");
    let file_tool = create_tool(
        "file_operations",
        "Perform file system operations",
        json!({
            "action": {
                "type": "string",
                "enum": ["read", "write", "delete", "list"],
                "description": "The file operation to perform"
            },
            "path": {
                "type": "string",
                "description": "File or directory path"
            },
            "content": {
                "type": "string",
                "description": "Content to write (only for write action)"
            }
        }),
        &["action", "path"]
    );

    println!("   Name: {}", file_tool.function.name);
    println!("   Required params: [\"action\", \"path\"]\n");

    // Demo 4: Tool choice options
    println!("4. Tool Choice Options:");
    let auto_choice = ToolChoice::auto();
    let none_choice = ToolChoice::none();
    let required_choice = ToolChoice::required();
    let specific_choice = ToolChoice::force_tool("calculator");

    println!("   Auto: {}", serde_json::to_string(&auto_choice)?);
    println!("   None: {}", serde_json::to_string(&none_choice)?);
    println!("   Required: {}", serde_json::to_string(&required_choice)?);
    println!("   Specific: {}\n", serde_json::to_string_pretty(&specific_choice)?);

    // Demo 5: Chat request with tools
    println!("5. Chat Request with Tools:");
    let request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3.1:free")
        .messages(vec![
            Message::new(Role::System, "You are a helpful assistant with access to tools."),
            Message::new(Role::User, "Calculate 15 * 7 and get weather for London"),
        ])
        .tool(calculator_tool)
        .tool(weather_tool)
        .tool(file_tool)
        .tool_choice_auto()
        .parallel_tool_calls(true)
        .max_tokens(1000)
        .build()?;

    println!("   Model: DeepSeek Chat v3.1 (Free)");
    println!("   Tools defined: {}", request.tools().map_or(0, |t| t.len()));
    println!("   Tool choice: Auto");
    println!("   Parallel calls: Enabled");

    // Demo 6: Serialized request (what gets sent to API)
    println!("\n6. Serialized Request Sample:");
    let serialized = serde_json::to_string_pretty(&request)?;
    
    // Show just the tools section for brevity
    let parsed: serde_json::Value = serde_json::from_str(&serialized)?;
    if let Some(tools) = parsed.get("tools") {
        println!("   Tools section:");
        println!("{}", serde_json::to_string_pretty(tools)?);
    }

    println!("\n✨ Tool definition demo completed!");
    println!("   Ready to use with actual API calls!");

    Ok(())
}