pub mod api_key;
mod loader;
mod model;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use loader::load_config;
pub use model::ModelConfig;

use crate::error::OpenRouterError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenRouterConfig {
    pub base_url: String,
    #[serde(default)]
    pub models: ModelConfig,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            base_url: "https://openrouter.ai/api/v1".to_string(),
            models: ModelConfig::default(),
        }
    }
}

pub fn get_config_path() -> Result<PathBuf, OpenRouterError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| OpenRouterError::ConfigError("Could not find config directory".into()))?
        .join("openrouter");

    Ok(config_dir.join("config.toml"))
}
