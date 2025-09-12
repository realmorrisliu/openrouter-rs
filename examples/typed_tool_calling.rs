//! # Typed Tool Calling Example
//!
//! This example demonstrates how to use strongly-typed tools with the OpenRouter SDK.
//! Instead of manually crafting JSON Schema objects, you define Rust structs and enums
//! that automatically generate the appropriate schemas.
//!
//! ## Features Demonstrated:
//!
//! - Strongly-typed tool parameters using structs
//! - Enum-based parameters with validation
//! - Optional parameters with defaults
//! - Automatic JSON Schema generation
//! - Type-safe tool calling workflow
//!
//! ## Usage:
//!
//! ```bash
//! OPENROUTER_API_KEY=your_key cargo run --example typed_tool_calling
//! ```

use std::env;

use openrouter_rs::{
    api::chat::{ChatCompletionRequest, Message},
    client::OpenRouterClient,
    types::{
        typed_tool::{TypedTool, TypedToolParams},
        Role, ToolChoice,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Weather query parameters with location and unit preferences
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct WeatherParams {
    /// The city and state/country, e.g. "San Francisco, CA" or "London, UK"
    pub location: String,
    /// Temperature unit preference
    #[serde(default = "default_temperature_unit")]
    pub unit: TemperatureUnit,
    /// Include forecast information (optional)
    #[serde(default)]
    pub include_forecast: bool,
}

/// Temperature unit options
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureUnit {
    /// Celsius temperature scale
    Celsius,
    /// Fahrenheit temperature scale  
    Fahrenheit,
    /// Kelvin temperature scale
    Kelvin,
}

fn default_temperature_unit() -> TemperatureUnit {
    TemperatureUnit::Fahrenheit
}

impl TypedTool for WeatherParams {
    fn name() -> &'static str {
        "get_weather"
    }

    fn description() -> &'static str {
        "Get current weather information for a specific location with temperature unit preferences"
    }
}

/// Calculator parameters for basic arithmetic operations
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct CalculatorParams {
    /// The arithmetic operation to perform
    pub operation: ArithmeticOperation,
    /// First number for the calculation
    pub a: f64,
    /// Second number for the calculation
    pub b: f64,
    /// Number of decimal places for the result (optional)
    #[serde(default = "default_precision")]
    pub precision: u32,
}

/// Available arithmetic operations
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ArithmeticOperation {
    /// Addition: a + b
    Add,
    /// Subtraction: a - b
    Subtract,
    /// Multiplication: a * b
    Multiply,
    /// Division: a / b
    Divide,
    /// Exponentiation: a^b
    Power,
    /// Modulo: a % b
    Modulo,
}

fn default_precision() -> u32 {
    2
}

impl TypedTool for CalculatorParams {
    fn name() -> &'static str {
        "calculator"
    }

    fn description() -> &'static str {
        "Perform basic arithmetic operations with configurable precision"
    }
}

/// Text analysis parameters for various NLP operations
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct TextAnalysisParams {
    /// The text to analyze
    pub text: String,
    /// Types of analysis to perform
    pub analysis_types: Vec<AnalysisType>,
    /// Language of the text (optional, auto-detect if not provided)
    pub language: Option<String>,
    /// Return detailed results (optional)
    #[serde(default)]
    pub detailed: bool,
}

/// Text analysis operations
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    /// Sentiment analysis (positive/negative/neutral)
    Sentiment,
    /// Word count and basic statistics
    WordCount,
    /// Language detection
    LanguageDetection,
    /// Extract key phrases
    KeyPhrases,
    /// Readability analysis
    Readability,
}

impl TypedTool for TextAnalysisParams {
    fn name() -> &'static str {
        "analyze_text"
    }

    fn description() -> &'static str {
        "Perform comprehensive text analysis including sentiment, word count, and key phrase extraction"
    }
}

/// Search parameters for web or document search
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct SearchParams {
    /// Search query
    pub query: String,
    /// Search type/source
    pub search_type: SearchType,
    /// Maximum number of results to return
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    /// Filter options (optional)
    pub filters: Option<SearchFilters>,
}

/// Search type options
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    /// Web search
    Web,
    /// Academic papers
    Academic,
    /// News articles
    News,
    /// Images
    Images,
    /// Videos
    Videos,
}

/// Optional search filters
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct SearchFilters {
    /// Date range filter (days ago)
    pub date_range: Option<u32>,
    /// Domain restriction (e.g., "example.com")
    pub domain: Option<String>,
    /// Language filter
    pub language: Option<String>,
}

fn default_max_results() -> u32 {
    10
}

