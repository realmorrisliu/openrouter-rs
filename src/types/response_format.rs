use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Configuration for structured output responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatType {
    /// Standard text response (default)
    Text,
    /// JSON object response
    JsonObject,
    /// JSON response following a specific schema
    JsonSchema,
    /// Grammar-constrained response
    Grammar,
    /// Python-constrained response
    Python,
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
    /// Legacy shorthand format type specification (e.g. "text")
    TypeOnly(ResponseFormatType),
    /// Canonical type envelope (e.g. {"type":"text"})
    Typed {
        #[serde(rename = "type")]
        type_: ResponseFormatType,
    },
    /// Full configuration for JSON Schema responses
    JsonSchema {
        #[serde(rename = "type")]
        type_: ResponseFormatType,
        json_schema: JsonSchemaConfig,
    },
    /// Grammar-constrained text response format
    Grammar {
        #[serde(rename = "type")]
        type_: ResponseFormatType,
        grammar: String,
    },
}

impl ResponseFormat {
    /// Create a standard text response format.
    pub fn text() -> Self {
        ResponseFormat::Typed {
            type_: ResponseFormatType::Text,
        }
    }

    /// Create a JSON object response format.
    pub fn json_object() -> Self {
        ResponseFormat::Typed {
            type_: ResponseFormatType::JsonObject,
        }
    }

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

    /// Create a grammar-constrained response format.
    pub fn grammar(grammar: impl Into<String>) -> Self {
        ResponseFormat::Grammar {
            type_: ResponseFormatType::Grammar,
            grammar: grammar.into(),
        }
    }

    /// Create a python-constrained response format.
    pub fn python() -> Self {
        ResponseFormat::Typed {
            type_: ResponseFormatType::Python,
        }
    }
}

impl Default for ResponseFormat {
    fn default() -> Self {
        ResponseFormat::text()
    }
}
