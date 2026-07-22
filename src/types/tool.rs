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
//! let auto_choice = ToolChoice::auto();
//!
//! // Force model to use tools
//! let required_choice = ToolChoice::required();
//!
//! // Force specific tool
//! let specific_choice = ToolChoice::force_tool("get_weather");
//! ```

use std::collections::HashMap;

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
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Tool {
    /// Type of tool (always "function" for now)
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Function definition
    pub function: FunctionDefinition,

    /// Optional cache-control directive for provider-side prompt caching.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<Value>,
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
                strict: None,
            },
            cache_control: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ToolBuilder {
    tool_type: Option<String>,
    name: Option<String>,
    description: Option<String>,
    parameters: Option<Value>,
    strict: Option<bool>,
    cache_control: Option<Value>,
}

impl ToolBuilder {
    /// Override the tool type. Defaults to `"function"`.
    pub fn tool_type(&mut self, tool_type: impl Into<String>) -> &mut Self {
        self.tool_type = Some(tool_type.into());
        self
    }

    /// Set the full function definition at once.
    pub fn function(&mut self, function: FunctionDefinition) -> &mut Self {
        self.name = Some(function.name);
        self.description = Some(function.description);
        self.parameters = Some(function.parameters);
        self.strict = function.strict;
        self
    }

    /// Build the tool, validating that the function name is present.
    pub fn build(&self) -> Result<Tool, OpenRouterError> {
        let name = self
            .name
            .clone()
            .ok_or_else(|| OpenRouterError::ConfigError("Tool name is required".to_string()))?;

        Ok(Tool {
            tool_type: self
                .tool_type
                .clone()
                .unwrap_or_else(|| "function".to_string()),
            function: FunctionDefinition {
                name,
                description: self.description.clone().unwrap_or_default(),
                parameters: self.parameters.clone().unwrap_or(Value::Null),
                strict: self.strict,
            },
            cache_control: self.cache_control.clone(),
        })
    }
}

/// Function definition within a tool
///
/// Defines the function that can be called, including its name,
/// description, and parameter schema.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
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

    /// Whether the model must strictly adhere to the parameter schema.
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
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
        self.name = Some(name.to_string());
        self
    }

    /// Set the function description
    pub fn description(&mut self, description: &str) -> &mut Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set the parameters as a JSON Value
    pub fn parameters(&mut self, parameters: Value) -> &mut Self {
        self.parameters = Some(parameters);
        self
    }

    /// Set parameters from a serializable struct
    pub fn parameters_from<T: Serialize>(
        &mut self,
        params: &T,
    ) -> Result<&mut Self, OpenRouterError> {
        let value = serde_json::to_value(params).map_err(OpenRouterError::Serialization)?;
        Ok(self.parameters(value))
    }

    /// Set parameters from a JSON string
    pub fn parameters_json(&mut self, json: &str) -> Result<&mut Self, OpenRouterError> {
        let value: Value = serde_json::from_str(json).map_err(OpenRouterError::Serialization)?;
        Ok(self.parameters(value))
    }

    /// Set the function strict-schema flag.
    pub fn strict(&mut self, strict: bool) -> &mut Self {
        self.strict = Some(strict);
        self
    }

    /// Set the top-level tool cache-control payload.
    pub fn cache_control(&mut self, cache_control: impl Into<Value>) -> &mut Self {
        self.cache_control = Some(cache_control.into());
        self
    }
}

impl FunctionDefinitionBuilder {
    /// Set parameters from a JSON Value
    pub fn parameters(&mut self, parameters: Value) -> &mut Self {
        self.parameters = Some(parameters);
        self
    }

    /// Set parameters from a serializable struct
    pub fn parameters_from<T: Serialize>(
        &mut self,
        params: &T,
    ) -> Result<&mut Self, OpenRouterError> {
        let value = serde_json::to_value(params).map_err(OpenRouterError::Serialization)?;
        self.parameters = Some(value);
        Ok(self)
    }

    /// Set parameters from a JSON string
    pub fn parameters_json(&mut self, json: &str) -> Result<&mut Self, OpenRouterError> {
        let value: Value = serde_json::from_str(json).map_err(OpenRouterError::Serialization)?;
        self.parameters = Some(value);
        Ok(self)
    }
}

