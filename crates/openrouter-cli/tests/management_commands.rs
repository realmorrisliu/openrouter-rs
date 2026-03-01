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
    request_text: String,
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
            request_text: format!("{header_text}{body_text}"),
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

fn base_cmd(base_url: &str) -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("openrouter-cli");
    cmd.arg("--management-key")
        .arg("mgmt-test-key")
        .arg("--base-url")
        .arg(base_url)
        .arg("--output")
        .arg("json")
        .env_remove("OPENROUTER_API_KEY")
        .env_remove("OPENROUTER_MANAGEMENT_KEY")
        .env_remove("OPENROUTER_BASE_URL")
        .env_remove("OPENROUTER_PROFILE")
        .env_remove("OPENROUTER_CLI_CONFIG");
    cmd
}

#[test]
fn test_keys_list_happy_path() {
    let (base_url, rx, server) =
        spawn_json_server(r#"{"data":[{"name":"ops","hash":"key_hash_1","disabled":false}]}"#);

    let mut cmd = base_cmd(&base_url);
    cmd.arg("keys")
        .arg("list")
        .arg("--offset")
        .arg("7")
        .arg("--include-disabled");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");

    assert_eq!(
        parsed.get("schema_version").and_then(Value::as_str),
        Some("0.1")
    );
    let data = parsed.get("data").expect("json envelope should have data");
    assert_eq!(
        data.as_array()
            .and_then(|values| values.first())
            .and_then(|entry| entry.get("hash"))
            .and_then(Value::as_str),
        Some("key_hash_1")
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/keys?offset=7&include_disabled=true HTTP/1.1"
    );
    let lower = captured.request_text.to_ascii_lowercase();
    assert!(
        lower.contains("authorization: bearer mgmt-test-key")
            || lower.contains("authorization:bearer mgmt-test-key"),
        "request should include management authorization header: {}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_guardrails_create_happy_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"gr_1","name":"Prod","created_at":"2026-03-01T00:00:00.000Z"}}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("guardrails")
        .arg("create")
        .arg("--name")
        .arg("Prod")
        .arg("--description")
        .arg("Production guardrail")
        .arg("--limit-usd")
        .arg("100")
        .arg("--reset-interval")
        .arg("monthly")
        .arg("--allowed-provider")
        .arg("openai")
        .arg("--allowed-model")
        .arg("openai/gpt-4.1")
        .arg("--enforce-zdr");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");

    assert_eq!(
        parsed.get("schema_version").and_then(Value::as_str),
        Some("0.1")
    );
    let data = parsed.get("data").expect("json envelope should have data");
    assert_eq!(data.get("id").and_then(Value::as_str), Some("gr_1"));
    assert_eq!(data.get("name").and_then(Value::as_str), Some("Prod"));

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(captured.request_line, "POST /api/v1/guardrails HTTP/1.1");
    let body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(body.get("name").and_then(Value::as_str), Some("Prod"));
    assert_eq!(
        body.get("description").and_then(Value::as_str),
        Some("Production guardrail")
    );
    assert_eq!(body.get("limit_usd").and_then(Value::as_f64), Some(100.0));
    assert_eq!(
        body.get("allowed_providers")
            .and_then(Value::as_array)
            .and_then(|values| values.first())
            .and_then(Value::as_str),
        Some("openai")
    );
    assert_eq!(
        body.get("allowed_models")
            .and_then(Value::as_array)
            .and_then(|values| values.first())
            .and_then(Value::as_str),
        Some("openai/gpt-4.1")
    );
    assert_eq!(body.get("enforce_zdr").and_then(Value::as_bool), Some(true));

    server.join().expect("server thread should finish");
}

#[test]
fn test_guardrail_key_assignment_assign_happy_path() {
    let (base_url, rx, server) = spawn_json_server(r#"{"assigned_count":2}"#);

    let mut cmd = base_cmd(&base_url);
    cmd.arg("guardrails")
        .arg("assignments")
        .arg("keys")
        .arg("assign")
        .arg("team/prod 1")
        .arg("key_hash_a")
        .arg("key_hash_b");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");
    assert_eq!(
        parsed
            .get("data")
            .and_then(|value| value.get("assigned_count"))
            .and_then(Value::as_f64),
        Some(2.0)
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/guardrails/team%2Fprod%201/assignments/keys HTTP/1.1"
    );
    let body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(
        body.get("key_hashes")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(2)
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_guardrail_member_assignment_list_global_happy_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[{"id":"gma_1","user_id":"user_1","organization_id":"org_1","guardrail_id":"gr_1","assigned_by":"user_admin","created_at":"2026-03-01T00:00:00Z"}],"total_count":1}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("guardrails")
        .arg("assignments")
        .arg("members")
        .arg("list")
        .arg("--offset")
        .arg("3")
        .arg("--limit")
        .arg("5");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");
    assert_eq!(
        parsed
            .get("data")
            .and_then(|value| value.get("total_count"))
            .and_then(Value::as_f64),
        Some(1.0)
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/guardrails/assignments/members?offset=3&limit=5 HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_guardrails_update_can_clear_allowlists() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"gr_1","name":"Prod","created_at":"2026-03-01T00:00:00.000Z"}}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("guardrails")
        .arg("update")
        .arg("gr_1")
        .arg("--clear-allowed-providers")
        .arg("--clear-allowed-models");
    cmd.assert().success();

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "PATCH /api/v1/guardrails/gr_1 HTTP/1.1"
    );
    let body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(
        body.get("allowed_providers")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(0)
    );
    assert_eq!(
        body.get("allowed_models")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(0)
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_keys_delete_requires_yes() {
    let mut cmd = base_cmd("http://127.0.0.1:9/api/v1");
    cmd.arg("keys").arg("delete").arg("key_hash_1");
    cmd.assert().failure().stderr(contains("without --yes"));
}

#[test]
fn test_guardrail_key_assignment_unassign_requires_yes() {
    let mut cmd = base_cmd("http://127.0.0.1:9/api/v1");
    cmd.arg("guardrails")
        .arg("assignments")
        .arg("keys")
        .arg("unassign")
        .arg("gr_1")
        .arg("key_hash_1");
    cmd.assert().failure().stderr(contains("without --yes"));
}
