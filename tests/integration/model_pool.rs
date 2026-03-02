use serde::Deserialize;
use std::{collections::HashSet, env, fs, path::PathBuf};

const DEFAULT_MODEL_POOL_FILE: &str = "tests/integration/hot_models.json";
const DEFAULT_CHAT_MODEL: &str = "x-ai/grok-code-fast-1";
const DEFAULT_REASONING_MODEL: &str = "deepseek/deepseek-r1";
const DEFAULT_STABLE_REGRESSION_MODELS: [&str; 3] = [
    "x-ai/grok-code-fast-1",
    "openai/gpt-4o-mini",
    "deepseek/deepseek-r1",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationTier {
    Stable,
    Hot,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct ModelPoolConfig {
    stable: StableModelPool,
    hot: HotModelPool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct StableModelPool {
    chat: Option<String>,
    reasoning: Option<String>,
    regression: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct HotModelPool {
    models: Vec<String>,
}

pub fn test_chat_model() -> String {
    env::var("OPENROUTER_TEST_CHAT_MODEL")
        .ok()
        .filter(|model| !model.trim().is_empty())
        .or_else(|| load_model_pool().and_then(|pool| normalize_model(pool.stable.chat)))
        .unwrap_or_else(|| DEFAULT_CHAT_MODEL.to_string())
}

pub fn test_reasoning_model() -> String {
    env::var("OPENROUTER_TEST_REASONING_MODEL")
        .ok()
        .filter(|model| !model.trim().is_empty())
        .or_else(|| load_model_pool().and_then(|pool| normalize_model(pool.stable.reasoning)))
        .unwrap_or_else(|| DEFAULT_REASONING_MODEL.to_string())
}

pub fn stable_regression_models() -> Vec<String> {
    if let Some(models) = env_model_list("OPENROUTER_TEST_STABLE_MODELS") {
        return models;
    }

    if let Some(pool) = load_model_pool() {
        let models = dedupe_models(pool.stable.regression);
        if !models.is_empty() {
            return models;
        }
    }

    DEFAULT_STABLE_REGRESSION_MODELS
        .iter()
        .map(ToString::to_string)
        .collect()
}

pub fn hot_models() -> Vec<String> {
    let mut models = if let Some(models) = env_model_list("OPENROUTER_TEST_HOT_MODELS") {
        models
    } else if let Some(pool) = load_model_pool() {
        let hot = dedupe_models(pool.hot.models);
        if hot.is_empty() {
            stable_regression_models()
        } else {
            hot
        }
    } else {
        stable_regression_models()
    };

    if let Some(limit) = hot_model_limit() {
        models.truncate(limit.min(models.len()));
    }

    models
}

pub fn integration_tier() -> IntegrationTier {
    match env::var("OPENROUTER_INTEGRATION_TIER")
        .unwrap_or_else(|_| "stable".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "hot" => IntegrationTier::Hot,
        _ => IntegrationTier::Stable,
    }
}

pub fn should_run_hot_sweep() -> bool {
    matches!(integration_tier(), IntegrationTier::Hot)
}

pub fn integration_tier_name() -> &'static str {
    match integration_tier() {
        IntegrationTier::Stable => "stable",
        IntegrationTier::Hot => "hot",
    }
}

fn load_model_pool() -> Option<ModelPoolConfig> {
    let path = env::var("OPENROUTER_TEST_MODEL_POOL_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_MODEL_POOL_FILE));

    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn hot_model_limit() -> Option<usize> {
    env::var("OPENROUTER_TEST_HOT_MODELS_LIMIT")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|limit| *limit > 0)
}

fn normalize_model(model: Option<String>) -> Option<String> {
    model.and_then(|model| {
        let trimmed = model.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn env_model_list(var: &str) -> Option<Vec<String>> {
    env::var(var)
        .ok()
        .map(|raw| parse_model_list(&raw))
        .filter(|models| !models.is_empty())
}

fn parse_model_list(raw: &str) -> Vec<String> {
    dedupe_models(
        raw.split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect(),
    )
}

fn dedupe_models(models: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for model in models {
        if seen.insert(model.clone()) {
            deduped.push(model);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::{ModelPoolConfig, parse_model_list};

    #[test]
    fn test_parse_model_list_handles_whitespace_and_dedup() {
        let models = parse_model_list("a/model-1, b/model-2, a/model-1,,");
        assert_eq!(models, vec!["a/model-1", "b/model-2"]);
    }

    #[test]
    fn test_model_pool_config_deserializes_missing_fields() {
        let parsed: ModelPoolConfig =
            serde_json::from_str(r#"{"stable":{"chat":"x-ai/grok-code-fast-1"}}"#).unwrap();
        assert_eq!(parsed.stable.chat.as_deref(), Some("x-ai/grok-code-fast-1"));
        assert!(parsed.stable.regression.is_empty());
        assert!(parsed.hot.models.is_empty());
    }
}
