use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use serde_json::{Value, json};

struct CapturedRequest {
    request_line: String,
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
        loop {
            let read = stream.read(&mut chunk).expect("server should read request");
            if read == 0 {
                break;
            }
            request_bytes.extend_from_slice(&chunk[..read]);
            if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let request_line = String::from_utf8_lossy(&request_bytes)
            .lines()
            .next()
            .unwrap_or_default()
            .to_string();
        tx.send(CapturedRequest { request_line })
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
    cmd.arg("--api-key")
        .arg("api-test-key")
        .arg("--base-url")
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

fn sample_models_response() -> &'static str {
    r#"{
      "data": [
        {
          "id": "openai/gpt-4.1",
          "name": "GPT-4.1",
          "created": 1710000000,
          "description": "Test model",
          "context_length": 128000,
          "architecture": {
            "modality": "text->text",
            "tokenizer": "GPT",
            "instruct_type": "chatml"
          },
          "top_provider": {
            "context_length": 128000,
            "max_completion_tokens": 16384,
            "is_moderated": true
          },
          "pricing": {
            "prompt": "0.000002",
            "completion": "0.000008",
            "image": null,
            "request": null,
            "input_cache_read": null,
            "input_cache_write": null,
            "web_search": null,
            "internal_reasoning": null
          },
          "per_request_limits": null
        }
      ]
    }"#
}

#[test]
fn test_models_list_json_snapshot() {
    let (base_url, rx, server) = spawn_json_server(sample_models_response());

    let mut cmd = base_cmd(&base_url, "json");
    cmd.arg("models")
        .arg("list")
        .arg("--category")
        .arg("programming");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");

    let expected = json!([{
        "id": "openai/gpt-4.1",
        "name": "GPT-4.1",
        "created": 1710000000.0,
        "description": "Test model",
        "context_length": 128000.0,
        "architecture": {
            "modality": "text->text",
            "tokenizer": "GPT",
            "instruct_type": "chatml"
        },
        "top_provider": {
            "context_length": 128000.0,
            "max_completion_tokens": 16384.0,
            "is_moderated": true
        },
        "pricing": {
            "prompt": "0.000002",
            "completion": "0.000008",
            "image": null,
            "request": null,
            "input_cache_read": null,
            "input_cache_write": null,
            "web_search": null,
            "internal_reasoning": null
        },
        "per_request_limits": null
    }]);
    assert_eq!(parsed, expected);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/models?category=programming HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_models_list_supported_parameter_filter_path() {
    let (base_url, rx, server) = spawn_json_server(sample_models_response());

    let mut cmd = base_cmd(&base_url, "json");
    cmd.arg("models")
        .arg("list")
        .arg("--supported-parameter")
        .arg("tools");
    cmd.assert().success();

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/models?supported_parameters=tools HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_models_show_json_snapshot() {
    let (base_url, rx, server) = spawn_json_server(sample_models_response());

    let mut cmd = base_cmd(&base_url, "json");
    cmd.arg("models").arg("show").arg("openai/gpt-4.1");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");

    assert_eq!(
        parsed.get("id").and_then(Value::as_str),
        Some("openai/gpt-4.1")
    );
    assert_eq!(parsed.get("name").and_then(Value::as_str), Some("GPT-4.1"));

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(captured.request_line, "GET /api/v1/models HTTP/1.1");

    server.join().expect("server thread should finish");
}

#[test]
fn test_models_endpoints_json_snapshot() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{
          "data": {
            "id": "openai/gpt-4.1",
            "name": "GPT-4.1",
            "created": 1710000000,
            "description": "Test model",
            "architecture": {
              "tokenizer": "GPT",
              "instruct_type": "chatml",
              "modality": "text->text"
            },
            "endpoints": [
              {
                "name": "OpenAI: GPT-4.1",
                "context_length": 128000,
                "pricing": {
                  "request": "0",
                  "image": "0",
                  "prompt": "0.000002",
                  "completion": "0.000008"
                },
                "provider_name": "OpenAI",
                "supported_parameters": ["tools", "temperature"],
                "quantization": null,
                "max_completion_tokens": 16384,
                "max_prompt_tokens": 128000,
                "status": {"state":"up"}
              }
            ]
          }
        }"#,
    );

    let mut cmd = base_cmd(&base_url, "json");
    cmd.arg("models").arg("endpoints").arg("openai/gpt-4.1");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");

    assert_eq!(
        parsed
            .get("endpoints")
            .and_then(Value::as_array)
            .and_then(|values| values.first())
            .and_then(|item| item.get("provider_name"))
            .and_then(Value::as_str),
        Some("OpenAI")
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/models/openai/gpt-4.1/endpoints HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[test]
fn test_providers_list_json_snapshot() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{
          "data": [
            {
              "name": "OpenAI",
              "slug": "openai",
              "privacy_policy_url": "https://openai.com/privacy",
              "terms_of_service_url": "https://openai.com/terms",
              "status_page_url": "https://status.openai.com"
            }
          ]
        }"#,
    );

    let mut cmd = base_cmd(&base_url, "json");
    cmd.arg("providers").arg("list");
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: Value = serde_json::from_slice(&output).expect("stdout should be json");
    assert_eq!(
        parsed.get(0).and_then(|item| item.get("slug")),
        Some(&json!("openai"))
    );

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("request should be captured");
    assert_eq!(captured.request_line, "GET /api/v1/providers HTTP/1.1");

    server.join().expect("server thread should finish");
}

#[test]
fn test_models_list_text_table_output() {
    let (base_url, _rx, server) = spawn_json_server(sample_models_response());

    let mut cmd = base_cmd(&base_url, "text");
    cmd.arg("models").arg("list");
    cmd.assert()
        .success()
        .stdout(contains("id"))
        .stdout(contains("prompt_price"))
        .stdout(contains("openai/gpt-4.1"));

    server.join().expect("server thread should finish");
}

#[test]
fn test_models_show_missing_model_exits_nonzero() {
    let (base_url, _rx, server) = spawn_json_server(r#"{"data":[]}"#);

    let mut cmd = base_cmd(&base_url, "json");
    cmd.arg("models").arg("show").arg("missing/model");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(contains("model not found"));

    server.join().expect("server thread should finish");
}
