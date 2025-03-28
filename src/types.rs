use serde::{Deserialize, Serialize};

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

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::System => "system".to_string(),
            Role::Developer => "developer".to_string(),
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
            Role::Tool => "tool".to_string(),
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

impl ToString for Effort {
    fn to_string(&self) -> String {
        match self {
            Effort::High => "high".to_string(),
            Effort::Medium => "medium".to_string(),
            Effort::Low => "low".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderPreferences {
    sort: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReasoningConfig {
    effort: Option<Effort>,
    max_tokens: Option<u32>,
    exclude: Option<bool>,
}
