use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Configuration for model selection and presets
///
/// # Examples
///
/// Basic usage from TOML config:
/// ```toml
/// [models]
/// enable = [
///     "preset:programming",
///     "preset:reasoning@sonnet",
///     "google/gemini-2.0-flash-exp:free"
/// ]
///
/// [models.presets]
/// programming = ["anthropic/claude-sonnet-4", "google/gemini-2.5-flash", "qwen/qwen3-coder"]
/// reasoning = ["anthropic/claude-sonnet-4", "google/gemini-2.5-pro", "deepseek/deepseek-r1:free"]
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelConfig {
    /// List of enabled models and presets
    ///
    /// Supports three formats:
    /// - `"model_id"`: Directly enable a specific model
    /// - `"preset:preset_name"`: Enable all models in a preset
    /// - `"preset:preset_name@filter"`: Enable models in preset matching filter
    #[serde(default)]
    pub enable: Vec<String>, // Supports preset: prefix syntax

    #[serde(rename = "presets", default)]
    pub presets: HashMap<String, Vec<String>>,

    /// Resolved list of enabled model IDs (calculated at runtime)
    #[serde(skip)]
    pub resolved_models: Vec<String>,
}

impl ModelConfig {
    /// Resolves the final list of enabled models by processing presets
    ///
    /// # Example
    /// ```rust
    /// use openrouter_rs::config::ModelConfig;
    /// use std::collections::HashMap;
    ///
    /// let mut config = ModelConfig {
    ///     enable: vec!["preset:programming".into()],
    ///     presets: {
    ///         let mut map = HashMap::new();
    ///         map.insert("programming".into(), vec!["anthropic/claude-sonnet-4".into()]);
    ///         map
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// config.resolve();
    /// assert!(config.is_enabled("anthropic/claude-sonnet-4"));
    /// ```
    pub fn resolve(&mut self) {
        let mut seen = HashSet::new();
        let mut resolved_models = Vec::new();

        for entry in &self.enable {
            if let Some(preset_name) = entry.strip_prefix("preset:") {
                // Handle selective enable with @
                let (preset, filter) = preset_name.split_once('@').unwrap_or((preset_name, ""));

                if let Some(models) = self.presets.get(preset) {
                    for model in models {
                        if filter.is_empty() || model.contains(filter) {
                            let model_id = model.to_string();
                            if seen.insert(model_id.clone()) {
                                resolved_models.push(model_id);
                            }
                        }
                    }
                }
            } else {
                // Directly add single model
                let model_id = entry.to_string();
                if seen.insert(model_id.clone()) {
                    resolved_models.push(model_id);
                }
            }
        }

        self.resolved_models = resolved_models;
    }

    /// Checks if a specific model is enabled
    ///
    /// # Example
    /// ```rust
    /// use openrouter_rs::config::ModelConfig;
    ///
    /// let config = ModelConfig {
    ///     resolved_models: vec!["anthropic/claude-sonnet-4".into()],
    ///     ..Default::default()
    /// };
    /// assert!(config.is_enabled("anthropic/claude-sonnet-4"));
    /// assert!(!config.is_enabled("google/gemini-2.5-flash"));
    /// ```
    pub fn is_enabled(&self, model_id: &str) -> bool {
        self.resolved_models.contains(&model_id.to_string())
    }
}
