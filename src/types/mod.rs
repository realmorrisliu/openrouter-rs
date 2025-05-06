pub mod completion;
pub mod provider;
pub mod response_format;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub use {completion::*, provider::*, response_format::*};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Developer,
    User,
    Assistant,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    High,
    Medium,
    Low,
}

impl Display for Effort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effort::High => write!(f, "high"),
            Effort::Medium => write!(f, "medium"),
            Effort::Low => write!(f, "low"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReasoningConfig {
    pub effort: Option<Effort>,
    pub max_tokens: Option<u32>,
    pub exclude: Option<bool>,
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
