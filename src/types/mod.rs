//! # Core Types and Data Structures
//!
//! This module contains all the core types, enums, and data structures used
//! throughout the OpenRouter SDK. These types provide type-safe representations
//! of API requests, responses, and configuration options.
//!
//! ## 📋 Type Categories
//!
//! ### Request/Response Types ([`completion`])
//! - **Chat Completions**: Modern conversational AI request/response structures
//! - **Text Completions**: Legacy prompt-based completion types
//! - **Streaming**: Types for handling real-time response streams
//! - **Reasoning**: Advanced reasoning and chain-of-thought structures
//!
//! ### Provider Information ([`provider`])
//! - **Model Metadata**: Provider-specific model information
//! - **Capabilities**: Model feature and parameter support
//! - **Pricing**: Cost information and token usage
//!
//! ### Response Formatting ([`response_format`])
//! - **JSON Schema**: Structured output formatting
//! - **Content Types**: Different response content formats
//! - **Validation**: Response format validation rules
//!
//! ### Tool Support ([`tool`])
//! - **Tool Definitions**: Function calling definitions and schemas
//! - **Tool Choice**: Control over tool usage behavior
//! - **Function Parameters**: JSON Schema for tool parameters
//!
//! ### Typed Tools ([`typed_tool`])
//! - **TypedTool Trait**: Strongly-typed tool definitions using Rust structs
//! - **Automatic Schema Generation**: JSON Schema generation from Rust types
//! - **Type Safety**: Compile-time validation of tool parameters
//!
//! ## 🎯 Core Enums
//!
//! ### Role
//! Defines the role of a message in a conversation:
//!
//! ```rust
//! use openrouter_rs::types::Role;
//!
//! let system_role = Role::System;    // System instructions
//! let user_role = Role::User;        // User input
//! let assistant_role = Role::Assistant; // AI response
//! let tool_role = Role::Tool;        // Tool/function results
//! let developer_role = Role::Developer; // Developer context
//! ```
//!
//! ### Effort
//! Specifies reasoning effort levels for chain-of-thought models:
//!
//! ```rust
//! use openrouter_rs::types::Effort;
//!
//! let xhigh_effort = Effort::Xhigh;   // Extra high reasoning depth
//! let high_effort = Effort::High;     // High reasoning depth
//! let medium_effort = Effort::Medium; // Balanced reasoning
//! let low_effort = Effort::Low;       // Quick reasoning
//! let minimal_effort = Effort::Minimal; // Minimal reasoning
//! let no_effort = Effort::None;       // Disable reasoning
//! ```
//!
//! ## 🔧 Configuration Types
//!
//! ### ReasoningConfig
//! Configuration for advanced reasoning capabilities:
//!
//! ```rust
//! use openrouter_rs::types::{ReasoningConfig, Effort};
//!
//! let reasoning = ReasoningConfig::builder()
//!     .enabled(true)
//!     .effort(Effort::High)
//!     .max_tokens(1000)
//!     .build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### ProviderPreferences
//! Specify preferences for model provider selection:
//!
//! ```rust
//! use openrouter_rs::types::ProviderPreferences;
//!
//! let prefs = ProviderPreferences {
//!     allow_fallbacks: true,
//!     require_parameters: Some(vec!["tools".to_string()]),
//!     data_collection: Some("deny".to_string()),
//! };
//! ```
//!
//! ## 📊 Model Categories
//!
//! Categories for filtering and organizing models:
//!
//! ```rust
//! use openrouter_rs::types::ModelCategory;
//!
//! // Filter models by use case
//! let programming_models = ModelCategory::Programming;
//! let reasoning_models = ModelCategory::Reasoning;
//! let image_models = ModelCategory::Image;
//! ```
//!
//! ## 🏗️ Builder Patterns
//!
//! Most complex types support the builder pattern for ergonomic construction:
//!
//! ```rust
//! use openrouter_rs::types::{ReasoningConfig, Effort};
//!
//! let config = ReasoningConfig::builder()
//!     .enabled(true)
//!     .effort(Effort::High)
//!     .max_tokens(2000)
//!     .exclude(false)
//!     .build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## 🔄 Serialization Support
//!
//! All types implement `Serialize` and `Deserialize` for JSON compatibility:
//!
//! ```rust
//! use openrouter_rs::types::Role;
//! use serde_json;
//!
//! let role = Role::Assistant;
//! let json = serde_json::to_string(&role)?;
//! let parsed: Role = serde_json::from_str(&json)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## 🎨 Display Formatting
//!
//! Common enums implement `Display` for human-readable output:
//!
//! ```rust
//! use openrouter_rs::types::{Role, Effort};
//!
//! println!("Role: {}", Role::User);        // "user"
//! println!("Effort: {}", Effort::High);    // "high"
//! ```

