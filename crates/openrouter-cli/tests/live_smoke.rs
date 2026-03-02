use std::{
    env, thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";

fn env_truthy(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|raw| {
            matches!(
                raw.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn should_run_live() -> bool {
    env_truthy("OPENROUTER_CLI_RUN_LIVE")
}

fn should_run_live_write() -> bool {
    env_truthy("OPENROUTER_CLI_RUN_LIVE_WRITE")
}

fn configured_base_url() -> String {
    env::var("OPENROUTER_BASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
}

fn read_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn unique_name(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("openrouter-cli-{prefix}-{millis}")
}

fn run_cli_json(
    args: &[&str],
    api_key: Option<&str>,
    management_key: Option<&str>,
) -> Result<Value, String> {
    let mut cmd = cargo_bin_cmd!("openrouter-cli");

    if let Some(api_key) = api_key {
        cmd.arg("--api-key").arg(api_key);
    }
    if let Some(management_key) = management_key {
        cmd.arg("--management-key").arg(management_key);
    }

    cmd.arg("--base-url")
        .arg(configured_base_url())
        .arg("--output")
        .arg("json")
        .args(args)
        .env_remove("OPENROUTER_API_KEY")
        .env_remove("OPENROUTER_MANAGEMENT_KEY")
        .env_remove("OPENROUTER_BASE_URL")
        .env_remove("OPENROUTER_PROFILE")
        .env_remove("OPENROUTER_CLI_CONFIG");

    let output = cmd
        .output()
        .map_err(|error| format!("failed to execute CLI command {args:?}: {error}"))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "CLI command failed: args={args:?}, exit={:?}, stdout={stdout}, stderr={stderr}",
            output.status.code(),
        ));
    }

    serde_json::from_slice::<Value>(&output.stdout)
        .map_err(|error| format!("failed to parse JSON stdout for args {args:?}: {error}"))
}

fn ensure_schema(payload: &Value) -> Result<(), String> {
    if payload
        .get("schema_version")
        .and_then(Value::as_str)
        .is_some_and(|version| version == "0.1")
    {
        Ok(())
    } else {
        Err(format!("invalid schema envelope: {payload}"))
    }
}

fn ensure(condition: bool, message: impl Into<String>) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

#[test]
fn test_live_cli_read_smoke() {
    if !should_run_live() {
        println!(
            "Skipping CLI live read smoke; set OPENROUTER_CLI_RUN_LIVE=1 to enable real API checks"
        );
        return;
    }

    let Some(api_key) = read_env("OPENROUTER_API_KEY") else {
        println!("Skipping CLI live read smoke; OPENROUTER_API_KEY is not configured");
        return;
    };

    let management_key = read_env("OPENROUTER_MANAGEMENT_KEY");

    let run_result: Result<(), String> = (|| {
        let models = run_cli_json(
            &["models", "list"],
            Some(&api_key),
            management_key.as_deref(),
        )?;
        ensure_schema(&models)?;
        ensure(
            models
                .pointer("/data")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty()),
            "models list should return non-empty data array",
        )?;

        let providers = run_cli_json(
            &["providers", "list"],
            Some(&api_key),
            management_key.as_deref(),
        )?;
        ensure_schema(&providers)?;
        ensure(
            providers
                .pointer("/data")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty()),
            "providers list should return non-empty data array",
        )?;

        let credits = run_cli_json(
            &["credits", "show"],
            Some(&api_key),
            management_key.as_deref(),
        )?;
        ensure_schema(&credits)?;
        ensure(
            credits
                .pointer("/data/total_credits")
                .and_then(Value::as_f64)
                .is_some(),
            "credits show should include numeric total_credits",
        )?;

        if let Some(management_key) = management_key.as_deref() {
            let activity =
                run_cli_json(&["usage", "activity"], Some(&api_key), Some(management_key))?;
            ensure_schema(&activity)?;
            ensure(
                activity
                    .pointer("/data")
                    .and_then(Value::as_array)
                    .is_some(),
                "usage activity should include data array",
            )?;
        } else {
            println!(
                "Skipping usage activity check in read smoke; OPENROUTER_MANAGEMENT_KEY is not configured"
            );
        }

        Ok(())
    })();

    if let Err(error) = run_result {
        panic!("CLI live read smoke failed: {error}");
    }
}

