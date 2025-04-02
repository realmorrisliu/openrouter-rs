use super::{OpenRouterConfig, get_config_path};
use crate::error::OpenRouterError;
use std::fs;

/// Loads and processes configuration with the following priority:
/// 1. Default values for missing config
/// 2. TOML file configuration
/// 3. Environment variable overrides
///
/// Environment variables take precedence to allow runtime customization
/// without modifying config files, which is particularly useful for:
/// - Containerized deployments
/// - CI/CD pipelines
/// - Temporary configuration changes
pub fn load_config() -> Result<OpenRouterConfig, OpenRouterError> {
    // First check standard config location to avoid unnecessary file operations
    let config_path = get_config_path()?;

    // Default config provides safe fallback values when:
    // - First-time setup
    // - Config file deleted/moved
    // - Permission issues
    if !config_path.exists() {
        return Ok(OpenRouterConfig::default());
    }

    // Read as string first instead of direct deserialization to:
    // 1. Provide better error context
    // 2. Allow potential future preprocessing
    let config_content = fs::read_to_string(&config_path).map_err(|e| {
        OpenRouterError::ConfigError(format!(
            "Failed to read config file at {}: {}",
            config_path.display(),
            e
        ))
    })?;

    // TOML parsing happens after file read to separate filesystem errors
    // from syntax errors, making troubleshooting clearer
    let mut config: OpenRouterConfig = toml::from_str(&config_content).map_err(|e| {
        OpenRouterError::ConfigError(format!(
            "Invalid config format in {}: {}",
            config_path.display(),
            e
        ))
    })?;

    // Environment variables override file config to support:
    // - Secrets management through env vars
    // - Runtime configuration changes
    // - Different environments (dev/staging/prod)
    if let Ok(url) = std::env::var("OPENROUTER_BASE_URL") {
        config.base_url = url;
    }

    // Late resolution of model presets ensures all configuration sources
    // (defaults, file, env vars) are merged before processing
    config.models.resolve()?;

    Ok(config)
}
