use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use serde_json::Value;

struct CapturedRequest {
    request_line: String,
    body_text: String,
}

fn spawn_json_server(
    response_body: &str,
) -> (
    String,
    mpsc::Receiver<CapturedRequest>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let body = response_body.to_string();
    let (tx, rx) = mpsc::channel::<CapturedRequest>();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("server should accept one connection");

        let mut request_bytes = Vec::new();
        let mut chunk = [0_u8; 1024];
        let header_end = loop {
            let read = stream.read(&mut chunk).expect("server should read request");
            if read == 0 {
                break None;
            }
            request_bytes.extend_from_slice(&chunk[..read]);
            if let Some(pos) = request_bytes
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
            {
                break Some(pos + 4);
            }
        }
        .expect("request should include header terminator");

        let header_text = String::from_utf8_lossy(&request_bytes[..header_end]).to_string();
        let request_line = header_text.lines().next().unwrap_or_default().to_string();
        let content_length = header_text
            .lines()
            .find_map(|line| {
                let lower = line.to_ascii_lowercase();
                if lower.starts_with("content-length:") {
                    line.split(':').nth(1)?.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let mut body_bytes = request_bytes[header_end..].to_vec();
        while body_bytes.len() < content_length {
            let read = stream
                .read(&mut chunk)
                .expect("server should read request body");
            if read == 0 {
                break;
            }
            body_bytes.extend_from_slice(&chunk[..read]);
        }
        let body_text = String::from_utf8_lossy(&body_bytes[..content_length]).to_string();

        tx.send(CapturedRequest {
            request_line,
            body_text,
        })
        .expect("captured request should send");

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    (format!("http://{addr}/api/v1"), rx, server)
}

fn base_cmd(base_url: &str, output: &str) -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("openrouter-cli");
    cmd.arg("--base-url")
        .arg(base_url)
        .arg("--output")
        .arg(output)
        .env_remove("OPENROUTER_API_KEY")
        .env_remove("OPENROUTER_MANAGEMENT_KEY")
        .env_remove("OPENROUTER_BASE_URL")
        .env_remove("OPENROUTER_PROFILE")
        .env_remove("OPENROUTER_CLI_CONFIG");
    cmd
}

#[test]
fn test_credits_show_json_contract() {
    let (base_url, rx, server) =
        spawn_json_server(r#"{"data":{"total_credits":120.5,"total_usage":40.25}}"#);

    let mut cmd = base_cmd(&base_url, "json");
    cmd.arg("--api-key")
        .arg("api-test-key")
        .arg("credits")
        .arg("show");
    let output = cmd.assert().success().get_output().stdout.clone();
    let json: Value = serde_json::from_slice(&output).expect("stdout should be json");

    assert_eq!(
        json.get("schema_version").and_then(Value::as_str),
        Some("0.1")
    );
    assert_eq!(
        json.get("data")
            .and_then(|value| value.get("total_credits"))
            .and_then(Value::as_f64),
        Some(120.5)
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(captured.request_line, "GET /api/v1/credits HTTP/1.1");

    server.join().expect("server thread should finish");
}

#[test]
fn test_credits_charge_json_contract() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"addresses":{"ethereum":"0xabc"},"calldata":{"tx":"0x123"},"chain_id":1,"sender":"0xsender","id":"charge_1"}}"#,
    );

    let mut cmd = base_cmd(&base_url, "json");
    cmd.arg("--api-key")
        .arg("api-test-key")
        .arg("credits")
        .arg("charge")
        .arg("--amount")
        .arg("10.5")
        .arg("--sender")
        .arg("0xsender")
        .arg("--chain-id")
        .arg("1");
    let output = cmd.assert().success().get_output().stdout.clone();
    let json: Value = serde_json::from_slice(&output).expect("stdout should be json");

    assert_eq!(
        json.get("schema_version").and_then(Value::as_str),
        Some("0.1")
    );
    assert_eq!(
        json.get("data")
            .and_then(|value| value.get("id"))
            .and_then(Value::as_str),
        Some("charge_1")
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/credits/coinbase HTTP/1.1"
    );
    let request_body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(
        request_body.get("sender").and_then(Value::as_str),
        Some("0xsender")
    );
    assert_eq!(
        request_body.get("amount").and_then(Value::as_f64),
        Some(10.5)
    );
    assert_eq!(
        request_body.get("chain_id").and_then(Value::as_u64),
        Some(1)
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_usage_activity_json_contract() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[{"date":"2026-02-28","model":"openai/gpt-4.1","model_permaslug":"openai/gpt-4.1-2025-04-14","endpoint_id":"endpoint_1","provider_name":"OpenAI","usage":1.5,"byok_usage_inference":0.0,"requests":2,"prompt_tokens":20,"completion_tokens":40,"reasoning_tokens":5}]}"#,
    );

    let mut cmd = base_cmd(&base_url, "json");
    cmd.arg("--management-key")
        .arg("mgmt-test-key")
        .arg("usage")
        .arg("activity")
        .arg("--date")
        .arg("2026-02-28");
    let output = cmd.assert().success().get_output().stdout.clone();
    let json: Value = serde_json::from_slice(&output).expect("stdout should be json");

    assert_eq!(
        json.get("schema_version").and_then(Value::as_str),
        Some("0.1")
    );
    assert_eq!(
        json.get("data")
            .and_then(Value::as_array)
            .and_then(|values| values.first())
            .and_then(|entry| entry.get("model"))
            .and_then(Value::as_str),
        Some("openai/gpt-4.1")
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/activity?date=2026-02-28 HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_usage_activity_requires_management_key_with_json_error() {
    let mut cmd = base_cmd("http://127.0.0.1:9/api/v1", "json");
    cmd.arg("usage").arg("activity");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(contains("\"schema_version\": \"0.1\""))
        .stderr(contains("\"code\": \"cli_error\""))
        .stderr(contains("management key is required"));
}

#[test]
fn test_credits_show_table_output() {
    let (base_url, _rx, server) =
        spawn_json_server(r#"{"data":{"total_credits":120.5,"total_usage":40.25}}"#);

    let mut cmd = base_cmd(&base_url, "table");
    cmd.arg("--api-key")
        .arg("api-test-key")
        .arg("credits")
        .arg("show");
    cmd.assert()
        .success()
        .stdout(contains("total_credits"))
        .stdout(contains("total_usage"))
        .stdout(contains("120.5"));

    server.join().expect("server thread should finish");
}
