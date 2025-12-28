//! # Tool and Function Call Types
//!
//! This module contains types for defining and working with tools (function calls)
//! in OpenRouter API requests. Tools allow LLMs to call external functions and
//! use their results in generating responses.
//!
//! ## Tool Definition
//!
//! Tools are defined using the [`Tool`] struct which follows OpenRouter's API format:
//!
//! ```rust
//! use openrouter_rs::types::tool::Tool;
//! use serde_json::json;
//!
//! let tool = Tool::builder()
//!     .name("get_weather")
//!     .description("Get the current weather for a location")
//!     .parameters(json!({
//!         "type": "object",
//!         "properties": {
//!             "location": {
//!                 "type": "string",
//!                 "description": "The city and state, e.g. San Francisco, CA"
//!             }
//!         },
//!         "required": ["location"]
//!     }))
//!     .build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Tool Choice Control
//!
//! Control how the model uses tools with [`ToolChoice`]:
//!
//! ```rust
//! use openrouter_rs::types::tool::ToolChoice;
//!
//! // Model chooses whether to use tools
//! let auto_choice = ToolChoice::Auto;
//!
//! // Force model to use tools
//! let required_choice = ToolChoice::Required;
//!
//! // Force specific tool
//! let specific_choice = ToolChoice::force_tool("get_weather");
//! ```

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::OpenRouterError;

/// Tool definition for function calling
///
/// Represents a tool that can be called by the LLM. Tools follow OpenRouter's
/// standardized format and are automatically converted to the appropriate
/// format for different model providers.
///
/// # Examples
///
/// ```rust
/// use openrouter_rs::types::tool::Tool;
/// use serde_json::json;
///
/// let weather_tool = Tool::builder()
///     .name("get_weather")
///     .description("Get current weather for a location")
///     .parameters(json!({
///         "type": "object",
///         "properties": {
///             "location": {"type": "string", "description": "City and state"}
///         },
///         "required": ["location"]
///     }))
///     .build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct Tool {
    /// Type of tool (always "function" for now)
    #[serde(rename = "type")]
    #[builder(default = r#""function".to_string()"#)]
    pub tool_type: String,

    /// Function definition
    pub function: FunctionDefinition,
}

impl Tool {
    /// Create a new tool builder
    pub fn builder() -> ToolBuilder {
        ToolBuilder::default()
    }

    /// Create a simple tool with name, description, and parameters
    pub fn new(name: &str, description: &str, parameters: Value) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: name.to_string(),
                description: description.to_string(),
                parameters,
            },
        }
    }
}

/// Function definition within a tool
///
/// Defines the function that can be called, including its name,
/// description, and parameter schema.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct FunctionDefinition {
    /// Name of the function
    #[builder(setter(into))]
    pub name: String,

    /// Description of what the function does
    #[builder(setter(into))]
    pub description: String,

    /// JSON Schema defining the function parameters
    #[builder(setter(custom))]
    pub parameters: Value,
}

impl FunctionDefinition {
    /// Create a new function definition builder
    pub fn builder() -> FunctionDefinitionBuilder {
        FunctionDefinitionBuilder::default()
    }
}

impl ToolBuilder {
    /// Set the function name
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.function = Some(
            FunctionDefinition::builder()
                .name(name)
                .description("")
                .parameters(Value::Null)
                .build()
                .unwrap(),
        );
        self
    }

    /// Set the function description
    pub fn description(&mut self, description: &str) -> &mut Self {
        if let Some(ref mut func) = self.function {
            func.description = description.to_string();
        }
        self
    }

    /// Set the parameters as a JSON Value
    pub fn parameters(&mut self, parameters: Value) -> &mut Self {
        if let Some(ref mut func) = self.function {
            func.parameters = parameters;
        }
        self
    }

    /// Set parameters from a serializable struct
    pub fn parameters_from<T: Serialize>(&mut self, params: &T) -> Result<&mut Self, OpenRouterError> {
        let value = serde_json::to_value(params)
            .map_err(|e| OpenRouterError::Serialization(e))?;
        Ok(self.parameters(value))
    }

    /// Set parameters from a JSON string
    pub fn parameters_json(&mut self, json: &str) -> Result<&mut Self, OpenRouterError> {
        let value: Value = serde_json::from_str(json)
            .map_err(|e| OpenRouterError::Serialization(e))?;
        Ok(self.parameters(value))
    }
}

impl FunctionDefinitionBuilder {
    /// Set parameters from a JSON Value
    pub fn parameters(&mut self, parameters: Value) -> &mut Self {
        self.parameters = Some(parameters);
        self
    }

