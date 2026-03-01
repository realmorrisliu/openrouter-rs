use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

#[test]
fn test_help_starts() {
    let mut cmd = cargo_bin_cmd!("openrouter-cli");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("OpenRouter CLI"))
        .stdout(contains("--profile"))
        .stdout(contains("profile"));
}

#[test]
fn test_parse_error_honors_json_output() {
    let mut cmd = cargo_bin_cmd!("openrouter-cli");
    cmd.arg("--output")
        .arg("json")
        .arg("usage")
        .arg("activity")
        .arg("--bad-flag");
    cmd.assert()
        .failure()
        .code(2)
        .stderr(contains("\"schema_version\": \"0.1\""))
        .stderr(contains("\"code\": \"cli_error\""))
        .stderr(contains("--bad-flag"));
}
