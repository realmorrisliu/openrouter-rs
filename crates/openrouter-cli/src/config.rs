use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cli::GlobalOptions;

pub const ENV_CONFIG_PATH: &str = "OPENROUTER_CLI_CONFIG";
pub const ENV_PROFILE: &str = "OPENROUTER_PROFILE";
pub const ENV_API_KEY: &str = "OPENROUTER_API_KEY";
pub const ENV_MANAGEMENT_KEY: &str = "OPENROUTER_MANAGEMENT_KEY";
pub const ENV_BASE_URL: &str = "OPENROUTER_BASE_URL";

const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueSource {
    Flag,
    Env,
    ProfileConfig,
    Default,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedProfile {
    pub profile: String,
    pub profile_source: ValueSource,
    pub config_path: PathBuf,
    pub config_path_source: ValueSource,
    pub base_url: String,
    pub base_url_source: ValueSource,
    pub api_key: Option<String>,
    pub api_key_source: ValueSource,
    pub management_key: Option<String>,
    pub management_key_source: ValueSource,
}

#[derive(Debug, Clone, Default)]
pub struct Environment {
    vars: HashMap<String, String>,
}

impl Environment {
    pub fn from_process() -> Self {
        Self {
            vars: env::vars().collect(),
        }
    }

    #[cfg(test)]
    fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let vars = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        Self { vars }
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(String::as_str)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct CliConfigFile {
    #[serde(default = "default_profile_name")]
    default_profile: String,
    #[serde(default)]
    profiles: HashMap<String, CliProfileConfig>,
}

impl Default for CliConfigFile {
    fn default() -> Self {
        Self {
            default_profile: default_profile_name(),
            profiles: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CliProfileConfig {
    api_key: Option<String>,
    management_key: Option<String>,
    base_url: Option<String>,
}

fn default_profile_name() -> String {
    "default".to_string()
}

fn default_config_path(env: &Environment) -> PathBuf {
    if let Some(xdg_dir) = env.get("XDG_CONFIG_HOME") {
        return Path::new(xdg_dir).join("openrouter").join("profiles.toml");
    }

    if let Some(home) = env.get("HOME") {
        return Path::new(home)
            .join(".config")
            .join("openrouter")
            .join("profiles.toml");
    }

    PathBuf::from(".openrouter").join("profiles.toml")
}

fn read_config_file(path: &Path) -> Result<(CliConfigFile, bool)> {
    if !path.exists() {
        return Ok((CliConfigFile::default(), false));
    }

    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file at {}", path.display()))?;
    let parsed = toml::from_str::<CliConfigFile>(&raw)
        .with_context(|| format!("invalid config TOML at {}", path.display()))?;
    Ok((parsed, true))
}

fn resolve_opt(
    flag: Option<String>,
    env: Option<String>,
    profile: Option<String>,
) -> (Option<String>, ValueSource) {
    if let Some(value) = flag {
        return (Some(value), ValueSource::Flag);
    }
    if let Some(value) = env {
        return (Some(value), ValueSource::Env);
    }
    if let Some(value) = profile {
        return (Some(value), ValueSource::ProfileConfig);
    }
    (None, ValueSource::Default)
}

fn resolve_string(
    flag: Option<String>,
    env: Option<String>,
    profile: Option<String>,
    default: &str,
) -> (String, ValueSource) {
    if let Some(value) = flag {
        return (value, ValueSource::Flag);
    }
    if let Some(value) = env {
        return (value, ValueSource::Env);
    }
    if let Some(value) = profile {
        return (value, ValueSource::ProfileConfig);
    }
    (default.to_string(), ValueSource::Default)
}

pub fn resolve_profile(global: &GlobalOptions, env: &Environment) -> Result<ResolvedProfile> {
    let (config_path, config_path_source) = if let Some(path) = global.config.clone() {
        (path, ValueSource::Flag)
    } else if let Some(path) = env.get(ENV_CONFIG_PATH) {
        (PathBuf::from(path), ValueSource::Env)
    } else {
        (default_config_path(env), ValueSource::Default)
    };

    let (config_file, config_file_exists) = read_config_file(&config_path)?;
    let (profile_name, profile_source) = if let Some(name) = global.profile.clone() {
        (name, ValueSource::Flag)
    } else if let Some(name) = env.get(ENV_PROFILE) {
        (name.to_string(), ValueSource::Env)
    } else if config_file_exists && !config_file.default_profile.trim().is_empty() {
        (
            config_file.default_profile.clone(),
            ValueSource::ProfileConfig,
        )
    } else {
        (default_profile_name(), ValueSource::Default)
    };

    let profile_config = config_file
        .profiles
        .get(&profile_name)
        .cloned()
        .unwrap_or_default();

    let (api_key, api_key_source) = resolve_opt(
        global.api_key.clone(),
        env.get(ENV_API_KEY).map(ToOwned::to_owned),
        profile_config.api_key.clone(),
    );
    let (management_key, management_key_source) = resolve_opt(
        global.management_key.clone(),
        env.get(ENV_MANAGEMENT_KEY).map(ToOwned::to_owned),
        profile_config.management_key.clone(),
    );
    let (base_url, base_url_source) = resolve_string(
        global.base_url.clone(),
        env.get(ENV_BASE_URL).map(ToOwned::to_owned),
        profile_config.base_url.clone(),
        DEFAULT_BASE_URL,
    );

    Ok(ResolvedProfile {
        profile: profile_name,
        profile_source,
        config_path,
        config_path_source,
        base_url,
        base_url_source,
        api_key,
        api_key_source,
        management_key,
        management_key_source,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::TempDir;

    use super::*;

    fn global_options() -> GlobalOptions {
        GlobalOptions {
            config: None,
            profile: None,
            api_key: None,
            management_key: None,
            base_url: None,
            output: crate::cli::OutputFormat::Text,
        }
    }

    fn write_config(temp_dir: &TempDir, content: &str) -> PathBuf {
        let config_path = temp_dir.path().join("profiles.toml");
        fs::write(&config_path, content).expect("test config file should be written");
        config_path
    }

    #[test]
    fn test_resolve_profile_prefers_flag_over_env_over_profile_config() {
        let temp_dir = TempDir::new().expect("temp dir should build");
        let config_path = write_config(
            &temp_dir,
            r#"
default_profile = "default"

[profiles.default]
api_key = "file-api-key"
management_key = "file-mgmt-key"
base_url = "https://from-file.example/api/v1"
"#,
        );

        let mut global = global_options();
        global.config = Some(config_path);
        global.api_key = Some("flag-api-key".to_string());
        global.management_key = Some("flag-mgmt-key".to_string());
        global.base_url = Some("https://from-flag.example/api/v1".to_string());

        let env = Environment::from_pairs(&[
            (ENV_API_KEY, "env-api-key"),
            (ENV_MANAGEMENT_KEY, "env-mgmt-key"),
            (ENV_BASE_URL, "https://from-env.example/api/v1"),
        ]);
        let resolved = resolve_profile(&global, &env).expect("resolution should succeed");

        assert_eq!(resolved.api_key.as_deref(), Some("flag-api-key"));
        assert_eq!(resolved.api_key_source, ValueSource::Flag);
        assert_eq!(resolved.management_key.as_deref(), Some("flag-mgmt-key"));
        assert_eq!(resolved.management_key_source, ValueSource::Flag);
        assert_eq!(resolved.base_url, "https://from-flag.example/api/v1");
        assert_eq!(resolved.base_url_source, ValueSource::Flag);
    }

    #[test]
    fn test_resolve_profile_uses_env_when_flag_absent() {
        let temp_dir = TempDir::new().expect("temp dir should build");
        let config_path = write_config(
            &temp_dir,
            r#"
default_profile = "default"

[profiles.default]
api_key = "file-api-key"
management_key = "file-mgmt-key"
"#,
        );

        let mut global = global_options();
        global.config = Some(config_path);

        let env = Environment::from_pairs(&[
            (ENV_API_KEY, "env-api-key"),
            (ENV_MANAGEMENT_KEY, "env-mgmt-key"),
            (ENV_BASE_URL, "https://env.example/api/v1"),
        ]);
        let resolved = resolve_profile(&global, &env).expect("resolution should succeed");

        assert_eq!(resolved.api_key.as_deref(), Some("env-api-key"));
        assert_eq!(resolved.api_key_source, ValueSource::Env);
        assert_eq!(resolved.management_key.as_deref(), Some("env-mgmt-key"));
        assert_eq!(resolved.management_key_source, ValueSource::Env);
        assert_eq!(resolved.base_url, "https://env.example/api/v1");
        assert_eq!(resolved.base_url_source, ValueSource::Env);
    }

    #[test]
    fn test_resolve_profile_uses_config_when_flag_and_env_absent() {
        let temp_dir = TempDir::new().expect("temp dir should build");
        let config_path = write_config(
            &temp_dir,
            r#"
default_profile = "default"

[profiles.default]
api_key = "file-api-key"
management_key = "file-mgmt-key"
base_url = "https://from-file.example/api/v1"
"#,
        );

        let mut global = global_options();
        global.config = Some(config_path);

        let resolved =
            resolve_profile(&global, &Environment::default()).expect("resolution should succeed");

        assert_eq!(resolved.api_key.as_deref(), Some("file-api-key"));
        assert_eq!(resolved.api_key_source, ValueSource::ProfileConfig);
        assert_eq!(resolved.management_key.as_deref(), Some("file-mgmt-key"));
        assert_eq!(resolved.management_key_source, ValueSource::ProfileConfig);
        assert_eq!(resolved.base_url, "https://from-file.example/api/v1");
        assert_eq!(resolved.base_url_source, ValueSource::ProfileConfig);
    }

    #[test]
    fn test_resolve_profile_selection_priority() {
        let temp_dir = TempDir::new().expect("temp dir should build");
        let config_path = write_config(
            &temp_dir,
            r#"
default_profile = "dev"

[profiles.dev]
api_key = "dev-file-api-key"

[profiles.prod]
api_key = "prod-file-api-key"
"#,
        );

        let mut global = global_options();
        global.config = Some(config_path.clone());
        global.profile = Some("prod".to_string());
        let env = Environment::from_pairs(&[(ENV_PROFILE, "dev")]);
        let from_flag = resolve_profile(&global, &env).expect("resolution should succeed");
        assert_eq!(from_flag.profile, "prod");
        assert_eq!(from_flag.profile_source, ValueSource::Flag);
        assert_eq!(from_flag.api_key.as_deref(), Some("prod-file-api-key"));

        let mut global = global_options();
        global.config = Some(config_path.clone());
        let env = Environment::from_pairs(&[(ENV_PROFILE, "prod")]);
        let from_env = resolve_profile(&global, &env).expect("resolution should succeed");
        assert_eq!(from_env.profile, "prod");
        assert_eq!(from_env.profile_source, ValueSource::Env);
        assert_eq!(from_env.api_key.as_deref(), Some("prod-file-api-key"));

        let mut global = global_options();
        global.config = Some(config_path);
        let from_config =
            resolve_profile(&global, &Environment::default()).expect("resolution should succeed");
        assert_eq!(from_config.profile, "dev");
        assert_eq!(from_config.profile_source, ValueSource::ProfileConfig);
        assert_eq!(from_config.api_key.as_deref(), Some("dev-file-api-key"));
    }

    #[test]
    fn test_default_config_path_convention() {
        let env = Environment::from_pairs(&[("HOME", "/tmp/test-home")]);
        let path = default_config_path(&env);
        assert_eq!(
            path,
            Path::new("/tmp/test-home")
                .join(".config")
                .join("openrouter")
                .join("profiles.toml")
        );

        let env = Environment::from_pairs(&[
            ("HOME", "/tmp/test-home"),
            ("XDG_CONFIG_HOME", "/tmp/xdg-config"),
        ]);
        let path = default_config_path(&env);
        assert_eq!(
            path,
            Path::new("/tmp/xdg-config")
                .join("openrouter")
                .join("profiles.toml")
        );
    }

    #[test]
    fn test_missing_config_file_defaults_profile_source_to_default() {
        let temp_dir = TempDir::new().expect("temp dir should build");
        let config_path = temp_dir.path().join("missing-profiles.toml");

        let mut global = global_options();
        global.config = Some(config_path);

        let resolved =
            resolve_profile(&global, &Environment::default()).expect("resolution should succeed");
        assert_eq!(resolved.profile, "default");
        assert_eq!(resolved.profile_source, ValueSource::Default);
    }
}
