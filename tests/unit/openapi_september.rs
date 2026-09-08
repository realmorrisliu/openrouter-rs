use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    OpenRouterClient,
    api::{
        api_keys::*, auth::*, byok::*, discovery::ListUserModelsParams, guardrails::*, messages::*,
        observability::*,
    },
};
use serde_json::{Value, json};

struct CapturedRequest {
    request_line: String,
    request_text: String,
    body_text: String,
}

fn spawn_server(
    response_body: &[u8],
    content_type: &str,
) -> (
    String,
    mpsc::Receiver<CapturedRequest>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let body = response_body.to_vec();
    let content_type = content_type.to_string();
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
        .expect("server should send request");

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            content_type,
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write header");
        stream
            .write_all(&body)
            .expect("server should write response body");
    });

    (format!("http://{addr}/api/v1"), rx, server)
}

fn client(base: &str) -> OpenRouterClient {
    OpenRouterClient::builder()
        .base_url(base)
        .api_key("test-api")
        .management_key("test-management")
        .build()
        .unwrap()
}

#[test]
fn nullable_management_lists_omit_set_clear_and_reset_independently() {
    let mut byok = UpdateByokKeyRequest::builder();
    assert_eq!(
        serde_json::to_value(byok.build().unwrap()).unwrap(),
        json!({})
    );
    byok.allowed_models(["model"])
        .allowed_api_key_hashes(["hash"]);
    assert_eq!(
        serde_json::to_value(byok.build().unwrap()).unwrap(),
        json!({"allowed_models":["model"],"allowed_api_key_hashes":["hash"]})
    );
    byok.clear_allowed_api_key_hashes();
    assert_eq!(
        serde_json::to_value(byok.build().unwrap()).unwrap(),
        json!({"allowed_models":["model"],"allowed_api_key_hashes":null})
    );
    byok.allowed_api_key_hashes(["other"]);
    assert_eq!(
        serde_json::to_value(byok.build().unwrap()).unwrap()["allowed_api_key_hashes"],
        json!(["other"])
    );
    let create = CreateByokKeyRequest::builder()
        .provider("test")
        .key("test")
        .allowed_api_key_hashes(["hash"])
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(create).unwrap()["allowed_api_key_hashes"],
        json!(["hash"])
    );

    let mut guardrail = UpdateGuardrailRequest::builder();
    assert_eq!(
        serde_json::to_value(guardrail.build().unwrap()).unwrap(),
        json!({})
    );
    guardrail
        .allowed_models(["model"])
        .allowed_data_regions(["europe"]);
    assert_eq!(
        serde_json::to_value(guardrail.build().unwrap()).unwrap(),
        json!({"allowed_models":["model"],"allowed_data_regions":["europe"]})
    );
    guardrail.clear_allowed_data_regions();
    assert_eq!(
        serde_json::to_value(guardrail.build().unwrap()).unwrap(),
        json!({"allowed_models":["model"],"allowed_data_regions":null})
    );
    guardrail.allowed_data_regions(["us"]);
    assert_eq!(
        serde_json::to_value(guardrail.build().unwrap()).unwrap()["allowed_data_regions"],
        json!(["us"])
    );
    let create = CreateGuardrailRequest::builder()
        .name("regions")
        .allowed_data_regions(["us"])
        .content_filter_builtins([ContentFilterBuiltinEntry::new(
            ContentFilterBuiltinSlug::Secrets,
            ContentFilterBuiltinAction::Redact,
        )])
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(create).unwrap()["content_filter_builtins"][0]["slug"],
        "secrets"
    );
}

#[test]
fn observability_regions_are_optional_and_serialize_in_patch() {
    let create = CreateObservabilityDestinationRequest::builder()
        .destination_type("webhook")
        .name("test")
        .config(json!({}))
        .build()
        .unwrap();
    assert!(
        serde_json::to_value(create)
            .unwrap()
            .get("regions")
            .is_none()
    );
    let patch = UpdateObservabilityDestinationRequest::builder()
        .regions(vec!["eu".into()])
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(patch).unwrap(),
        json!({"regions":["eu"]})
    );
}

#[test]
fn messages_preserve_new_controls_and_container_citations() {
    let wire = json!({"role":"assistant","clear_at":"next_user_message","output_config":{"effort":"xhigh"},"content":[{"type":"openrouter_shell_tool_result","tool_use_id":"tool-1","content":{},"container_id":"container-1","files":[{"type":"container_file_citation","file_id":"file-1","container_id":"container-1","filename":"out.txt","start_index":0,"end_index":1}]}]});
    let msg: AnthropicMessage = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(msg).unwrap(), wire);
    for wire in [
        json!({"type":"enabled","budget_tokens":1024,"display":"updates","block_binding":{"prefix_mismatch_behavior":"error"}}),
        json!({"type":"adaptive","display":"updates"}),
    ] {
        let thinking: AnthropicThinking = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(thinking).unwrap(), wire);
    }
    assert_eq!(
        serde_json::to_value(AnthropicThinking::enabled(10)).unwrap(),
        json!({"type":"enabled","budget_tokens":10})
    );
    assert_eq!(
        serde_json::to_value(AnthropicMessage::user("hello")).unwrap(),
        json!({"role":"user","content":"hello"})
    );
}

