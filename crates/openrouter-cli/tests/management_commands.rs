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
fn test_keys_list_with_workspace_filter_happy_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[{"name":"ops","hash":"key_hash_1","disabled":false,"workspace_id":"ws_123"}]}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("keys")
        .arg("list")
        .arg("--workspace-id")
        .arg("ws_123");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");

    assert_eq!(
        parsed
            .pointer("/data/0/workspace_id")
            .and_then(Value::as_str),
        Some("ws_123")
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/keys?workspace_id=ws_123 HTTP/1.1"
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
fn test_guardrails_list_with_workspace_filter_happy_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[{"id":"gr_1","name":"Prod","workspace_id":"ws_123","created_at":"2026-03-01T00:00:00.000Z"}],"total_count":1}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("guardrails")
        .arg("list")
        .arg("--workspace-id")
        .arg("ws_123");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");

    assert_eq!(
        parsed
            .pointer("/data/data/0/workspace_id")
            .and_then(Value::as_str),
        Some("ws_123")
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/guardrails?workspace_id=ws_123 HTTP/1.1"
    );

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
fn test_workspaces_list_happy_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[{"id":"ws_1","name":"Core","slug":"core","io_logging_api_key_ids":null,"io_logging_sampling_rate":1.0,"is_observability_io_logging_enabled":false,"is_observability_broadcast_enabled":true,"is_data_discount_logging_enabled":false,"created_at":"2026-03-01T00:00:00.000Z"}],"total_count":1}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("workspaces")
        .arg("list")
        .arg("--offset")
        .arg("2")
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
    assert_eq!(
        parsed.pointer("/data/data/0/slug").and_then(Value::as_str),
        Some("core")
    );
    assert_eq!(
        parsed
            .pointer("/data/data/0/io_logging_sampling_rate")
            .and_then(Value::as_f64),
        Some(1.0)
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/workspaces?offset=2&limit=5 HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_workspaces_create_happy_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"ws_1","name":"Core","slug":"core","description":"Core team","default_text_model":"openai/gpt-4.1","io_logging_api_key_ids":[123],"io_logging_sampling_rate":0.5,"is_observability_io_logging_enabled":true,"is_observability_broadcast_enabled":false,"is_data_discount_logging_enabled":true,"created_at":"2026-03-01T00:00:00.000Z"}}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("workspaces")
        .arg("create")
        .arg("--name")
        .arg("Core")
        .arg("--slug")
        .arg("core")
        .arg("--description")
        .arg("Core team")
        .arg("--default-text-model")
        .arg("openai/gpt-4.1")
        .arg("--enable-data-discount-logging")
        .arg("--disable-observability-broadcast")
        .arg("--enable-observability-io-logging");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");

    assert_eq!(
        parsed.pointer("/data/id").and_then(Value::as_str),
        Some("ws_1")
    );
    assert_eq!(
        parsed
            .pointer("/data/io_logging_api_key_ids/0")
            .and_then(Value::as_u64),
        Some(123)
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(captured.request_line, "POST /api/v1/workspaces HTTP/1.1");
    let body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(body.get("name").and_then(Value::as_str), Some("Core"));
    assert_eq!(body.get("slug").and_then(Value::as_str), Some("core"));
    assert_eq!(
        body.get("description").and_then(Value::as_str),
        Some("Core team")
    );
    assert_eq!(
        body.get("default_text_model").and_then(Value::as_str),
        Some("openai/gpt-4.1")
    );
    assert_eq!(
        body.get("is_data_discount_logging_enabled")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        body.get("is_observability_broadcast_enabled")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        body.get("is_observability_io_logging_enabled")
            .and_then(Value::as_bool),
        Some(true)
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_workspaces_create_sends_io_logging_filters() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"ws_1","name":"Core","slug":"core","io_logging_api_key_ids":[101,202],"io_logging_sampling_rate":0.25,"is_observability_io_logging_enabled":true,"is_observability_broadcast_enabled":false,"is_data_discount_logging_enabled":true,"created_at":"2026-03-01T00:00:00.000Z"}}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("workspaces")
        .arg("create")
        .arg("--name")
        .arg("Core")
        .arg("--io-logging-api-key-id")
        .arg("101")
        .arg("--io-logging-api-key-id")
        .arg("202")
        .arg("--io-logging-sampling-rate")
        .arg("0.25");
    cmd.assert().success();

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    let body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(
        body.get("io_logging_api_key_ids"),
        Some(&serde_json::json!([101, 202]))
    );
    assert_eq!(
        body.get("io_logging_sampling_rate").and_then(Value::as_f64),
        Some(0.25)
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_workspaces_get_happy_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"ws_1","name":"Core","slug":"core","io_logging_api_key_ids":null,"io_logging_sampling_rate":1.0,"is_observability_io_logging_enabled":false,"is_observability_broadcast_enabled":true,"is_data_discount_logging_enabled":false,"created_at":"2026-03-01T00:00:00.000Z"}}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("workspaces").arg("get").arg("ws_1");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");
    assert_eq!(
        parsed.pointer("/data/slug").and_then(Value::as_str),
        Some("core")
    );
    assert_eq!(
        parsed
            .pointer("/data/io_logging_sampling_rate")
            .and_then(Value::as_f64),
        Some(1.0)
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/workspaces/ws_1 HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_workspaces_update_happy_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"ws_1","name":"Core Updated","slug":"core","io_logging_api_key_ids":null,"io_logging_sampling_rate":1.0,"is_observability_io_logging_enabled":false,"is_observability_broadcast_enabled":true,"is_data_discount_logging_enabled":false,"created_at":"2026-03-01T00:00:00.000Z","updated_at":"2026-03-02T00:00:00.000Z"}}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("workspaces")
        .arg("update")
        .arg("ws_1")
        .arg("--name")
        .arg("Core Updated")
        .arg("--disable-data-discount-logging");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");
    assert_eq!(
        parsed.pointer("/data/name").and_then(Value::as_str),
        Some("Core Updated")
    );
    assert_eq!(
        parsed
            .pointer("/data/io_logging_sampling_rate")
            .and_then(Value::as_f64),
        Some(1.0)
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "PATCH /api/v1/workspaces/ws_1 HTTP/1.1"
    );
    let body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(
        body.get("name").and_then(Value::as_str),
        Some("Core Updated")
    );
    assert_eq!(
        body.get("is_data_discount_logging_enabled")
            .and_then(Value::as_bool),
        Some(false)
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_workspaces_update_can_clear_io_logging_filters() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"ws_1","name":"Core","slug":"core","io_logging_api_key_ids":null,"io_logging_sampling_rate":1.0,"is_observability_io_logging_enabled":false,"is_observability_broadcast_enabled":true,"is_data_discount_logging_enabled":false,"created_at":"2026-03-01T00:00:00.000Z","updated_at":"2026-03-02T00:00:00.000Z"}}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("workspaces")
        .arg("update")
        .arg("ws_1")
        .arg("--clear-io-logging-api-key-ids");
    cmd.assert().success();

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    let body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(body.get("io_logging_api_key_ids"), Some(&Value::Null));

    server.join().expect("server thread should finish");
}

