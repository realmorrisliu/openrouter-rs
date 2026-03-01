use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;
use tempfile::TempDir;

#[test]
fn test_profile_show_resolves_from_config() {
    let temp_dir = TempDir::new().expect("temp dir should build");
    let config_path = temp_dir.path().join("profiles.toml");
    std::fs::write(
        &config_path,
        r#"
default_profile = "default"

[profiles.default]
api_key = "test-api-key"
management_key = "test-management-key"
base_url = "https://profile.example/api/v1"
"#,
    )
    .expect("test config should be written");

    let mut cmd = cargo_bin_cmd!("openrouter-cli");
    cmd.arg("--config")
        .arg(config_path)
        .arg("--output")
        .arg("json")
        .arg("profile")
        .arg("show")
        .env_remove("OPENROUTER_API_KEY")
        .env_remove("OPENROUTER_MANAGEMENT_KEY")
        .env_remove("OPENROUTER_BASE_URL")
        .env_remove("OPENROUTER_PROFILE");
    let output = cmd.assert().success().get_output().stdout.clone();
    let json: Value = serde_json::from_slice(&output).expect("stdout should be JSON");

    assert_eq!(json.get("profile").and_then(Value::as_str), Some("default"));
    assert_eq!(
        json.get("base_url").and_then(Value::as_str),
        Some("https://profile.example/api/v1")
    );
    assert_eq!(
        json.get("api_key_present").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("management_key_present").and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn test_profile_show_resolves_flag_over_env() {
    let temp_dir = TempDir::new().expect("temp dir should build");
    let config_path = temp_dir.path().join("profiles.toml");
    std::fs::write(
        &config_path,
        r#"
default_profile = "default"

[profiles.default]
api_key = "file-api-key"
"#,
    )
    .expect("test config should be written");

    let mut cmd = cargo_bin_cmd!("openrouter-cli");
    cmd.arg("--config")
        .arg(config_path)
        .arg("--api-key")
        .arg("flag-api-key")
        .arg("--output")
        .arg("json")
        .arg("profile")
        .arg("show")
        .env("OPENROUTER_API_KEY", "env-api-key");
    let output = cmd.assert().success().get_output().stdout.clone();
    let json: Value = serde_json::from_slice(&output).expect("stdout should be JSON");

    assert_eq!(
        json.get("api_key_source").and_then(Value::as_str),
        Some("flag")
    );
}
