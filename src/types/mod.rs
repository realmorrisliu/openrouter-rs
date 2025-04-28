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
