//! Guardrail policies (`/guardrails/policies*`) and agent credit purchases.

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::{
        credits::{CreateCreditPurchaseRequest, CreditPurchase},
        guardrail_policies::{
            AccountGuardrailPolicy, AccountGuardrailPolicyRequest,
            ChangeGuardrailPolicyAssignmentRequest, CreateGuardrailBudgetPolicyRequest,
            CreateGuardrailModerationPolicyRequest, GuardrailAccountPolicy,
            GuardrailAccountPolicyRequest, GuardrailBudgetPolicyAuthoringConfig,
            GuardrailBudgetPolicyConfig, GuardrailBudgetPolicyDetail, GuardrailBudgetPolicyList,
            GuardrailBudgetResetInterval, GuardrailModelAvailabilityPolicyConfig,
            GuardrailModerationPolicyConfig, GuardrailPolicyAssignmentState,
            GuardrailPolicyAssignments, GuardrailPolicyType, UpdateGuardrailBudgetPolicyRequest,
        },
        guardrails::{
            ContentFilterAction, ContentFilterBuiltinAction, ContentFilterBuiltinEntry,
            ContentFilterBuiltinSlug, ContentFilterEntry,
        },
        images::ImageGenerationRequest,
    },
    types::PaginationOptions,
};
use serde_json::json;

use crate::test_client;

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
        .expect("request should contain header terminator");

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
        let request_text = format!("{header_text}{body_text}");
        tx.send(CapturedRequest {
            request_line,
            request_text,
            body_text,
        })
        .expect("server should send captured request");

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

fn budget_policy_config() -> GuardrailBudgetPolicyAuthoringConfig {
    GuardrailBudgetPolicyAuthoringConfig::new(
        Some(200.0),
        Some(GuardrailBudgetResetInterval::Monthly),
        false,
    )
}