#[tokio::test]
async fn container_file_domain_routes_auth_pagination_and_binary_content() {
    let file = json!({"id":"file-1","object":"container.file","container_id":"container-1","created_at":1,"bytes":3,"path":"/out.txt","source":"assistant"});
    let list = json!({"object":"list","data":[file.clone()],"first_id":"file-1","last_id":"file-1","has_more":false});
    let (base, rx, server) = spawn_server(list.to_string().as_bytes(), "application/json");
    let result = client(&base)
        .files()
        .list_container_files("container/1", Some("file+0"), Some(2))
        .await
        .unwrap();
    assert_eq!(result.data[0].bytes, 3);
    let req = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(
        req.request_line,
        "GET /api/v1/containers/container%2F1/files?after=file%2B0&limit=2 HTTP/1.1"
    );
    assert!(req.request_text.contains("Bearer test-api"));
    server.join().unwrap();
    let (base, rx, server) = spawn_server(file.to_string().as_bytes(), "application/json");
    assert_eq!(
        client(&base)
            .files()
            .get_container_file("c/1", "f/1")
            .await
            .unwrap()
            .id,
        "file-1"
    );
    assert_eq!(
        rx.recv().unwrap().request_line,
        "GET /api/v1/containers/c%2F1/files/f%2F1 HTTP/1.1"
    );
    server.join().unwrap();
    let (base, rx, server) = spawn_server(&[0, 255, 1], "application/octet-stream");
    assert_eq!(
        client(&base)
            .files()
            .download_container_file("c", "f")
            .await
            .unwrap(),
        vec![0, 255, 1]
    );
    assert_eq!(
        rx.recv().unwrap().request_line,
        "GET /api/v1/containers/c/files/f/content HTTP/1.1"
    );
    server.join().unwrap();
    let promoted = json!({"_shape":"openai","id":"file-promoted","filename":"out.txt","created_at":1,"bytes":3,"object":"file","purpose":"user_data","status":"processed"});
    let (base, rx, server) = spawn_server(promoted.to_string().as_bytes(), "application/json");
    assert_eq!(
        client(&base)
            .files()
            .promote_container_file("c", "f")
            .await
            .unwrap()
            .shape,
        "openai"
    );
    let req = rx.recv().unwrap();
    assert_eq!(
        req.request_line,
        "POST /api/v1/containers/c/files/f/promote HTTP/1.1"
    );
    assert!(req.body_text.is_empty());
    server.join().unwrap();
}

#[tokio::test]
async fn scim_sync_jobs_use_management_auth_and_unwrap_data() {
    let body = json!({"data":{"id":"job","status":"queued","created_at":"now","started_at":null,"finished_at":null,"error_message":null,"synced_groups":null,"deleted_groups":null}});
    for create in [true, false] {
        let (base, rx, server) = spawn_server(body.to_string().as_bytes(), "application/json");
        let c = client(&base);
        let job = if create {
            c.management().create_scim_sync_job().await.unwrap()
        } else {
            c.management().get_scim_sync_job("job/1").await.unwrap()
        };
        assert_eq!(job.status, "queued");
        assert!(job.synced_groups.is_none());
        let req = rx.recv().unwrap();
        assert!(req.request_text.contains("Bearer test-management"));
        assert_eq!(
            req.request_line,
            if create {
                "POST /api/v1/scim/sync-jobs HTTP/1.1"
            } else {
                "GET /api/v1/scim/sync-jobs/job%2F1 HTTP/1.1"
            }
        );
        server.join().unwrap();
    }
}