#[test]
fn test_live_cli_management_key_lifecycle_smoke() {
    if !should_run_live() {
        println!("Skipping CLI live key lifecycle smoke; set OPENROUTER_CLI_RUN_LIVE=1 to enable");
        return;
    }
    if !should_run_live_write() {
        println!(
            "Skipping CLI live key lifecycle smoke; set OPENROUTER_CLI_RUN_LIVE_WRITE=1 to enable write-path checks"
        );
        return;
    }

    let Some(management_key) = read_env("OPENROUTER_MANAGEMENT_KEY") else {
        println!(
            "Skipping CLI live key lifecycle smoke; OPENROUTER_MANAGEMENT_KEY is not configured"
        );
        return;
    };

    let key_name = unique_name("key");
    let mut created_hash: Option<String> = None;

    let lifecycle_result: Result<(), String> = (|| {
        let created = run_cli_json(
            &["keys", "create", "--name", &key_name],
            None,
            Some(&management_key),
        )?;
        ensure_schema(&created)?;

        let hash = created
            .pointer("/data/hash")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("keys create response missing hash: {created}"))?
            .to_string();
        created_hash = Some(hash.clone());

        let mut seen_in_list = false;
        for _ in 0..5 {
            let listed = run_cli_json(
                &["keys", "list", "--include-disabled"],
                None,
                Some(&management_key),
            )?;
            ensure_schema(&listed)?;

            if listed
                .pointer("/data")
                .and_then(Value::as_array)
                .is_some_and(|items| {
                    items.iter().any(|item| {
                        item.get("hash")
                            .and_then(Value::as_str)
                            .is_some_and(|candidate| candidate == hash)
                    })
                })
            {
                seen_in_list = true;
                break;
            }

            thread::sleep(Duration::from_millis(400));
        }

        ensure(
            seen_in_list,
            format!("created key hash {hash} should appear in keys list"),
        )?;

        let fetched = run_cli_json(&["keys", "get", &hash], None, Some(&management_key))?;
        ensure_schema(&fetched)?;
        ensure(
            fetched
                .pointer("/data/hash")
                .and_then(Value::as_str)
                .is_some_and(|value| value == hash),
            format!("keys get should return hash {hash}"),
        )?;

        let deleted = run_cli_json(
            &["keys", "delete", &hash, "--yes"],
            None,
            Some(&management_key),
        )?;
        ensure_schema(&deleted)?;
        ensure(
            deleted
                .pointer("/data/deleted")
                .and_then(Value::as_bool)
                .is_some_and(|value| value),
            format!("keys delete should mark deleted=true for hash {hash}"),
        )?;

        created_hash = None;
        Ok(())
    })();

    if let Some(hash) = created_hash.take() {
        let _ = run_cli_json(
            &["keys", "delete", &hash, "--yes"],
            None,
            Some(&management_key),
        );
    }

    if let Err(error) = lifecycle_result {
        panic!("CLI live key lifecycle smoke failed: {error}");
    }
}

#[test]
fn test_live_cli_management_guardrail_lifecycle_smoke() {
    if !should_run_live() {
        println!(
            "Skipping CLI live guardrail lifecycle smoke; set OPENROUTER_CLI_RUN_LIVE=1 to enable"
        );
        return;
    }
    if !should_run_live_write() {
        println!(
            "Skipping CLI live guardrail lifecycle smoke; set OPENROUTER_CLI_RUN_LIVE_WRITE=1 to enable write-path checks"
        );
        return;
    }

    let Some(management_key) = read_env("OPENROUTER_MANAGEMENT_KEY") else {
        println!(
            "Skipping CLI live guardrail lifecycle smoke; OPENROUTER_MANAGEMENT_KEY is not configured"
        );
        return;
    };

    let guardrail_name = unique_name("guardrail");
    let mut created_guardrail_id: Option<String> = None;

    let lifecycle_result: Result<(), String> = (|| {
        let created = run_cli_json(
            &["guardrails", "create", "--name", &guardrail_name],
            None,
            Some(&management_key),
        )?;
        ensure_schema(&created)?;

        let guardrail_id = created
            .pointer("/data/id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("guardrails create response missing id: {created}"))?
            .to_string();
        created_guardrail_id = Some(guardrail_id.clone());

        let fetched = run_cli_json(
            &["guardrails", "get", &guardrail_id],
            None,
            Some(&management_key),
        )?;
        ensure_schema(&fetched)?;
        ensure(
            fetched
                .pointer("/data/id")
                .and_then(Value::as_str)
                .is_some_and(|value| value == guardrail_id),
            format!("guardrails get should return id {guardrail_id}"),
        )?;

        let deleted = run_cli_json(
            &["guardrails", "delete", &guardrail_id, "--yes"],
            None,
            Some(&management_key),
        )?;
        ensure_schema(&deleted)?;
        ensure(
            deleted
                .pointer("/data/deleted")
                .and_then(Value::as_bool)
                .is_some_and(|value| value),
            format!("guardrails delete should mark deleted=true for id {guardrail_id}"),
        )?;

        created_guardrail_id = None;
        Ok(())
    })();

    if let Some(guardrail_id) = created_guardrail_id.take() {
        let _ = run_cli_json(
            &["guardrails", "delete", &guardrail_id, "--yes"],
            None,
            Some(&management_key),
        );
    }

    if let Err(error) = lifecycle_result {
        panic!("CLI live guardrail lifecycle smoke failed: {error}");
    }
}