pub mod completion;
pub mod pagination;
pub mod provider;
pub mod response_format;
pub mod stream;
pub mod tool;
pub mod typed_tool;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub use {
    completion::*, pagination::*, provider::*, response_format::*, stream::*, tool::*,
    typed_tool::*,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub data: T,
}

/// Message role in a conversation
///
/// Specifies who or what is sending a message in a chat completion.
/// Different roles have different behaviors and restrictions.
///
/// # Examples
///
/// ```rust
/// use openrouter_rs::types::Role;
/// use openrouter_rs::api::chat::Message;
///
/// let system_msg = Message::new(Role::System, "You are a helpful assistant");
/// let user_msg = Message::new(Role::User, "Hello, world!");
/// let assistant_msg = Message::new(Role::Assistant, "Hello! How can I help?");
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System instructions that guide the AI's behavior
    System,
    /// Developer/admin context (provider-specific)
    Developer,
    /// User input or questions
    User,
    /// AI assistant responses
    Assistant,
    /// Results from tool/function calls
    Tool,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::Developer => write!(f, "developer"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::Tool => write!(f, "tool"),
        }
    }
}

/// Reasoning effort level for chain-of-thought models
///
/// Controls how much computational effort the model should put into
/// reasoning through problems. Higher effort levels typically produce
/// more detailed reasoning but take longer and cost more.
///
/// # Examples
///
/// ```rust
/// use openrouter_rs::types::Effort;
/// use openrouter_rs::api::chat::ChatCompletionRequest;
///
/// let request = ChatCompletionRequest::builder()
///     .model("deepseek/deepseek-r1")
///     .reasoning_effort(Effort::High)  // Maximum reasoning depth
///     // ... other fields
///     .build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    /// Extra high reasoning depth and thoroughness
    Xhigh,
    /// Maximum reasoning depth and thoroughness
    High,
    /// Balanced reasoning effort
    Medium,
    /// Quick, lightweight reasoning
    Low,
    /// Minimal reasoning effort
    Minimal,
    /// Disable reasoning effort
    None,
}

