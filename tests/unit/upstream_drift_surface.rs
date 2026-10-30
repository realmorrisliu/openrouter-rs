//! Surface added by the 2026-10-30 OpenAPI drift: private providers,
//! external users, guardrail harness flag, prompt-cache breakpoint TTL, user
//! model client filter, and vault routing metadata.

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
};

use openrouter_rs::{
    api::{
        chat::{PromptCacheBreakpoint, PromptCacheMode},
        discovery::ListUserModelsParams,
        external_users::{
            CreateExternalUserRequest, ListExternalUsersParams, UpdateExternalUserRequest,
        },
        guardrails::{CreateGuardrailRequest, UpdateGuardrailRequest},
        private_providers::{
            CreatePrivateProviderRequest, PrivateProviderDataPolicy, UpdatePrivateProviderRequest,
        },
        vault::VaultInternConnectionListResponse,
    },
    types::ApiResponse,
};
use serde_json::json;

use crate::test_client;

struct CapturedRequest {
    line: String,
    headers: String,
    body: String,
}

fn spawn_server_sequence(
    responses: Vec<(u16, String)>,
) -> (
    String,
    mpsc::Receiver<CapturedRequest>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener binds");
    let addr = listener.local_addr().expect("local address");
    let (tx, rx) = mpsc::channel();
    let task = thread::spawn(move || {
        for (status, response_body) in responses {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut bytes = Vec::new();
            let mut chunk = [0_u8; 2048];
            let header_end = loop {
                let count = stream.read(&mut chunk).expect("read request");
                if count == 0 {
                    break None;
                }
                bytes.extend_from_slice(&chunk[..count]);
                if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                    break Some(end + 4);
                }
            }
            .expect("request has headers");
            let head = String::from_utf8_lossy(&bytes[..header_end]).to_string();
            let mut body = String::from_utf8_lossy(&bytes[header_end..]).to_string();
            let length = head
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())?
                })
                .unwrap_or(0);
            while body.len() < length {
                let count = stream.read(&mut chunk).expect("read body");
                if count == 0 {
                    break;
                }
                body.push_str(&String::from_utf8_lossy(&chunk[..count]));
            }
            tx.send(CapturedRequest {
                line: head.lines().next().unwrap_or_default().to_string(),
                headers: head,
                body,
            })
            .expect("send captured request");
            let body_bytes = response_body.as_bytes();
            let response = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body_bytes.len()
            );
            stream.write_all(response.as_bytes()).expect("write head");
            stream.write_all(body_bytes).expect("write body");
        }
    });
    (format!("http://{addr}/api/v1"), rx, task)
}

const PROVIDER: &str = r#"{"data":{"name":"acme","slug":"acme","display_name":"Acme","base_url":"https://llm.acme.test","data_policy":{"training":false,"retains_prompts":true,"prompt_retention_days":30},"created_at":"2026-10-30T00:00:00Z"}}"#;