#[tokio::test]
async fn oauth_uses_form_encoding_and_does_not_require_an_existing_key() {
    let body = json!({"access_token":"token","expires_in":300,"issued_token_type":"urn:ietf:params:oauth:token-type:access_token","scope":"inference","token_type":"Bearer"});
    let (base, rx, server) = spawn_server(body.to_string().as_bytes(), "application/json");
    let c = OpenRouterClient::builder().base_url(&base).build().unwrap();
    let request = OAuthTokenExchangeRequest::builder()
        .federation_policy_id("policy")
        .subject_token("jwt+&=")
        .build()
        .unwrap();
    assert_eq!(
        c.management()
            .exchange_oauth_token(&request)
            .await
            .unwrap()
            .expires_in,
        300
    );
    let req = rx.recv().unwrap();
    assert_eq!(req.request_line, "POST /api/v1/oauth/token HTTP/1.1");
    assert!(
        req.request_text
            .contains("application/x-www-form-urlencoded")
    );
    assert!(req.body_text.contains("subject_token=jwt%2B%26%3D"));
    assert!(
        req.body_text
            .contains("grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange")
    );
    server.join().unwrap();
    let (base, rx, server) = spawn_server(br#"{"keys":[{"alg":"ES256","crv":"P-256","kid":"key","kty":"EC","use":"sig","x":"a","y":"b"}]}"#, "application/json");
    assert_eq!(
        client(&base)
            .management()
            .get_oauth_jwks()
            .await
            .unwrap()
            .keys[0]
            .key_use,
        "sig"
    );
    assert_eq!(
        rx.recv().unwrap().request_line,
        "GET /api/v1/oauth/jwks HTTP/1.1"
    );
    server.join().unwrap();
}

#[tokio::test]
async fn external_key_identity_and_user_model_filters_reach_the_wire() {
    let (base, rx, server) =
        spawn_server(br#"{"data":{"external_user":"alice"}}"#, "application/json");
    let request = CreateApiKeyRequest::builder()
        .name("external")
        .external(ExternalApiKeyIdentity::new("alice"))
        .build()
        .unwrap();
    assert_eq!(
        client(&base)
            .management()
            .create_api_key_with_options(&request)
            .await
            .unwrap()
            .external_user
            .as_deref(),
        Some("alice")
    );
    let req = rx.recv().unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&req.body_text).unwrap(),
        json!({"name":"external","external":{"user":"alice"}})
    );
    server.join().unwrap();
    let (base, rx, server) = spawn_server(br#"{"data":[]}"#, "application/json");
    let params = ListUserModelsParams::builder()
        .limit(2)
        .offset(1)
        .output_modalities("text,image")
        .build()
        .unwrap();
    client(&base)
        .models()
        .list_for_user_with_params(&params)
        .await
        .unwrap();
    assert_eq!(
        rx.recv().unwrap().request_line,
        "GET /api/v1/models/user?limit=2&offset=1&output_modalities=text%2Cimage HTTP/1.1"
    );
    server.join().unwrap();
}

#[test]
fn endpoint_pricing_cost_and_chat_additions_preserve_old_payload_defaults() {
    use openrouter_rs::api::{
        chat::{Message, VideoUrl},
        embeddings::EmbeddingCostDetails,
        models::{Endpoint, PricingOverride},
    };
    use openrouter_rs::types::{ResponseCostDetails, Role};
    let pricing: PricingOverride =
        serde_json::from_value(json!({"utc_days":["monday"],"utc_start":0,"utc_end":12})).unwrap();
    assert_eq!(
        pricing.utc_days.as_deref(),
        Some(["monday".to_string()].as_slice())
    );
    let endpoint: Endpoint = serde_json::from_value(json!({"name":"endpoint","context_length":1024,"pricing":{"prompt":"0","completion":"0"},"provider_name":"provider","supported_parameters":[],"supports_tool_choice":{"auto":true,"none":true,"required":false,"function":true},"perf_last_30m_by_workload":{"embeddings":{"latency":null,"request_count":2}}})).unwrap();
    assert!(endpoint.supports_tool_choice.as_ref().unwrap().function);
    assert_eq!(
        serde_json::to_value(endpoint).unwrap()["perf_last_30m_by_workload"]["embeddings"]["request_count"],
        2
    );
    let cost = json!({"upstream_inference_completions_cost":0.0,"upstream_inference_prompt_cost":0.1,"server_tool_cost":0.2});
    let completion: ResponseCostDetails = serde_json::from_value(cost.clone()).unwrap();
    let embedding: EmbeddingCostDetails = serde_json::from_value(cost).unwrap();
    assert_eq!(completion.server_tool_cost, Some(0.2));
    assert_eq!(embedding.server_tool_cost, Some(0.2));
    assert_eq!(
        serde_json::to_value(VideoUrl::new("https://example.com/video")).unwrap(),
        json!({"url":"https://example.com/video"})
    );
    assert_eq!(
        serde_json::to_value(VideoUrl::new("https://example.com/video").processing("agentic"))
            .unwrap()["processing"],
        "agentic"
    );
    let mut message = Message::new(Role::System, "configure");
    assert!(
        serde_json::to_value(&message)
            .unwrap()
            .get("configuration_update")
            .is_none()
    );
    message.configuration_update = Some(json!({"reasoning":{"effort":"high"}}));
    assert_eq!(
        serde_json::to_value(message).unwrap()["configuration_update"]["reasoning"]["effort"],
        "high"
    );
}
