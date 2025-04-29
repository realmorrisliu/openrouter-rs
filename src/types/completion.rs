use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
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