#[test]
fn test_workspaces_delete_happy_path() {
    let (base_url, rx, server) = spawn_json_server(r#"{"deleted":true}"#);

    let mut cmd = base_cmd(&base_url);
    cmd.arg("workspaces").arg("delete").arg("ws_1").arg("--yes");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");
    assert_eq!(
        parsed.pointer("/data/deleted").and_then(Value::as_bool),
        Some(true)
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "DELETE /api/v1/workspaces/ws_1 HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_workspace_members_add_happy_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"added_count":2,"data":[{"id":"wsm_1","workspace_id":"ws_1","user_id":"user_1","role":"member","created_at":"2026-03-01T00:00:00.000Z"},{"id":"wsm_2","workspace_id":"ws_1","user_id":"user_2","role":"member","created_at":"2026-03-01T00:00:00.000Z"}]}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("workspaces")
        .arg("members")
        .arg("add")
        .arg("ws_1")
        .arg("user_1")
        .arg("user_2");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");
    assert_eq!(
        parsed.pointer("/data/added_count").and_then(Value::as_f64),
        Some(2.0)
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/workspaces/ws_1/members/add HTTP/1.1"
    );
    let body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(
        body.get("user_ids").and_then(Value::as_array).map(Vec::len),
        Some(2)
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_workspace_members_remove_happy_path() {
    let (base_url, rx, server) = spawn_json_server(r#"{"removed_count":2}"#);

    let mut cmd = base_cmd(&base_url);
    cmd.arg("workspaces")
        .arg("members")
        .arg("remove")
        .arg("ws_1")
        .arg("user_1")
        .arg("user_2")
        .arg("--yes");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");
    assert_eq!(
        parsed
            .pointer("/data/removed_count")
            .and_then(Value::as_f64),
        Some(2.0)
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/workspaces/ws_1/members/remove HTTP/1.1"
    );
    let body: Value =
        serde_json::from_str(&captured.body_text).expect("request body should be valid json");
    assert_eq!(
        body.get("user_ids").and_then(Value::as_array).map(Vec::len),
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
fn test_organization_members_list_happy_path() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[{"id":"mem_1","first_name":"Ada","last_name":"Lovelace","email":"ada@example.com","role":"owner"}],"total_count":1}"#,
    );

    let mut cmd = base_cmd(&base_url);
    cmd.arg("organization")
        .arg("members")
        .arg("list")
        .arg("--offset")
        .arg("2")
        .arg("--limit")
        .arg("4");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");
    assert_eq!(
        parsed
            .get("data")
            .and_then(|value| value.get("total_count"))
            .and_then(Value::as_f64),
        Some(1.0)
    );
    assert_eq!(
        parsed.pointer("/data/data/0/email").and_then(Value::as_str),
        Some("ada@example.com")
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/organization/members?offset=2&limit=4 HTTP/1.1"
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
    assert_eq!(body.get("allowed_providers"), Some(&Value::Null));
    assert_eq!(body.get("allowed_models"), Some(&Value::Null));

    server.join().expect("server thread should finish");
}

#[test]
fn test_keys_delete_requires_yes() {
    let mut cmd = base_cmd("http://127.0.0.1:9/api/v1");
    cmd.arg("keys").arg("delete").arg("key_hash_1");
    cmd.assert().failure().stderr(contains("without --yes"));
}

#[test]
fn test_workspaces_delete_requires_yes() {
    let mut cmd = base_cmd("http://127.0.0.1:9/api/v1");
    cmd.arg("workspaces").arg("delete").arg("ws_1");
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

#[test]
fn test_workspace_members_remove_requires_yes() {
    let mut cmd = base_cmd("http://127.0.0.1:9/api/v1");
    cmd.arg("workspaces")
        .arg("members")
        .arg("remove")
        .arg("ws_1")
        .arg("user_1");
    cmd.assert().failure().stderr(contains("without --yes"));
}