/// OpenRouter built-in server tool definition.
///
/// Server tools are OpenRouter-hosted capabilities such as web search,
/// datetime lookup, files, bash, and model search. They share a common wire
/// shape: a `type`, optional `parameters`, and optional tool-specific
/// top-level fields.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ServerTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ServerTool {
    pub fn new(tool_type: impl Into<String>) -> Self {
        Self {
            tool_type: tool_type.into(),
            parameters: None,
            extra: HashMap::new(),
        }
    }

    pub fn with_parameters(tool_type: impl Into<String>, parameters: impl Into<Value>) -> Self {
        Self::new(tool_type).parameters(parameters)
    }

    pub fn parameters(mut self, parameters: impl Into<Value>) -> Self {
        self.parameters = Some(parameters.into());
        self
    }

    pub fn parameters_from<T: Serialize>(mut self, params: &T) -> Result<Self, OpenRouterError> {
        self.parameters =
            Some(serde_json::to_value(params).map_err(OpenRouterError::Serialization)?);
        Ok(self)
    }

    pub fn option(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }

    pub fn web_search() -> Self {
        Self::new("openrouter:web_search")
    }

    pub fn web_search_with_parameters(parameters: impl Into<Value>) -> Self {
        Self::with_parameters("openrouter:web_search", parameters)
    }

    pub fn web_search_preview() -> Self {
        Self::new("web_search_preview")
    }

    pub fn datetime() -> Self {
        Self::new("openrouter:datetime")
    }

    pub fn datetime_with_timezone(timezone: impl Into<String>) -> Self {
        Self::with_parameters(
            "openrouter:datetime",
            serde_json::json!({ "timezone": timezone.into() }),
        )
    }

    pub fn files() -> Self {
        Self::new("openrouter:files")
    }

    pub fn bash() -> Self {
        Self::new("openrouter:bash")
    }

    pub fn web_fetch() -> Self {
        Self::new("openrouter:web_fetch")
    }

    pub fn advisor() -> Self {
        Self::new("openrouter:advisor")
    }

    pub fn subagent() -> Self {
        Self::new("openrouter:subagent")
    }

    pub fn image_generation() -> Self {
        Self::new("openrouter:image_generation")
    }

    pub fn search_models() -> Self {
        Self::new("openrouter:experimental__search_models")
    }

    pub fn apply_patch() -> Self {
        Self::new("openrouter:apply_patch")
    }

    pub(crate) fn is_server_tool_type(tool_type: &str) -> bool {
        tool_type.starts_with("openrouter:")
            || matches!(
                tool_type,
                "web_search"
                    | "web_search_2025_08_26"
                    | "web_search_preview"
                    | "web_search_preview_2025_03_11"
                    | "apply_patch"
                    | "shell"
                    | "namespace"
            )
    }

    pub(crate) fn is_files_tool_type(tool_type: &str) -> bool {
        matches!(tool_type, "openrouter:files" | "files")
    }

    pub(crate) fn is_files_tool(&self) -> bool {
        Self::is_files_tool_type(&self.tool_type)
    }

    pub(crate) fn is_server_tool_value(value: &Value) -> bool {
        value
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(Self::is_server_tool_type)
    }

    pub(crate) fn is_files_tool_value(value: &Value) -> bool {
        value
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(Self::is_files_tool_type)
    }
}

impl From<ServerTool> for Value {
    fn from(tool: ServerTool) -> Self {
        serde_json::to_value(tool).expect("server tool serialization should not fail")
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
/// let auto = ToolChoice::auto();
///
/// // Prevent tool use
/// let none = ToolChoice::none();
///
/// // Require tool use
/// let required = ToolChoice::required();
///
/// // Force specific tool
/// let specific = ToolChoice::force_tool("get_weather");
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
#[serde(untagged)]
pub enum ToolChoice {
    /// Simple string choices: "none", "auto", "required"
    String(String),
    /// Force a specific tool to be called
    Specific(SpecificToolChoice),
    /// Force a specific OpenRouter server tool to be called
    Server(ServerToolChoice),
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

    /// Force the model to call a specific OpenRouter server tool.
    pub fn force_server_tool(tool_type: impl Into<String>) -> Self {
        Self::Server(ServerToolChoice {
            tool_type: tool_type.into(),
        })
    }
}

/// Specific tool choice for forcing a particular tool
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SpecificToolChoice {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: SpecificToolFunction,
}

/// Function specification for specific tool choice
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SpecificToolFunction {
    pub name: String,
}

/// Specific server-tool choice for forcing an OpenRouter built-in tool.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ServerToolChoice {
    #[serde(rename = "type")]
    pub tool_type: String,
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
pub fn create_tool(name: &str, description: &str, properties: Value, required: &[&str]) -> Tool {
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
            &["location"],
        );

        assert_eq!(tool.function.name, "weather");
        assert_eq!(tool.function.description, "Get weather");

        let params = &tool.function.parameters;
        assert_eq!(params["type"], "object");
        assert_eq!(params["required"], json!(["location"]));
    }
}
