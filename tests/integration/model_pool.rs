use serde::Deserialize;
use std::{collections::HashSet, env, fs, path::PathBuf};

const DEFAULT_MODEL_POOL_FILE: &str = "tests/integration/model_pool.json";
const LEGACY_MODEL_POOL_FILE: &str = "tests/integration/hot_models.json";
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
    responses: ResponsesModelPool,
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

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct ResponsesModelPool {
    hot: HotModelPool,
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

pub fn hot_responses_models() -> Vec<String> {
    let mut models = if let Some(models) = env_model_list_with_legacy_aliases(
        "OPENROUTER_TEST_HOT_RESPONSES_MODELS",
        &["OPENROUTER_TEST_HOT_MODELS"],
    ) {
        models
    } else if let Some(pool) = load_model_pool() {
        let hot = dedupe_models(if pool.responses.hot.models.is_empty() {
            pool.hot.models
        } else {
            pool.responses.hot.models
        });
        if hot.is_empty() {
            stable_regression_models()
        } else {
            hot
        }
    } else {
        stable_regression_models()
    };

    if let Some(limit) = hot_responses_model_limit() {
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

pub fn should_run_hot_responses_sweep() -> bool {
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

    let raw = fs::read_to_string(&path).ok().or_else(|| {
        (path == PathBuf::from(DEFAULT_MODEL_POOL_FILE))
            .then(|| fs::read_to_string(LEGACY_MODEL_POOL_FILE).ok())
            .flatten()
    })?;
    serde_json::from_str(&raw).ok()
}

fn hot_responses_model_limit() -> Option<usize> {
    env_var_with_legacy_aliases(
        "OPENROUTER_TEST_HOT_RESPONSES_MODELS_LIMIT",
        &["OPENROUTER_TEST_HOT_MODELS_LIMIT"],
    )
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

fn env_var_with_legacy_aliases(
    primary: &str,
    legacy_aliases: &[&str],
) -> Result<String, env::VarError> {
    env::var(primary).or_else(|_| {
        legacy_aliases
            .iter()
            .find_map(|alias| env::var(alias).ok())
            .ok_or(env::VarError::NotPresent)
    })
}

fn env_model_list_with_legacy_aliases(
    primary: &str,
    legacy_aliases: &[&str],
) -> Option<Vec<String>> {
    env_var_with_legacy_aliases(primary, legacy_aliases)
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
        assert!(parsed.responses.hot.models.is_empty());
        assert!(parsed.hot.models.is_empty());
    }

    #[test]
    fn test_model_pool_config_deserializes_responses_hot_models() {
        let parsed: ModelPoolConfig = serde_json::from_str(
            r#"{"responses":{"hot":{"models":["openai/gpt-5.4-pro","x-ai/grok-4.20-beta"]}}}"#,
        )
        .unwrap();
        assert_eq!(
            parsed.responses.hot.models,
            vec!["openai/gpt-5.4-pro", "x-ai/grok-4.20-beta"]
        );
    }
}
