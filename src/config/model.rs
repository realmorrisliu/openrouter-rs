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
///     "preset:thinking",
///     "preset:coding@sonnet",
///     "google/gemini-2.5-pro-exp-03-25:free"
/// ]
///
/// [models.presets]
/// thinking = ["openai/o3-mini-high", "anthropic/claude-3.7-sonnet:thinking", "deepseek/deepseek-r1"]
/// coding = ["anthropic/claude-3.7-sonnet", "deepseek/deepseek-chat-v3-0324"]
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
    /// let mut config = ModelConfig {
    ///     enable: vec!["preset:coding".into()],
    ///     presets: {
    ///         let mut map = HashMap::new();
    ///         map.insert("coding".into(), vec!["anthropic/claude-3.7-sonnet".into()]);
    ///         map
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// config.resolve();
    /// assert!(config.is_enabled("anthropic/claude-3.7-sonnet"));
    /// ```
    pub fn resolve(&mut self) {
        let mut new_models = HashSet::new();

        for entry in &self.enable {
            if let Some(preset_name) = entry.strip_prefix("preset:") {
                // Handle selective enable with @
                let (preset, filter) = preset_name.split_once('@').unwrap_or((preset_name, ""));

                if let Some(models) = self.presets.get(preset) {
                    for model in models {
                        if filter.is_empty() || model.contains(filter) {
                            new_models.insert(model.to_string());
                        }
                    }
                }
            } else {
                // Directly add single model
                new_models.insert(entry.to_string());
            }
        }

        self.resolved_models = new_models.into_iter().collect();
    }

    /// Checks if a specific model is enabled
    ///
    /// # Example
    /// ```rust
    /// let config = ModelConfig {
    ///     resolved_models: vec!["anthropic/claude-3.7-sonnet".into()],
    ///     ..Default::default()
    /// };
    /// assert!(config.is_enabled("anthropic/claude-3.7-sonnet"));
    /// assert!(!config.is_enabled("openai/gpt-4o-mini"));
    /// ```
    pub fn is_enabled(&self, model_id: &str) -> bool {
        self.resolved_models.contains(&model_id.to_string())
    }
}
