use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Configuration for structured output responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatType {
    /// Standard text response (default)
    Text,
    /// JSON response following a specific schema
    JsonSchema,
}

/// JSON Schema configuration for structured outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaConfig {
    /// Name for the schema (used for reference)
    pub name: String,
    /// Whether to strictly enforce the schema
    #[serde(default)]
    pub strict: bool,
    /// The actual JSON Schema definition
    pub schema: Value,
}

/// Response format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum ResponseFormat {
    /// Simple format type specification
    TypeOnly(ResponseFormatType),
    /// Full configuration for JSON Schema responses
    JsonSchema {
        #[serde(rename = "type")]
        type_: ResponseFormatType,
        json_schema: JsonSchemaConfig,
    },
}

impl ResponseFormat {
    /// Create a new JSON Schema response format
    pub fn json_schema(name: impl Into<String>, strict: bool, schema: Value) -> Self {
        ResponseFormat::JsonSchema {
            type_: ResponseFormatType::JsonSchema,
            json_schema: JsonSchemaConfig {
                name: name.into(),
                strict,
                schema,
            },
        }
    }
}

impl Default for ResponseFormat {
    fn default() -> Self {
        ResponseFormat::TypeOnly(ResponseFormatType::Text)
    }
}
