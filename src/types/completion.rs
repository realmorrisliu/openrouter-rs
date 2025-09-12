use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReasoningDetail {
    /// The type of reasoning block (e.g., "reasoning.text")
    #[serde(rename = "type")]
    pub block_type: String,
    /// The actual reasoning content (Anthropic uses "text" field)
    #[serde(alias = "content")]
    pub text: String,
    /// Cryptographic signature (Anthropic specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Format identifier (Anthropic specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

impl ReasoningDetail {
    /// Get the content/text of this reasoning detail
    pub fn content(&self) -> &str {
        &self.text
    }

    /// Get the type of this reasoning block
    pub fn reasoning_type(&self) -> &str {
        &self.block_type
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseUsage {
    /// Including images and tools if any
    pub prompt_tokens: u32,
    /// The tokens generated
    pub completion_tokens: u32,
    /// Sum of the above two fields
    pub total_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String, // Always "function" according to TS type
    pub function: FunctionCall,
}

impl ToolCall {
    /// Parse tool arguments into typed parameters
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use openrouter_rs::types::typed_tool::TypedTool;
    /// use serde::{Deserialize, Serialize};
    /// use schemars::JsonSchema;
    /// 
    /// #[derive(Serialize, Deserialize, JsonSchema)]
    /// struct WeatherParams {
    ///     location: String,
    /// }
    /// 
    /// impl TypedTool for WeatherParams {
    ///     fn name() -> &'static str { "get_weather" }
    ///     fn description() -> &'static str { "Get weather" }
    /// }
    /// 
    /// // Parse tool call parameters
    /// let params: WeatherParams = tool_call.parse_params()?;
    /// ```
    pub fn parse_params<T>(&self) -> Result<T, crate::error::OpenRouterError> 
    where 
        T: crate::types::typed_tool::TypedTool,
    {
        serde_json::from_str(&self.function.arguments)
            .map_err(|e| crate::error::OpenRouterError::Serialization(e))
    }
    
    /// Check if this tool call matches a specific tool type
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// if tool_call.is_tool::<WeatherParams>() {
    ///     let params = tool_call.parse_params::<WeatherParams>()?;
    ///     // Handle weather tool
    /// }
    /// ```
    pub fn is_tool<T>(&self) -> bool 
    where 
        T: crate::types::typed_tool::TypedTool,
    {
        self.function.name == T::name()
    }
    
    /// Get the tool name
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// match tool_call.name() {
    ///     "get_weather" => { /* handle weather */ }
    ///     "calculator" => { /* handle calculator */ }
    ///     _ => { /* unknown tool */ }
    /// }
    /// ```
    pub fn name(&self) -> &str {
        &self.function.name
    }
    
    /// Get the raw JSON arguments as a string
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// println!("Raw arguments: {}", tool_call.arguments_json());
    /// ```
    pub fn arguments_json(&self) -> &str {
        &self.function.arguments
    }
    
    /// Get the tool call ID
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// println!("Tool call ID: {}", tool_call.id());
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get the tool type (usually "function")
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// assert_eq!(tool_call.tool_type(), "function");
    /// ```
    pub fn tool_type(&self) -> &str {
        &self.type_
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Choice {
    NonChat(NonChatChoice),
    NonStreaming(NonStreamingChoice),
    Streaming(StreamingChoice),
}

impl Choice {
    pub fn content(&self) -> Option<&str> {
        match self {
            Choice::NonChat(choice) => Some(choice.text.as_str()),
            Choice::NonStreaming(choice) => choice.message.content.as_deref(),
            Choice::Streaming(choice) => choice.delta.content.as_deref(),
        }
    }

    pub fn role(&self) -> Option<&str> {
        match self {
            Choice::NonChat(_) => None,
            Choice::NonStreaming(choice) => choice.message.role.as_deref(),
            Choice::Streaming(choice) => choice.delta.role.as_deref(),
        }
    }

    pub fn tool_calls(&self) -> Option<&[ToolCall]> {
        match self {
            Choice::NonChat(_) => None,
            Choice::NonStreaming(choice) => choice.message.tool_calls.as_deref(),
            Choice::Streaming(choice) => choice.delta.tool_calls.as_deref(),
        }
    }

    pub fn finish_reason(&self) -> Option<&FinishReason> {
        match self {
            Choice::NonChat(choice) => choice.finish_reason.as_ref(),
            Choice::NonStreaming(choice) => choice.finish_reason.as_ref(),
            Choice::Streaming(choice) => choice.finish_reason.as_ref(),
        }
    }

    pub fn native_finish_reason(&self) -> Option<&str> {
        match self {
            Choice::NonChat(_) => None,
            Choice::NonStreaming(choice) => choice.native_finish_reason.as_deref(),
            Choice::Streaming(choice) => choice.native_finish_reason.as_deref(),
        }
    }

    pub fn error(&self) -> Option<&ErrorResponse> {
        match self {
            Choice::NonChat(choice) => choice.error.as_ref(),
            Choice::NonStreaming(choice) => choice.error.as_ref(),
            Choice::Streaming(choice) => choice.error.as_ref(),
        }
    }

    pub fn reasoning(&self) -> Option<&str> {
        match self {
            Choice::NonChat(_) => None,
            Choice::NonStreaming(choice) => choice.message.reasoning.as_deref(),
            Choice::Streaming(choice) => choice.delta.reasoning.as_deref(),
        }
    }

    pub fn reasoning_details(&self) -> Option<&[ReasoningDetail]> {
        match self {
            Choice::NonChat(_) => None,
            Choice::NonStreaming(choice) => choice.message.reasoning_details.as_deref(),
            Choice::Streaming(choice) => choice.delta.reasoning_details.as_deref(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    ToolCalls,
    Stop,
    Length,
    ContentFilter,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NonChatChoice {
    pub finish_reason: Option<FinishReason>,
    pub text: String,
    pub error: Option<ErrorResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NonStreamingChoice {
    pub finish_reason: Option<FinishReason>,
    pub native_finish_reason: Option<String>,
    pub message: Message,
    pub error: Option<ErrorResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamingChoice {
    pub finish_reason: Option<FinishReason>,
    pub native_finish_reason: Option<String>,
    pub delta: Delta,
    pub error: Option<ErrorResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub content: Option<String>,
    pub role: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<Vec<ReasoningDetail>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<Vec<ReasoningDetail>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ObjectType {
    #[serde(rename = "chat.completion")]
    ChatCompletion,
    #[serde(rename = "chat.completion.chunk")]
    ChatCompletionChunk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompletionsResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: u64, // Unix timestamp
    pub model: String,
    #[serde(rename = "object")]
    pub object_type: ObjectType,
    pub provider: Option<String>,
    pub system_fingerprint: Option<String>,
    pub usage: Option<ResponseUsage>,
}