    /// Set parameters from a serializable struct
    pub fn parameters_from<T: Serialize>(&mut self, params: &T) -> Result<&mut Self, OpenRouterError> {
        let value = serde_json::to_value(params)
            .map_err(|e| OpenRouterError::Serialization(e))?;
        self.parameters = Some(value);
        Ok(self)
    }

    /// Set parameters from a JSON string
    pub fn parameters_json(&mut self, json: &str) -> Result<&mut Self, OpenRouterError> {
        let value: Value = serde_json::from_str(json)
            .map_err(|e| OpenRouterError::Serialization(e))?;
        self.parameters = Some(value);
        Ok(self)
    }
}

/// Control how the model chooses to use tools
///
/// Specifies whether the model should use tools, and if so, how it should
/// choose which tools to call.
///
/// # Examples
///
/// ```rust
/// use openrouter_rs::types::tool::ToolChoice;
///
/// // Let model decide
/// let auto = ToolChoice::Auto;
///
/// // Prevent tool use
/// let none = ToolChoice::None;
///
/// // Require tool use
/// let required = ToolChoice::Required;
///
/// // Force specific tool
/// let specific = ToolChoice::force_tool("get_weather");
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Simple string choices: "none", "auto", "required"
    String(String),
    /// Force a specific tool to be called
    Specific(SpecificToolChoice),
}

impl ToolChoice {
    /// Model will not call any tools
    pub fn none() -> Self {
        Self::String("none".to_string())
    }

    /// Model can choose whether to call tools
    pub fn auto() -> Self {
        Self::String("auto".to_string())
    }

    /// Model must call at least one tool
    pub fn required() -> Self {
        Self::String("required".to_string())
    }

    /// Force the model to call a specific tool
    pub fn force_tool(tool_name: &str) -> Self {
        Self::Specific(SpecificToolChoice {
            tool_type: "function".to_string(),
            function: SpecificToolFunction {
                name: tool_name.to_string(),
            },
        })
    }
}

/// Specific tool choice for forcing a particular tool
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpecificToolChoice {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: SpecificToolFunction,
}

/// Function specification for specific tool choice
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpecificToolFunction {
    pub name: String,
}

/// Helper function to create a tool with common parameter structure
///
/// Creates a tool with an object-type parameter schema and the specified properties.
///
/// # Examples
///
/// ```rust
/// use openrouter_rs::types::tool::create_tool;
/// use serde_json::json;
///
/// let tool = create_tool(
///     "calculator",
///     "Perform basic arithmetic operations",
///     json!({
///         "operation": {"type": "string", "enum": ["add", "subtract", "multiply", "divide"]},
///         "a": {"type": "number"},
///         "b": {"type": "number"}
///     }),
///     &["operation", "a", "b"]
/// );
/// ```
pub fn create_tool(
    name: &str,
    description: &str,
    properties: Value,
    required: &[&str],
) -> Tool {
    let parameters = serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    });

    Tool::new(name, description, parameters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_creation() {
        let tool = Tool::builder()
            .name("test_function")
            .description("A test function")
            .parameters(json!({"type": "object"}))
            .build()
            .unwrap();

        assert_eq!(tool.tool_type, "function");
        assert_eq!(tool.function.name, "test_function");
        assert_eq!(tool.function.description, "A test function");
    }

    #[test]
    fn test_tool_choice_variants() {
        let auto = ToolChoice::auto();
        let none = ToolChoice::none();
        let required = ToolChoice::required();
        let specific = ToolChoice::force_tool("my_function");

        // Test serialization
        assert_eq!(serde_json::to_string(&auto).unwrap(), r#""auto""#);
        assert_eq!(serde_json::to_string(&none).unwrap(), r#""none""#);
        assert_eq!(serde_json::to_string(&required).unwrap(), r#""required""#);
        
        if let ToolChoice::Specific(spec) = specific {
            assert_eq!(spec.function.name, "my_function");
        } else {
            panic!("Expected specific tool choice");
        }
    }

    #[test]
    fn test_create_tool_helper() {
        let tool = create_tool(
            "weather",
            "Get weather",
            json!({"location": {"type": "string"}}),
            &["location"]
        );

        assert_eq!(tool.function.name, "weather");
        assert_eq!(tool.function.description, "Get weather");
        
        let params = &tool.function.parameters;
        assert_eq!(params["type"], "object");
        assert_eq!(params["required"], json!(["location"]));
    }
}