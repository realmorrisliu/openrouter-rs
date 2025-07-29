//! # Configuration Management
//!
//! This module provides configuration management for the OpenRouter Rust SDK,
//! including model presets, default settings, and configuration loading.
//!
//! ## Overview
//!
//! The configuration system supports:
//! - **Model Presets**: Predefined groups of models for specific use cases
//! - **Default Configuration**: Built-in TOML configuration with sensible defaults
//! - **Custom Configuration**: Load configuration from external TOML files
//! - **Model Resolution**: Automatic expansion of preset references to actual model IDs
//!
//! ## Model Presets
//!
//! The SDK includes three built-in presets:
//!
//! - **`programming`**: High-performance models optimized for code generation and programming tasks
//! - **`reasoning`**: Models with advanced reasoning capabilities and chain-of-thought processing
//! - **`free`**: Free-tier models suitable for experimentation and development
//!
//! ## Configuration Format
//!
//! Configuration uses TOML format:
//!
//! ```toml
//! default_model = "deepseek/deepseek-chat-v3-0324:free"
//!
//! [models]
//! enable = ["preset:programming", "preset:reasoning"]
//!
//! [models.presets]
//! programming = [
//!   "anthropic/claude-sonnet-4",
//!   "google/gemini-2.5-flash",
//!   "qwen/qwen3-coder"
//! ]
//! reasoning = [
//!   "anthropic/claude-sonnet-4",
//!   "deepseek/deepseek-r1:free",
//!   "google/gemini-2.5-pro"
//! ]
//! ```
//!
//! ## Usage Examples
//!
//! ### Using Default Configuration
//!
//! ```rust
//! use openrouter_rs::config::OpenRouterConfig;
//!
//! let config = OpenRouterConfig::default();
//! println!("Default model: {}", config.get_default_model());
//! println!("Available models: {:?}", config.get_resolved_models());
//! ```
//!
//! ### Loading Custom Configuration
//!
//! ```rust
//! use openrouter_rs::config::load_config;
//!
//! let config = load_config("./my_config.toml")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Checking Model Availability
//!
//! ```rust
//! use openrouter_rs::config::OpenRouterConfig;
//!
//! let config = OpenRouterConfig::default();
//! if config.models.is_enabled("anthropic/claude-sonnet-4") {
//!     println!("Claude Sonnet 4 is available!");
//! }
//! ```

mod model;

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

pub use model::ModelConfig;

use crate::error::OpenRouterError;

/// Main configuration structure for OpenRouter SDK
///
/// Contains the default model to use and model configuration including presets.
/// This structure can be loaded from TOML files or created with sensible defaults.
///
/// # Examples
///
/// ```rust
/// use openrouter_rs::config::OpenRouterConfig;
///
/// // Use default configuration with built-in presets
/// let config = OpenRouterConfig::default();
///
/// // Access default model
/// println!("Default model: {}", config.get_default_model());
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenRouterConfig {
    /// The default model to use when no specific model is requested
    #[serde(default)]
    pub default_model: String,

    /// Model configuration including enabled models and presets
    #[serde(default)]
    pub models: ModelConfig,
}

impl OpenRouterConfig {
    /// Resolves all preset references in the model configuration
    ///
    /// This method expands any `preset:name` entries in the enabled models list
    /// to their actual model IDs. This is automatically called when loading
    /// configuration but can be called manually if the configuration is modified.
    pub fn resolve_models(&mut self) {
        self.models.resolve();
    }

    /// Returns the default model ID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openrouter_rs::config::OpenRouterConfig;
    ///
    /// let config = OpenRouterConfig::default();
    /// assert_eq!(config.get_default_model(), "deepseek/deepseek-chat-v3-0324:free");
    /// ```
    pub fn get_default_model(&self) -> &str {
        &self.default_model
    }

    /// Returns a list of all resolved (enabled) model IDs
    ///
    /// This includes models from expanded presets and individual model IDs.
    /// Duplicates are automatically removed during resolution.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openrouter_rs::config::OpenRouterConfig;
    ///
    /// let config = OpenRouterConfig::default();
    /// let models = config.get_resolved_models();
    /// assert!(!models.is_empty());
    /// ```
    pub fn get_resolved_models(&self) -> Vec<String> {
        self.models.resolved_models.clone()
    }
}

impl Default for OpenRouterConfig {
    /// Creates a default configuration with built-in model presets
    ///
    /// The default configuration includes:
    /// - Default model: `deepseek/deepseek-chat-v3-0324:free`
    /// - Three presets: `programming`, `reasoning`, and `free`
    /// - Automatic resolution of preset references
    ///
    /// # Examples
    ///
    /// ```rust
    /// use openrouter_rs::config::OpenRouterConfig;
    ///
    /// let config = OpenRouterConfig::default();
    /// assert_eq!(config.models.presets.len(), 3);
    /// assert!(config.models.presets.contains_key("programming"));
    /// assert!(config.models.presets.contains_key("reasoning"));
    /// assert!(config.models.presets.contains_key("free"));
    /// ```
    fn default() -> Self {
        let default_config = include_str!("default_config.toml");
        let mut config = toml::from_str(default_config).unwrap_or(Self {
            default_model: "deepseek/deepseek-chat-v3-0324:free".to_string(),
            models: ModelConfig::default(),
        });

        config.models.resolve();

        config
    }
}

/// Loads configuration from a TOML file
///
/// If the file doesn't exist, returns the default configuration.
/// If the file exists but contains invalid TOML or configuration structure,
/// returns an error.
///
/// # Arguments
///
/// * `config_path` - Path to the TOML configuration file
///
/// # Returns
///
/// * `Ok(OpenRouterConfig)` - Successfully loaded and resolved configuration
/// * `Err(OpenRouterError)` - Failed to read or parse the configuration file
///
/// # Examples
///
/// ```rust
/// use openrouter_rs::config::load_config;
///
/// // Load config from file, or use defaults if file doesn't exist
/// let config = load_config("./openrouter.toml").unwrap();
/// ```
pub fn load_config(config_path: impl AsRef<Path>) -> Result<OpenRouterConfig, OpenRouterError> {
    let config_path = config_path.as_ref();

    if !config_path.exists() {
        return Ok(OpenRouterConfig::default());
    }

    let config_content = fs::read_to_string(config_path).map_err(|e| {
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