#[tokio::test]
async fn private_providers_cover_every_documented_route() {
    let (base_url, rx, server) = spawn_server_sequence(vec![
        (200, r#"{"data":[]}"#.to_string()),
        (201, PROVIDER.to_string()),
        (200, PROVIDER.to_string()),
        (200, PROVIDER.to_string()),
        (200, r#"{"data":{"slug":"acme"}}"#.to_string()),
    ]);
    let client = test_client(&base_url, "management-key");

    assert!(
        client
            .management()
            .list_private_providers()
            .await
            .unwrap()
            .is_empty()
    );

    let created = client
        .management()
        .create_private_provider(
            &CreatePrivateProviderRequest::builder()
                .name("acme")
                .slug("acme")
                .display_name("Acme")
                .base_url("https://llm.acme.test")
                .data_policy(
                    PrivateProviderDataPolicy::builder()
                        .training(false)
                        .retains_prompts(true)
                        .prompt_retention_days(30)
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.slug, "acme");

    client
        .management()
        .get_private_provider("acme")
        .await
        .unwrap();
    client
        .management()
        .update_private_provider(
            "acme",
            &UpdatePrivateProviderRequest::builder()
                .display_name("Acme LLM")
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    let deleted = client
        .management()
        .delete_private_provider("acme")
        .await
        .unwrap();
    assert_eq!(deleted.slug, "acme");

    let requests: Vec<_> = (0..5).map(|_| rx.recv().unwrap()).collect();
    assert_eq!(
        requests.iter().map(|r| r.line.as_str()).collect::<Vec<_>>(),
        [
            "GET /api/v1/private-providers HTTP/1.1",
            "POST /api/v1/private-providers HTTP/1.1",
            "GET /api/v1/private-providers/acme HTTP/1.1",
            "PATCH /api/v1/private-providers/acme HTTP/1.1",
            "DELETE /api/v1/private-providers/acme HTTP/1.1",
        ]
    );
    assert!(requests.iter().all(|request| {
        request
            .headers
            .to_ascii_lowercase()
            .contains("authorization: bearer management-key")
    }));
    let create_body: serde_json::Value = serde_json::from_str(&requests[1].body).unwrap();
    assert_eq!(create_body["name"], "acme");
    assert_eq!(
        create_body["data_policy"],
        json!({"training": false, "retains_prompts": true, "prompt_retention_days": 30})
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[3].body).unwrap(),
        json!({"display_name": "Acme LLM"})
    );
    server.join().expect("finish server");
}

#[tokio::test]
async fn external_users_use_the_renamed_route() {
    let user = r#"{"data":{"user":"employee_123","is_active":true,"created_at":"2026-10-30T00:00:00Z","updated_at":"2026-10-30T00:00:00Z"}}"#;
    let (base_url, rx, server) = spawn_server_sequence(vec![
        (200, r#"{"data":[],"total_count":0}"#.to_string()),
        (200, user.to_string()),
        (200, user.to_string()),
        (200, user.to_string()),
        (204, String::new()),
    ]);
    let client = test_client(&base_url, "management-key");

    client
        .management()
        .list_external_users(
            &ListExternalUsersParams::builder()
                .include_inactive(true)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    client
        .management()
        .create_external_user(
            &CreateExternalUserRequest::builder()
                .user("employee_123")
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    client
        .management()
        .get_external_user("employee_123")
        .await
        .unwrap();
    client
        .management()
        .update_external_user(
            "employee_123",
            &UpdateExternalUserRequest::builder()
                .is_active(false)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    client
        .management()
        .delete_external_user("employee_123")
        .await
        .unwrap();

    let requests: Vec<_> = (0..5).map(|_| rx.recv().unwrap()).collect();
    assert_eq!(
        requests.iter().map(|r| r.line.as_str()).collect::<Vec<_>>(),
        [
            "GET /api/v1/external-users?include_inactive=true HTTP/1.1",
            "POST /api/v1/external-users HTTP/1.1",
            "GET /api/v1/external-users/employee_123 HTTP/1.1",
            "PATCH /api/v1/external-users/employee_123 HTTP/1.1",
            "DELETE /api/v1/external-users/employee_123 HTTP/1.1",
        ]
    );
    server.join().expect("finish server");
}

#[test]
fn guardrail_harness_flag_round_trips() {
    let created = CreateGuardrailRequest::builder()
        .name("harness")
        .enforce_first_party_harness_models(true)
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(&created).unwrap()["enforce_first_party_harness_models"],
        true
    );

    let updated = UpdateGuardrailRequest::builder()
        .enforce_first_party_harness_models(false)
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(&updated).unwrap()["enforce_first_party_harness_models"],
        false
    );
}

#[test]
fn prompt_cache_breakpoint_carries_a_ttl() {
    let breakpoint = PromptCacheBreakpoint::explicit_with_ttl("1h");
    assert_eq!(
        serde_json::to_value(&breakpoint).unwrap(),
        json!({"mode": "explicit", "ttl": "1h"})
    );
    assert_eq!(
        serde_json::to_value(PromptCacheBreakpoint::explicit()).unwrap(),
        json!({"mode": "explicit"})
    );

    let restored: PromptCacheBreakpoint =
        serde_json::from_value(json!({"mode": "explicit", "ttl": "5m"})).unwrap();
    assert_eq!(restored.mode, PromptCacheMode::Explicit);
    assert_eq!(restored.ttl.as_deref(), Some("5m"));
}

#[tokio::test]
async fn user_models_filter_by_client() {
    let (base_url, rx, server) = spawn_server_sequence(vec![(200, r#"{"data":[]}"#.to_string())]);
    let client = test_client(&base_url, "api-key");

    client
        .models()
        .list_for_user_with_params(
            &ListUserModelsParams::builder()
                .client("codex")
                .limit(10)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    let captured = rx.recv().expect("capture request");
    assert_eq!(
        captured.line,
        "GET /api/v1/models/user?limit=10&client=codex HTTP/1.1"
    );
    server.join().expect("finish server");
}

#[test]
fn vault_routing_metadata_is_kept() {
    let payload: ApiResponse<VaultInternConnectionListResponse> = serde_json::from_value(json!({
        "data": {
            "data": [{"key": "GITHUB_TOKEN"}],
            "routing": {"agent_id": "agent-1", "custom_origins": []},
            "routing_text": "GITHUB_TOKEN -> github.com"
        }
    }))
    .unwrap();

    assert_eq!(payload.data.data.len(), 1);
    assert_eq!(
        payload.data.routing.as_ref().unwrap()["agent_id"],
        "agent-1"
    );
    assert_eq!(
        payload.data.routing_text.as_deref(),
        Some("GITHUB_TOKEN -> github.com")
    );

    // A response without the routing fields still decodes.
    let bare: ApiResponse<VaultInternConnectionListResponse> =
        serde_json::from_value(json!({"data": {"data": [{"key": "TOKEN"}]}})).unwrap();
    assert!(bare.data.routing.is_none());
    assert!(bare.data.routing_text.is_none());
}
