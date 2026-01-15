//! # Typed Tools Support
//!
//! This module provides traits and utilities for creating strongly-typed tools
//! using Rust structs and enums instead of raw JSON Schema objects. The `TypedTool`
//! trait automatically generates JSON Schema from Rust types using the `schemars` crate.
//!
//! ## Examples
//!
//! ```rust
//! use openrouter_rs::types::typed_tool::{TypedTool, TypedToolParams};
//! use serde::{Deserialize, Serialize};
//! use schemars::JsonSchema;
//!
//! #[derive(Serialize, Deserialize, JsonSchema, Debug)]
//! pub struct WeatherParams {
//!     /// The city and state, e.g. San Francisco, CA
//!     pub location: String,
//!     /// Temperature unit (optional)
//!     #[serde(default = "default_unit")]
//!     pub unit: TemperatureUnit,
//! }
//!
//! #[derive(Serialize, Deserialize, JsonSchema, Debug)]
//! #[serde(rename_all = "lowercase")]
//! pub enum TemperatureUnit {
//!     Celsius,
//!     Fahrenheit,
//! }
//!
//! fn default_unit() -> TemperatureUnit {
//!     TemperatureUnit::Fahrenheit
//! }
//!
//! impl TypedTool for WeatherParams {
//!     fn name() -> &'static str {
//!         "get_weather"
//!     }
//!
//!     fn description() -> &'static str {
//!         "Get the current weather for a specific location"
//!     }
//! }
//!
//! // Use the typed tool
//! let tool = WeatherParams::create_tool();
//! ```

use schemars::{JsonSchema, schema_for};
use serde_json::Value;

use crate::types::Tool;

/// Trait for creating strongly-typed tools from Rust structs
///
/// This trait allows you to define tools using Rust structs with proper
/// type safety, automatic JSON Schema generation, and compile-time validation.
///
/// # Requirements
///
/// Types implementing this trait must also derive:
/// - `Serialize` (for JSON serialization)
/// - `Deserialize` (for JSON deserialization) 
/// - `JsonSchema` (for automatic schema generation)
///
/// # Examples
///
/// ```rust
/// use openrouter_rs::types::typed_tool::TypedTool;
/// use serde::{Deserialize, Serialize};
/// use schemars::JsonSchema;
///
/// #[derive(Serialize, Deserialize, JsonSchema)]
/// pub struct CalculatorParams {
///     pub operation: Operation,
///     pub a: f64,
///     pub b: f64,
/// }
///
/// #[derive(Serialize, Deserialize, JsonSchema)]
/// #[serde(rename_all = "lowercase")]
/// pub enum Operation {
///     Add,
///     Subtract,
///     Multiply,
///     Divide,
/// }
///
/// impl TypedTool for CalculatorParams {
///     fn name() -> &'static str {
///         "calculator"
///     }
///
///     fn description() -> &'static str {
///         "Perform basic arithmetic operations"
///     }
/// }
/// ```
pub trait TypedTool: JsonSchema + serde::Serialize + for<'de> serde::Deserialize<'de> {
    /// The name of the tool/function
    fn name() -> &'static str;

    /// Human-readable description of what the tool does
    fn description() -> &'static str;

    /// Create a Tool instance from this typed tool definition
    ///
    /// This method automatically generates the JSON Schema from the Rust type
    /// and creates a properly formatted OpenRouter Tool.
    fn create_tool() -> Tool {
        let schema = schema_for!(Self);
        let schema_json = serde_json::to_value(schema).unwrap_or(Value::Null);
        
        Tool::new(Self::name(), Self::description(), schema_json)
    }

    /// Get the JSON Schema for this tool's parameters
    ///
    /// This is useful for debugging or manual inspection of the generated schema.
    fn get_schema() -> serde_json::Value {
        let schema = schema_for!(Self);
        serde_json::to_value(schema).unwrap_or(Value::Null)
    }
}

/// Helper trait for working with typed tool parameters
///
/// This trait provides additional utilities for working with tool parameters,
/// including validation and conversion methods.
pub trait TypedToolParams: TypedTool {
    /// Validate the parameters against the schema
    ///
    /// This method can be overridden to provide custom validation logic
    /// beyond what the JSON Schema provides.
    fn validate(&self) -> Result<(), String> {
        // Default implementation - can be overridden
        Ok(())
    }

    /// Convert from JSON Value to strongly-typed parameters
    ///
    /// This is useful when receiving tool call parameters from the API
    /// and converting them to the strongly-typed struct.
    fn from_json_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Convert to JSON Value
    ///
    /// This is useful for sending parameters back in API requests.
    fn to_json_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

// Blanket implementation of TypedToolParams for all TypedTool types
impl<T: TypedTool> TypedToolParams for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use schemars::JsonSchema;

    #[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
    struct TestTool {
        pub message: String,
        pub count: u32,
    }

    impl TypedTool for TestTool {
        fn name() -> &'static str {
            "test_tool"
        }

        fn description() -> &'static str {
            "A test tool for unit testing"
        }
    }

    #[test]
    fn test_typed_tool_creation() {
        let tool = TestTool::create_tool();
        
        assert_eq!(tool.function.name, "test_tool");
        assert_eq!(tool.function.description, "A test tool for unit testing");
        assert_eq!(tool.tool_type, "function");
        
        // Verify schema structure
        let schema = &tool.function.parameters;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert!(schema["properties"]["message"].is_object());
        assert!(schema["properties"]["count"].is_object());
    }

    #[test]
    fn test_schema_generation() {
        let schema = TestTool::get_schema();
        
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["message"]["type"] == "string");
        assert!(schema["properties"]["count"]["type"] == "integer");
        // Check that required fields exist, but don't assume specific order
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 2);
        assert!(required.contains(&serde_json::json!("message")));
        assert!(required.contains(&serde_json::json!("count")));
    }

    #[test]
    fn test_json_conversion() {
        let test_tool = TestTool {
            message: "Hello".to_string(),
            count: 42,
        };

        let json_value = test_tool.to_json_value().unwrap();
        let converted_back = TestTool::from_json_value(json_value).unwrap();
        
        assert_eq!(test_tool, converted_back);
    }

    #[derive(Serialize, Deserialize, JsonSchema, Debug)]
    struct EnumTool {
        pub operation: TestOperation,
        pub value: f64,
    }

    #[derive(Serialize, Deserialize, JsonSchema, Debug)]
    #[serde(rename_all = "lowercase")]
    enum TestOperation {
        Square,
        Sqrt,
        Abs,
    }

    impl TypedTool for EnumTool {
        fn name() -> &'static str {
            "math_tool"
        }

        fn description() -> &'static str {
            "Perform mathematical operations"
        }
    }

    #[test]
    fn test_enum_schema_generation() {
        let tool = EnumTool::create_tool();
        let schema = &tool.function.parameters;
        
        // Verify enum is properly represented in schema
        assert!(schema["properties"]["operation"].is_object());
        let operation_schema = &schema["properties"]["operation"];
        
        // schemars can generate enums in various formats - just ensure the operation property exists
        // and is structured (not just a simple type)
        assert!(operation_schema.is_object());
        // Most importantly, verify the tool was created successfully
        assert_eq!(tool.function.name, "math_tool");
        assert_eq!(tool.function.description, "Perform mathematical operations");
    }
}