impl TypedTool for SearchParams {
    fn name() -> &'static str {
        "search"
    }

    fn description() -> &'static str {
        "Search various sources (web, academic, news) with configurable filters and result limits"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable is required");

    let client = OpenRouterClient::builder()
        .api_key(&api_key)
        .http_referer("https://localhost")
        .x_title("Typed Tool Calling Example")
        .build()?;

    println!("🔧 Typed Tool Calling Example");
    println!("==============================\n");

    // Demonstrate schema generation
    println!("📋 Generated JSON Schemas:");
    println!("---------------------------");
    
    println!("Weather Tool Schema:");
    println!("{}", serde_json::to_string_pretty(&WeatherParams::get_schema())?);
    println!();

    println!("Calculator Tool Schema:");
    println!("{}", serde_json::to_string_pretty(&CalculatorParams::get_schema())?);
    println!();

    // Create a chat completion request with multiple typed tools
    let request = ChatCompletionRequest::builder()
        .model("deepseek/deepseek-chat-v3.1:free")
        .messages(vec![
            Message::new(
                Role::System,
                "You are a helpful assistant with access to various tools. Use appropriate tools to help answer user questions."
            ),
            Message::new(
                Role::User,
                "I need to know the weather in New York City in Celsius, and I also want to calculate 15.5 * 3.7 with 3 decimal places of precision."
            ),
        ])
        .typed_tool::<WeatherParams>()        // Add weather tool
        .typed_tool::<CalculatorParams>()     // Add calculator tool
        .typed_tool::<TextAnalysisParams>()   // Add text analysis tool
        .typed_tool::<SearchParams>()         // Add search tool
        .tool_choice_auto()                   // Let model choose which tools to use
        .build()?;

    println!("🤖 Sending request with {} typed tools...", request.tools().map_or(0, |t| t.len()));
    
    match client.send_chat_completion(&request).await {
        Ok(response) => {
            println!("✅ Response received!\n");
            
            for (i, choice) in response.choices.iter().enumerate() {
                println!("Choice {}: {}", i + 1, "=".repeat(50));
                
                if let Some(content) = choice.content() {
                    println!("💬 Content: {}", content);
                }

                if let Some(role) = choice.role() {
                    println!("🎭 Role: {}", role);
                }

                if let Some(tool_calls) = choice.tool_calls() {
                    println!("🔧 Tool Calls ({}):", tool_calls.len());
                    for (j, tool_call) in tool_calls.iter().enumerate() {
                        println!("  {}. {} (ID: {})", j + 1, tool_call.function.name, tool_call.id);
                        println!("     Arguments: {}", tool_call.function.arguments);
                        
                        // Demonstrate typed parameter parsing
                        match tool_call.function.name.as_str() {
                            "get_weather" => {
                                match serde_json::from_str::<WeatherParams>(&tool_call.function.arguments) {
                                    Ok(params) => {
                                        println!("     ✅ Parsed weather params:");
                                        println!("        Location: {}", params.location);
                                        println!("        Unit: {:?}", params.unit);
                                        println!("        Include forecast: {}", params.include_forecast);
                                    }
                                    Err(e) => println!("     ❌ Failed to parse weather params: {}", e),
                                }
                            }
                            "calculator" => {
                                match serde_json::from_str::<CalculatorParams>(&tool_call.function.arguments) {
                                    Ok(params) => {
                                        println!("     ✅ Parsed calculator params:");
                                        println!("        Operation: {:?}", params.operation);
                                        println!("        A: {}, B: {}", params.a, params.b);
                                        println!("        Precision: {}", params.precision);
                                    }
                                    Err(e) => println!("     ❌ Failed to parse calculator params: {}", e),
                                }
                            }
                            _ => {
                                println!("     ℹ️  Tool not handled in this example");
                            }
                        }
                    }
                }

                if let Some(finish_reason) = choice.finish_reason() {
                    println!("🏁 Finish Reason: {:?}", finish_reason);
                }

                println!();
            }

            // Show usage information
            if let Some(usage) = response.usage {
                println!("📊 Token Usage:");
                println!("   Prompt tokens: {}", usage.prompt_tokens);
                println!("   Completion tokens: {}", usage.completion_tokens);
                println!("   Total tokens: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }

    // Additional demonstration: Show how to work with typed parameters
    println!("\n🧪 Type Safety Demo:");
    println!("---------------------");
    
    // Create typed parameters programmatically
    let weather_params = WeatherParams {
        location: "Tokyo, Japan".to_string(),
        unit: TemperatureUnit::Celsius,
        include_forecast: true,
    };

    let calculator_params = CalculatorParams {
        operation: ArithmeticOperation::Power,
        a: 2.0,
        b: 8.0,
        precision: 1,
    };

    println!("Weather params JSON: {}", weather_params.to_json_value()?);
    println!("Calculator params JSON: {}", calculator_params.to_json_value()?);

    // Validate parameters
    println!("Weather params validation: {:?}", weather_params.validate());
    println!("Calculator params validation: {:?}", calculator_params.validate());

    Ok(())
}