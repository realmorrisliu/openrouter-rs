//! # Typed Tool Schema Generation Demo
//!
//! This example demonstrates the automatic JSON Schema generation from Rust types
//! without making any API calls. Perfect for testing and development.
//!
//! ## Usage:
//!
//! ```bash
//! cargo run --example typed_tool_schema_demo
//! ```

use openrouter_rs::{
    api::chat::{ChatCompletionRequest, Message},
    types::{typed_tool::{TypedTool, TypedToolParams}, Role},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Simple calculator parameters
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct CalculatorParams {
    /// First number
    pub a: f64,
    /// Second number  
    pub b: f64,
    /// Operation to perform
    pub operation: MathOperation,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MathOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl TypedTool for CalculatorParams {
    fn name() -> &'static str {
        "calculator"
    }

    fn description() -> &'static str {
        "Perform basic math operations"
    }
}

/// Weather query parameters
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct WeatherParams {
    /// Location to get weather for
    pub location: String,
    /// Include forecast (optional)
    #[serde(default)]
    pub include_forecast: bool,
}

impl TypedTool for WeatherParams {
    fn name() -> &'static str {
        "get_weather"
    }

    fn description() -> &'static str {
        "Get weather information"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Typed Tool Schema Generation Demo");
    println!("====================================\n");

    // 1. Create individual typed tools
    let calculator_tool = CalculatorParams::create_tool();
    let weather_tool = WeatherParams::create_tool();

    println!("📋 Generated Tools:");
    println!("------------------");
    
    println!("Calculator Tool:");
    println!("  Name: {}", calculator_tool.function.name);
    println!("  Description: {}", calculator_tool.function.description);
    println!("  Schema: {}\n", serde_json::to_string_pretty(&calculator_tool.function.parameters)?);

    println!("Weather Tool:");
    println!("  Name: {}", weather_tool.function.name);
    println!("  Description: {}", weather_tool.function.description);
    println!("  Schema: {}\n", serde_json::to_string_pretty(&weather_tool.function.parameters)?);

    // 2. Create a chat request using typed tools
    let request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3.1:free")
        .messages(vec![
            Message::new(Role::User, "Calculate 15 * 3 and get weather for Tokyo")
        ])
        .typed_tool::<CalculatorParams>()
        .typed_tool::<WeatherParams>()
        .tool_choice_auto()
        .build()?;

    println!("🤖 Chat Request with Typed Tools:");
    println!("--------------------------------");
    println!("Tools count: {}", request.tools().map_or(0, |t| t.len()));
    println!("Tool choice: {:?}", request.tool_choice());

    // 3. Demonstrate type-safe parameter creation
    println!("\n🧪 Type-Safe Parameters:");
    println!("-----------------------");
    
    let calc_params = CalculatorParams {
        a: 15.0,
        b: 3.0,
        operation: MathOperation::Multiply,
    };

    let weather_params = WeatherParams {
        location: "Tokyo, Japan".to_string(),
        include_forecast: true,
    };

    println!("Calculator params JSON: {}", calc_params.to_json_value()?);
    println!("Weather params JSON: {}", weather_params.to_json_value()?);

    // 4. Demonstrate validation
    println!("\n✅ Parameter Validation:");
    println!("-----------------------");
    println!("Calculator params valid: {:?}", calc_params.validate());
    println!("Weather params valid: {:?}", weather_params.validate());

    // 5. Show request serialization
    println!("\n📤 Request Tools Section:");
    println!("------------------------");
    if let Some(tools) = request.tools() {
        println!("{}", serde_json::to_string_pretty(tools)?);
    }

    println!("\n✨ Demo completed! Typed tools are working correctly.");
    Ok(())
}