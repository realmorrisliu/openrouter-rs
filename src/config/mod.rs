mod model;

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

pub use model::ModelConfig;

use crate::error::OpenRouterError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenRouterConfig {
    #[serde(default)]
    pub models: ModelConfig,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        let default_config = include_str!("default_config.toml");
        let mut config = toml::from_str(default_config).unwrap_or(Self {
            models: ModelConfig::default(),
        });

        config.models.resolve();

        config
    }
}

pub fn load_config(config_path: impl AsRef<Path>) -> Result<OpenRouterConfig, OpenRouterError> {
    let config_path = config_path.as_ref();

    if !config_path.exists() {
        return Ok(OpenRouterConfig::default());
    }

    let config_content = fs::read_to_string(&config_path).map_err(|e| {
        OpenRouterError::ConfigError(format!(
            "Failed to read config file at {}: {}",
            config_path.display(),
            e
        ))
    })?;

    let mut config: OpenRouterConfig = toml::from_str(&config_content).map_err(|e| {
        OpenRouterError::ConfigError(format!(
            "Invalid config format in {}: {}",
            config_path.display(),
            e
        ))
    })?;

    config.models.resolve();

    Ok(config)
}