impl Display for Effort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effort::Xhigh => write!(f, "xhigh"),
            Effort::High => write!(f, "high"),
            Effort::Medium => write!(f, "medium"),
            Effort::Low => write!(f, "low"),
            Effort::Minimal => write!(f, "minimal"),
            Effort::None => write!(f, "none"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<Effort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

impl ReasoningConfig {
    /// Create a new ReasoningConfig with default enabled settings (medium effort)
    pub fn enabled() -> Self {
        Self {
            effort: None,
            max_tokens: None,
            exclude: None,
            enabled: Some(true),
        }
    }

    /// Create a ReasoningConfig with specific effort level
    pub fn with_effort(effort: Effort) -> Self {
        Self {
            effort: Some(effort),
            max_tokens: None,
            exclude: None,
            enabled: None,
        }
    }

    /// Create a ReasoningConfig with max tokens limit
    pub fn with_max_tokens(max_tokens: u32) -> Self {
        Self {
            effort: None,
            max_tokens: Some(max_tokens),
            exclude: None,
            enabled: None,
        }
    }

    /// Create a ReasoningConfig that excludes reasoning from response
    pub fn excluded() -> Self {
        Self {
            effort: None,
            max_tokens: None,
            exclude: Some(true),
            enabled: None,
        }
    }

    /// Set effort level
    pub fn effort(mut self, effort: Effort) -> Self {
        self.effort = Some(effort);
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set exclude flag
    pub fn exclude(mut self, exclude: bool) -> Self {
        self.exclude = Some(exclude);
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ModelCategory {
    Roleplay,
    Programming,
    Marketing,
    #[serde(rename = "marketing/seo")]
    MarketingSeo,
    Technology,
    Science,
    Translation,
    Legal,
    Finance,
    Health,
    Trivia,
    Academia,
}

impl Display for ModelCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelCategory::Roleplay => write!(f, "roleplay"),
            ModelCategory::Programming => write!(f, "programming"),
            ModelCategory::Marketing => write!(f, "marketing"),
            ModelCategory::MarketingSeo => write!(f, "marketing/seo"),
            ModelCategory::Technology => write!(f, "technology"),
            ModelCategory::Science => write!(f, "science"),
            ModelCategory::Translation => write!(f, "translation"),
            ModelCategory::Legal => write!(f, "legal"),
            ModelCategory::Finance => write!(f, "finance"),
            ModelCategory::Health => write!(f, "health"),
            ModelCategory::Trivia => write!(f, "trivia"),
            ModelCategory::Academia => write!(f, "academia"),
        }
    }
}

impl ModelCategory {
    pub fn all() -> Vec<ModelCategory> {
        vec![
            ModelCategory::Roleplay,
            ModelCategory::Programming,
            ModelCategory::Marketing,
            ModelCategory::MarketingSeo,
            ModelCategory::Technology,
            ModelCategory::Science,
            ModelCategory::Translation,
            ModelCategory::Legal,
            ModelCategory::Finance,
            ModelCategory::Health,
            ModelCategory::Trivia,
            ModelCategory::Academia,
        ]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SupportedParameters {
    Tools,
    Temperature,
    TopP,
    TopK,
    MinP,
    TopA,
    FrequencyPenalty,
    PresencePenalty,
    RepetitionPenalty,
    MaxTokens,
    LogitBias,
    Logprobs,
    TopLogprobs,
    Seed,
    ResponseFormat,
    StructuredOutputs,
    Stop,
    IncludeReasoning,
    Reasoning,
    WebSearchOptions,
}

impl Display for SupportedParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedParameters::Tools => write!(f, "tools"),
            SupportedParameters::Temperature => write!(f, "temperature"),
            SupportedParameters::TopP => write!(f, "top_p"),
            SupportedParameters::TopK => write!(f, "top_k"),
            SupportedParameters::MinP => write!(f, "min_p"),
            SupportedParameters::TopA => write!(f, "top_a"),
            SupportedParameters::FrequencyPenalty => write!(f, "frequency_penalty"),
            SupportedParameters::PresencePenalty => write!(f, "presence_penalty"),
            SupportedParameters::RepetitionPenalty => write!(f, "repetition_penalty"),
            SupportedParameters::MaxTokens => write!(f, "max_tokens"),
            SupportedParameters::LogitBias => write!(f, "logit_bias"),
            SupportedParameters::Logprobs => write!(f, "logprobs"),
            SupportedParameters::TopLogprobs => write!(f, "top_logprobs"),
            SupportedParameters::Seed => write!(f, "seed"),
            SupportedParameters::ResponseFormat => write!(f, "response_format"),
            SupportedParameters::StructuredOutputs => write!(f, "structured_outputs"),
            SupportedParameters::Stop => write!(f, "stop"),
            SupportedParameters::IncludeReasoning => write!(f, "include_reasoning"),
            SupportedParameters::Reasoning => write!(f, "reasoning"),
            SupportedParameters::WebSearchOptions => write!(f, "web_search_options"),
        }
    }
}

impl SupportedParameters {
    pub fn all() -> Vec<SupportedParameters> {
        vec![
            SupportedParameters::Tools,
            SupportedParameters::Temperature,
            SupportedParameters::TopP,
            SupportedParameters::TopK,
            SupportedParameters::MinP,
            SupportedParameters::TopA,
            SupportedParameters::FrequencyPenalty,
            SupportedParameters::PresencePenalty,
            SupportedParameters::RepetitionPenalty,
            SupportedParameters::MaxTokens,
            SupportedParameters::LogitBias,
            SupportedParameters::Logprobs,
            SupportedParameters::TopLogprobs,
            SupportedParameters::Seed,
            SupportedParameters::ResponseFormat,
            SupportedParameters::StructuredOutputs,
            SupportedParameters::Stop,
            SupportedParameters::IncludeReasoning,
            SupportedParameters::Reasoning,
            SupportedParameters::WebSearchOptions,
        ]
    }
}
