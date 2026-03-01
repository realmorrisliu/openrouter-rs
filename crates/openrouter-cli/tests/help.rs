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