#[test]
fn create_budget_policy_serializes_policy_type_config_and_assignments() {
    let request = CreateGuardrailBudgetPolicyRequest::builder()
        .name("Engineering monthly budget")
        .config(budget_policy_config())
        .workspace_id("ws_123")
        .assignments(
            GuardrailPolicyAssignments::builder()
                .member_ids(vec!["user_1".to_string()])
                .build()
                .unwrap(),
        )
        .build()
        .expect("create budget policy request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value["name"], "Engineering monthly budget");
    assert_eq!(value["policy_type"], "budget");
    assert_eq!(value["workspace_id"], "ws_123");
    assert_eq!(value["config"]["limit_usd"], 200.0);
    assert_eq!(value["config"]["reset_interval"], "monthly");
    assert_eq!(value["config"]["include_byok_in_budgets"], false);
    assert_eq!(value["member_ids"], json!(["user_1"]));
    // Unset assignment fields stay off the wire.
    assert!(value.get("api_key_hashes").is_none());
    assert!(value.get("is_workspace_default").is_none());
}

#[test]
fn budget_authoring_config_sends_null_for_unset_limit_and_interval() {
    let request = UpdateGuardrailBudgetPolicyRequest::builder()
        .name("Unlimited")
        .config(GuardrailBudgetPolicyAuthoringConfig::new(None, None, true))
        .workspace_id("ws_123")
        .build()
        .expect("update budget policy request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value["config"]["limit_usd"], serde_json::Value::Null);
    assert_eq!(value["config"]["reset_interval"], serde_json::Value::Null);
    assert_eq!(value["config"]["include_byok_in_budgets"], true);
    // Update bodies carry no policy_type.
    assert!(value.get("policy_type").is_none());
}

#[test]
fn moderation_policy_config_reuses_guardrail_content_filter_types() {
    let config = GuardrailModerationPolicyConfig::builder()
        .content_filter_builtins(vec![ContentFilterBuiltinEntry::new(
            ContentFilterBuiltinSlug::Secrets,
            ContentFilterBuiltinAction::Block,
        )])
        .content_filters(vec![ContentFilterEntry::new(
            "secret",
            ContentFilterAction::Flag,
        )])
        .content_filter_blocks_requests(true)
        .build()
        .expect("moderation config should build");

    let request = CreateGuardrailModerationPolicyRequest::builder()
        .name("Block secrets")
        .config(config)
        .workspace_id("ws_123")
        .build()
        .expect("create moderation policy request should build");

    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value["policy_type"], "moderation");
    assert_eq!(
        value["config"]["content_filter_builtins"][0]["slug"],
        "secrets"
    );
    assert_eq!(
        value["config"]["content_filter_builtins"][0]["action"],
        "block"
    );
    assert_eq!(value["config"]["content_filters"][0]["pattern"], "secret");
    assert_eq!(value["config"]["content_filter_blocks_requests"], true);
    assert!(value["config"].get("armor").is_none());
}

#[test]
fn model_availability_config_omits_unset_switches() {
    let config = GuardrailModelAvailabilityPolicyConfig::builder()
        .allowed_models(vec!["openai/gpt-4.1".to_string()])
        .enforce_zdr(true)
        .build()
        .expect("model availability config should build");

    let value = serde_json::to_value(&config).expect("config should serialize");
    assert_eq!(value["allowed_models"], json!(["openai/gpt-4.1"]));
    assert_eq!(value["enforce_zdr"], true);
    assert!(value.get("allowed_providers").is_none());
    assert!(value.get("platform_banned_models").is_none());
}

#[test]
fn account_policy_requests_are_tagged_by_policy_type() {
    let budget = AccountGuardrailPolicyRequest::Budget(
        GuardrailAccountPolicyRequest::builder()
            .name("Account budget")
            .config(budget_policy_config())
            .is_default(true)
            .opted_out_workspace_ids(vec!["ws_9".to_string()])
            .build()
            .expect("account budget request should build"),
    );
    let value = serde_json::to_value(&budget).expect("request should serialize");
    assert_eq!(value["policy_type"], "budget");
    assert_eq!(value["name"], "Account budget");
    assert_eq!(value["is_default"], true);
    assert_eq!(value["opted_out_workspace_ids"], json!(["ws_9"]));

    let moderation = AccountGuardrailPolicyRequest::Moderation(
        GuardrailAccountPolicyRequest::builder()
            .name("Account moderation")
            .config(GuardrailModerationPolicyConfig::default())
            .build()
            .expect("account moderation request should build"),
    );
    let value = serde_json::to_value(&moderation).expect("request should serialize");
    assert_eq!(value["policy_type"], "moderation");
}

#[test]
fn account_policy_responses_dispatch_on_policy_type() {
    let budget_raw = json!({
        "policy_type": "budget",
        "data": {
            "id": "pol_1",
            "name": "Account budget",
            "policy_type": "budget",
            "config": {"limit_usd": 200, "reset_interval": "monthly", "include_byok_in_budgets": false},
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-02T00:00:00Z",
            "is_default": true,
            "workspace_count": 2,
            "workspace_ids": ["ws_1", "ws_2"],
            "opted_out_workspace_ids": [],
            "is_synced": false,
            "is_workspace_default_synced": false
        }
    });
    let budget: GuardrailAccountPolicy<GuardrailBudgetPolicyConfig> =
        serde_json::from_value(budget_raw["data"].clone())
            .expect("account budget should deserialize");
    assert_eq!(budget.workspace_count, 2);
    assert_eq!(budget.config.limit_usd, Some(200.0));

    let policies: Vec<AccountGuardrailPolicy> = serde_json::from_value(json!([
        {
            "id": "pol_1",
            "name": "Account budget",
            "policy_type": "budget",
            "config": {"limit_usd": 200},
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-02T00:00:00Z",
            "is_default": false,
            "workspace_count": 0,
            "workspace_ids": [],
            "opted_out_workspace_ids": [],
            "is_synced": false,
            "is_workspace_default_synced": false
        },
        {
            "id": "pol_2",
            "name": "Approved models",
            "policy_type": "model_availability",
            "config": {"allowed_models": ["openai/gpt-4.1"]},
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-02T00:00:00Z",
            "is_default": true,
            "workspace_count": 1,
            "workspace_ids": ["ws_1"],
            "opted_out_workspace_ids": [],
            "is_synced": false,
            "is_workspace_default_synced": false
        }
    ]))
    .expect("account policies should deserialize");
    assert!(matches!(policies[0], AccountGuardrailPolicy::Budget(_)));
    match &policies[1] {
        AccountGuardrailPolicy::ModelAvailability(policy) => assert_eq!(
            policy.config.allowed_models.as_deref(),
            Some(&["openai/gpt-4.1".to_string()][..])
        ),
        other => panic!("expected model availability policy, got {other:?}"),
    }
}

#[test]
fn workspace_policy_list_and_detail_deserialize() {
    let list: GuardrailBudgetPolicyList = serde_json::from_value(json!({
        "data": [{
            "id": "pol_1",
            "name": "Engineering monthly budget",
            "policy_type": "budget",
            "config": {"limit_usd": 200, "reset_interval": "monthly", "include_byok_in_budgets": true},
            "updated_at": "2025-01-02T00:00:00Z",
            "member_count": 3,
            "api_key_count": 2,
            "is_workspace_default": true
        }],
        "total_count": 1
    }))
    .expect("policy list should deserialize");
    assert_eq!(list.total_count, 1);
    assert_eq!(list.data[0].member_count, 3);
    assert_eq!(list.data[0].is_workspace_default, Some(true));

    let detail: GuardrailBudgetPolicyDetail = serde_json::from_value(json!({
        "id": "pol_1",
        "name": "Engineering monthly budget",
        "policy_type": "budget",
        "config": {"limit_usd": 200},
        "updated_at": "2025-01-02T00:00:00Z",
        "member_ids": ["user_1"],
        "api_key_hashes": ["hash_1"],
        "opted_out_api_key_hashes": [],
        "overridden_api_key_hashes": ["hash_2"]
    }))
    .expect("policy detail should deserialize");
    assert_eq!(detail.member_ids, ["user_1"]);
    assert_eq!(detail.overridden_api_key_hashes, ["hash_2"]);
}

#[tokio::test]
async fn list_budget_policies_sends_workspace_and_pagination() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[],"total_count":0}"#);

    let result = test_client(&base_url, "mgmt-key")
        .management()
        .list_budget_policies("ws_123", Some(PaginationOptions::new(Some(5), Some(10))))
        .await
        .expect("list budget policies should succeed");
    assert_eq!(result.total_count, 0);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/guardrails/policies/budget?workspace_id=ws_123&offset=5&limit=10 HTTP/1.1"
    );
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer mgmt-key")
            || request_lower.contains("authorization:bearer mgmt-key"),
        "management key should be sent, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn get_and_delete_moderation_policy_send_workspace_query() {
    let detail = json!({
        "data": {
            "id": "pol 1",
            "name": "Block secrets",
            "policy_type": "moderation",
            "config": {"content_filter_blocks_requests": true},
            "updated_at": "2025-01-02T00:00:00Z",
            "member_ids": [],
            "api_key_hashes": [],
            "opted_out_api_key_hashes": [],
            "overridden_api_key_hashes": []
        }
    })
    .to_string();
    let (base_url, rx, server) = spawn_json_server(&detail);
    let policy = test_client(&base_url, "mgmt-key")
        .management()
        .get_moderation_policy("pol 1", "ws_123")
        .await
        .expect("get moderation policy should succeed");
    assert_eq!(policy.config.content_filter_blocks_requests, Some(true));
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/guardrails/policies/moderation/pol%201?workspace_id=ws_123 HTTP/1.1"
    );
    server.join().expect("server thread should finish");

    let (base_url, rx, server) = spawn_json_server(r#"{"data":true}"#);
    let deleted = test_client(&base_url, "mgmt-key")
        .management()
        .delete_moderation_policy("pol_1", "ws_123")
        .await
        .expect("delete moderation policy should succeed");
    assert!(deleted);
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "DELETE /api/v1/guardrails/policies/moderation/pol_1?workspace_id=ws_123 HTTP/1.1"
    );
    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn change_member_policy_assignment_puts_body_and_unwraps_data() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":true}"#);
    let request = ChangeGuardrailPolicyAssignmentRequest::builder()
        .workspace_id("ws_123")
        .policy_type(GuardrailPolicyType::Budget)
        .policy_id("pol_1")
        .assigned(true)
        .expected_assignment(
            GuardrailPolicyAssignmentState::builder()
                .policy_ids(vec!["pol_1".to_string()])
                .build()
                .expect("expected assignment should build"),
        )
        .build()
        .expect("assignment request should build");

    let changed = test_client(&base_url, "mgmt-key")
        .management()
        .change_member_policy_assignment("user_1", &request)
        .await
        .expect("change assignment should succeed");
    assert!(changed);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "PUT /api/v1/guardrails/policies/assignments/members/user_1 HTTP/1.1"
    );
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("body should be json");
    assert_eq!(body["workspace_id"], "ws_123");
    assert_eq!(body["policy_type"], "budget");
    assert_eq!(body["policy_id"], "pol_1");
    assert_eq!(body["assigned"], true);
    assert_eq!(body["expected_assignment"]["policy_ids"], json!(["pol_1"]));

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn api_key_policy_assignment_sends_policy_type_query() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"policy_ids":[],"workspace_default_policy_ids":["pol_9"],"opted_out":true}}"#,
    );

    let state = test_client(&base_url, "mgmt-key")
        .management()
        .get_api_key_policy_assignment("hash_1", "ws_123", GuardrailPolicyType::RateLimit)
        .await
        .expect("read assignment should succeed");
    assert!(state.opted_out);
    assert_eq!(state.workspace_default_policy_ids, ["pol_9"]);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/guardrails/policies/assignments/api-keys/hash_1?workspace_id=ws_123&policy_type=rate_limit HTTP/1.1"
    );
    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn account_policy_endpoints_use_expected_paths() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[]}"#);
    let policies = test_client(&base_url, "mgmt-key")
        .management()
        .list_account_guardrail_policies(None)
        .await
        .expect("list account policies should succeed");
    assert!(policies.is_empty());
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/guardrails/policies/account HTTP/1.1"
    );
    server.join().expect("server thread should finish");

    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"id":"pol_1","name":"Account budget","policy_type":"budget","config":{},"created_at":"2025-01-01T00:00:00Z","updated_at":"2025-01-02T00:00:00Z","is_default":false,"workspace_count":0,"workspace_ids":[],"opted_out_workspace_ids":[],"is_synced":false,"is_workspace_default_synced":false}}"#,
    );
    let request = AccountGuardrailPolicyRequest::Budget(
        GuardrailAccountPolicyRequest::builder()
            .name("Account budget")
            .config(budget_policy_config())
            .build()
            .expect("account policy request should build"),
    );
    let created = test_client(&base_url, "mgmt-key")
        .management()
        .create_account_guardrail_policy(&request)
        .await
        .expect("create account policy should succeed");
    assert!(matches!(created, AccountGuardrailPolicy::Budget(_)));
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/guardrails/policies/account HTTP/1.1"
    );
    assert!(captured.body_text.contains("\"policy_type\":\"budget\""));
    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn credit_purchase_round_trip_uses_api_key() {
    let purchase = json!({
        "data": {
            "id": "purchase_1",
            "status": "pending",
            "credit_amount_usd": "10.00",
            "platform_fee_usd": "0.30",
            "total_charged_usd": "10.30",
            "protocol": "mpp",
            "network": "tempo",
            "payment_intent_id": null,
            "failure_reason": null,
            "created_at": "2025-01-01T00:00:00Z"
        }
    })
    .to_string();
    let (base_url, rx, server) = spawn_json_server(&purchase);
    let created = test_client(&base_url, "api-key")
        .management()
        .create_credit_purchase(&CreateCreditPurchaseRequest::new("10.00"))
        .await
        .expect("create credit purchase should succeed");
    assert_eq!(created.status, "pending");
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/credits/purchases HTTP/1.1"
    );
    let body: serde_json::Value = serde_json::from_str(&captured.body_text).expect("json body");
    assert_eq!(body["credit_amount_usd"], "10.00");
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "workspace api key should be sent"
    );
    server.join().expect("server thread should finish");

    let (base_url, rx, server) = spawn_json_server(&purchase);
    let polled: CreditPurchase = test_client(&base_url, "api-key")
        .management()
        .get_credit_purchase("purchase_1")
        .await
        .expect("poll credit purchase should succeed");
    assert_eq!(polled.credit_amount_usd, "10.00");
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/credits/purchases/purchase_1 HTTP/1.1"
    );
    server.join().expect("server thread should finish");
}

#[test]
fn models_list_params_expose_endpoints_filter() {
    let params = openrouter_rs::api::models::ListModelsParams::builder()
        .endpoints("/api/v1/responses")
        .build()
        .expect("params should build");
    let value = serde_json::to_value(&params).expect("params should serialize");
    assert_eq!(value["endpoints"], "/api/v1/responses");
}

#[test]
fn image_requests_expose_frame_and_markup_fields() {
    let request = ImageGenerationRequest::builder()
        .model("openai/gpt-image-1")
        .prompt("a walking cat")
        .output_format("gif")
        .frame_count(8)
        .frame_duration_ms(100)
        .markup(
            json!({"type": "image_url", "image_url": {"url": "https://example.com/overlay.png"}}),
        )
        .build()
        .expect("image request should build");
    let value = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(value["output_format"], "gif");
    assert_eq!(value["frame_count"], 8);
    assert_eq!(value["frame_duration_ms"], 100);
    assert_eq!(
        value["markup"]["image_url"]["url"],
        "https://example.com/overlay.png"
    );
}
