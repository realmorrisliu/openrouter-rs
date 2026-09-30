use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
};

use futures_util::StreamExt;
use openrouter_rs::{
    OpenRouterClient,
    api::{
        api_keys::ApiKeyDetails,
        audio::{
            SpeechRequest, TranscriptionInputAudio, TranscriptionRequest, TranscriptionResponse,
        },
        byok::{ByokKey, CreateByokKeyRequest, UpdateByokKeyRequest},
        decisions::DecisionsRequest,
        embeddings::EmbeddingRequest,
        images::ImageGenerationRequest,
        interns::{
            CreateInternRequest, DeleteInternRequest, InternChatRequest, UpdateInternRequest,
        },
        messages::{AnthropicMessage, AnthropicMessagesRequest},
        rerank::RerankRequest,
        vault::{VaultSecretCopyRequest, VaultSecretWriteRequest},
        videos::VideoGenerationRequest,
        workspaces::{CreateWorkspaceRequest, UpdateWorkspaceRequest},
    },
};
use serde_json::json;

struct CapturedRequest {
    line: String,
    headers: String,
    body: String,
}

fn spawn_server(
    response_body: &str,
) -> (
    String,
    mpsc::Receiver<CapturedRequest>,
    thread::JoinHandle<()>,
) {
    spawn_server_with_status(response_body, 200)
}

fn spawn_server_with_status(
    response_body: &str,
    status: u16,
) -> (
    String,
    mpsc::Receiver<CapturedRequest>,
    thread::JoinHandle<()>,
) {
    spawn_server_sequence(vec![(status, response_body.to_string())])
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
            let headers = String::from_utf8_lossy(&bytes[..header_end]).to_string();
            let line = headers.lines().next().unwrap_or_default().to_string();
            let content_length = headers
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|length| length.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            while bytes.len() < header_end + content_length {
                let count = stream.read(&mut chunk).expect("read body");
                if count == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..count]);
            }
            let body = String::from_utf8_lossy(
                &bytes[header_end..(header_end + content_length).min(bytes.len())],
            )
            .to_string();
            tx.send(CapturedRequest {
                line,
                headers,
                body,
            })
            .expect("send capture");
            let content_type = if response_body.starts_with("data:") {
                "text/event-stream"
            } else {
                "application/json"
            };
            let response = format!(
                "HTTP/1.1 {status} OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body,
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        }
    });
    (format!("http://{addr}/api/v1"), rx, task)
}

fn client(base_url: String) -> OpenRouterClient {
    OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("test-key")
        .build()
        .expect("build client")
}

#[test]
fn issue_251_request_types_round_trip_upstream_fields() {
    let api_key: ApiKeyDetails = serde_json::from_value(json!({
        "label": "test",
        "usage": 1.0,
        "is_free_tier": false,
        "is_management_key": false,
        "rate_limit": {"requests": 100, "interval": "1h", "note": "deprecated"},
        "limit": null,
        "limit_remaining": null,
        "limit_reset": null,
        "allowed_data_regions": ["global", "europe"],
        "free_model_daily_requests": {"limit": 50, "remaining": 30, "used": 20},
        "organization_id": "org_123",
        "workspace_id": "workspace_123"
    }))
    .expect("decode key response");
    assert_eq!(api_key.allowed_data_regions, ["global", "europe"]);
    assert_eq!(
        api_key
            .free_model_daily_requests
            .as_ref()
            .unwrap()
            .remaining,
        30
    );
    assert_eq!(api_key.organization_id.as_deref(), Some("org_123"));
    assert_eq!(api_key.workspace_id.as_deref(), Some("workspace_123"));
    assert_eq!(api_key.limit_reset, None);

    let byok: ByokKey = serde_json::from_value(json!({
        "id": "byok_1", "provider": "openai", "workspace_id": null,
        "label": "OpenAI", "disabled": false, "is_fallback": false,
        "is_byok_only": true, "is_required": true, "declared_zdr": null,
        "sort_order": 0, "created_at": "2026-09-23T00:00:00Z"
    }))
    .expect("decode BYOK key");
    assert!(byok.is_byok_only && byok.is_required);
    assert_eq!(byok.declared_zdr, None);

    let byok_update = UpdateByokKeyRequest::builder()
        .is_byok_only(true)
        .is_required(false)
        .clear_declared_zdr()
        .build()
        .expect("build BYOK update");
    let byok_update = serde_json::to_value(byok_update).unwrap();
    assert_eq!(byok_update["is_byok_only"], true);
    assert_eq!(byok_update["is_required"], false);
    assert!(byok_update["declared_zdr"].is_null());

    let byok_create = CreateByokKeyRequest::builder()
        .provider("openai")
        .key("secret")
        .is_byok_only(true)
        .is_required(true)
        .declared_zdr(true)
        .build()
        .expect("build BYOK create");
    assert_eq!(
        serde_json::to_value(byok_create).unwrap()["declared_zdr"],
        true
    );

    let update = UpdateInternRequest::builder()
        .clear_model()
        .description("Researches customer questions")
        .build()
        .expect("build intern update");
    let update = serde_json::to_value(update).expect("serialize intern update");
    assert_eq!(
        update,
        json!({"description": "Researches customer questions", "model": null})
    );
    let cleared_description = UpdateInternRequest::builder()
        .clear_description()
        .build()
        .expect("build intern description clear");
    assert_eq!(
        serde_json::to_value(cleared_description).unwrap(),
        json!({"description": null})
    );

    let speech = SpeechRequest::builder()
        .model("openai/tts-1")
        .input("Hello")
        .session_id("session-1")
        .trace(Default::default())
        .user("user-1")
        .build()
        .expect("build speech request");
    let speech = serde_json::to_value(speech).expect("serialize speech request");
    assert_eq!(speech["session_id"], "session-1");
    assert_eq!(speech["user"], "user-1");
    assert!(speech.get("trace").is_some());

    let transcription_request = TranscriptionRequest::builder()
        .model("openai/whisper-1")
        .input_audio(TranscriptionInputAudio::new("UklGRiQA", "wav"))
        .session_id("session-1")
        .trace(Default::default())
        .user("user-1")
        .build()
        .expect("build transcription request");
    let transcription_request = serde_json::to_value(transcription_request).unwrap();
    assert_eq!(transcription_request["session_id"], "session-1");
    assert!(transcription_request.get("trace").is_some());

    let transcription: TranscriptionResponse = serde_json::from_value(json!({
        "text": "hello",
        "confidence": 0.98,
        "words": [{"word": "hello", "start": 0.0, "end": 0.5, "confidence": 0.97}]
    }))
    .expect("decode transcription");
    assert_eq!(transcription.confidence, Some(0.98));
    assert_eq!(transcription.words.unwrap()[0].confidence, Some(0.97));

    let video = VideoGenerationRequest::builder()
        .model("provider/video-model")
        .previous_job_id("gen-vid-1789493115-a1B2c3D4e5F6g7H8i9J0")
        .creativity(1)
        .upscale_factor(2.0)
        .session_id("session-2")
        .user("user-2")
        .trace(Default::default())
        .build()
        .expect("build video request");
    let video = serde_json::to_value(video).expect("serialize video request");
    assert_eq!(
        video["previous_job_id"],
        "gen-vid-1789493115-a1B2c3D4e5F6g7H8i9J0"
    );
    assert_eq!(video["creativity"], 1);
    assert_eq!(video["upscale_factor"], 2.0);

    let messages = AnthropicMessagesRequest::builder()
        .model("anthropic/claude-sonnet")
        .max_tokens(128)
        .messages(vec![AnthropicMessage::user("hello")])
        .safeguards([json!({"id": "moderation"})])
        .build()
        .expect("build messages request");
    let messages = serde_json::to_value(messages).expect("serialize messages");
    assert_eq!(messages["safeguards"], json!([{"id": "moderation"}]));

    let image = ImageGenerationRequest::builder()
        .model("openai/gpt-image-1")
        .prompt("a small blue bird")
        .session_id("session-3")
        .trace(Default::default())
        .user("user-3")
        .build()
        .expect("build image request");
    let image = serde_json::to_value(image).unwrap();
    assert_eq!(image["session_id"], "session-3");
    assert_eq!(image["user"], "user-3");

    let embedding = EmbeddingRequest::builder()
        .model("openai/text-embedding-3-large")
        .input("text")
        .session_id("session-4")
        .trace(Default::default())
        .build()
        .expect("build embedding request");
    let embedding = serde_json::to_value(embedding).unwrap();
    assert_eq!(embedding["session_id"], "session-4");

    let rerank = RerankRequest::builder()
        .model("cohere/rerank-v3.5")
        .query("query")
        .documents(["document"])
        .session_id("session-5")
        .trace(Default::default())
        .user("user-5")
        .build()
        .expect("build rerank request");
    let rerank = serde_json::to_value(rerank).unwrap();
    assert_eq!(rerank["session_id"], "session-5");
    assert_eq!(rerank["user"], "user-5");

    let workspace = UpdateWorkspaceRequest::builder()
        .disabled_server_tools(Vec::<String>::new())
        .build()
        .expect("build workspace update");
    assert_eq!(
        serde_json::to_value(workspace).expect("serialize workspace update")["disabled_server_tools"],
        json!([])
    );
    let workspace = UpdateWorkspaceRequest::builder()
        .clear_disabled_server_tools()
        .build()
        .expect("build workspace update that clears disabled tools");
    assert_eq!(
        serde_json::to_value(workspace).expect("serialize cleared workspace tools")["disabled_server_tools"],
        json!(null)
    );
    let cleared_workspace: UpdateWorkspaceRequest =
        serde_json::from_value(json!({"disabled_server_tools": null}))
            .expect("deserialize explicitly cleared workspace tools");
    assert_eq!(
        serde_json::to_value(cleared_workspace).unwrap()["disabled_server_tools"],
        json!(null)
    );

    let workspace = CreateWorkspaceRequest::builder()
        .name("Production")
        .slug("production")
        .disabled_server_tools(vec!["openrouter:web_search".to_string()])
        .build()
        .expect("build workspace request");
    assert_eq!(
        serde_json::to_value(workspace).unwrap()["disabled_server_tools"],
        json!(["openrouter:web_search"])
    );

    let create_intern = CreateInternRequest::builder()
        .name("worker")
        .description("Runs research tasks")
        .build()
        .expect("build intern request");
    assert_eq!(
        serde_json::to_value(create_intern).unwrap(),
        json!({"name": "worker", "description": "Runs research tasks"})
    );
    let chat = InternChatRequest::builder()
        .messages([json!({"role":"user","content":"hi"})])
        .build()
        .expect("build intern chat request");
    assert_eq!(serde_json::to_value(chat).unwrap()["stream"], true);
    let secret = VaultSecretWriteRequest::builder()
        .value("secret")
        .hosts(["api.example.com"])
        .build()
        .expect("build vault write request");
    assert_eq!(
        serde_json::to_value(secret).unwrap()["hosts"],
        json!(["api.example.com"])
    );
    let copy = VaultSecretCopyRequest::builder()
        .names(["token"])
        .build()
        .expect("build vault copy request");
    assert_eq!(
        serde_json::to_value(copy).unwrap()["names"],
        json!(["token"])
    );
}

#[tokio::test]
async fn decisions_and_system_one_use_their_documented_paths() {
    let request = DecisionsRequest::builder()
        .model("typesafe/jev-1.13")
        .state(json!({"issue": 251}))
        .questions([(
            "is_bug",
            json!({
                "type": "noul",
                "instructions": "Is this a bug?",
                "criteria": {"true": "yes", "false": "no"}
            }),
        )])
        .build()
        .expect("build Decisions request");
    let response = r#"{"model":"typesafe/jev-1.13","answers":{"is_bug":{"type":"noul","noul":0.8}},"usage":{"input_tokens":3,"output_tokens":1}}"#;

    let (base_url, rx, server) = spawn_server(response);
    client(base_url)
        .decisions()
        .create(&request)
        .await
        .expect("request Decisions");
    let captured = rx.recv().expect("capture Decisions request");
    assert_eq!(captured.line, "POST /api/alpha/decisions HTTP/1.1");
    assert!(
        captured
            .headers
            .to_ascii_lowercase()
            .contains("x-openrouter-title: openrouter-rs")
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&captured.body).unwrap()["model"],
        "typesafe/jev-1.13"
    );
    server.join().expect("finish Decisions server");

    let (base_url, rx, server) = spawn_server(response);
    client(base_url)
        .decisions()
        .create_system_one(&request)
        .await
        .expect("request System One");
    assert_eq!(
        rx.recv().expect("capture System One request").line,
        "POST /api/v1/systemone HTTP/1.1"
    );
    server.join().expect("finish System One server");
}

#[tokio::test]
async fn intern_and_vault_domains_send_expected_paths_and_auth() {
    let response = r#"{"data":[],"has_more":false}"#;
    let (base_url, rx, server) = spawn_server(response);
    client(base_url)
        .interns()
        .list(
            &openrouter_rs::api::interns::ListInternsParams::builder()
                .limit(5)
                .build()
                .unwrap(),
        )
        .await
        .expect("list interns");
    let captured = rx.recv().expect("capture intern list");
    assert!(captured.line.starts_with("GET /api/v1/interns?limit=5"));
    assert!(
        captured
            .headers
            .to_ascii_lowercase()
            .contains("authorization: bearer test-key")
    );
    server.join().expect("finish intern server");

    let (base_url, rx, server) = spawn_server(response);
    client(base_url)
        .vault()
        .list_for_intern("intern-1", Some(10), Some(0))
        .await
        .expect("list intern vault");
    let captured = rx.recv().expect("capture vault list");
    assert!(
        captured
            .line
            .starts_with("GET /api/v1/vault/interns/intern-1/secrets?")
    );
    server.join().expect("finish vault server");
}

#[tokio::test]
async fn intern_and_vault_mutations_cover_all_documented_routes() {
    let intern = r#"{"id":"intern-1","name":"worker","description":null,"instructions":"instructions","model":null,"status":"active","last_failure_message":null,"progress":null,"hostname":"worker.local","workspace_id":"workspace-1","vault_id":null,"attached_vault_id":null,"created_at":"2026-09-23T00:00:00Z","updated_at":"2026-09-23T00:00:00Z"}"#;
    let secret = r#"{"data":{"name":"token","hosts":["api.example.com"],"fingerprint":"sha256:abc","created_at":"2026-09-23T00:00:00Z"}}"#;
    let (base_url, rx, server) = spawn_server_sequence(vec![
        (201, intern.to_string()),
        (200, intern.to_string()),
        (200, intern.to_string()),
        (202, r#"{"deleting":true}"#.to_string()),
        (202, r#"{"deleting":true}"#.to_string()),
        (202, r#"{"provisioning":true}"#.to_string()),
        (200, r#"{"suspended":true}"#.to_string()),
        (200, r#"{"data":[],"has_more":false}"#.to_string()),
        (200, secret.to_string()),
        (204, String::new()),
        (200, secret.to_string()),
        (204, String::new()),
        (200, r#"{"data":[]}"#.to_string()),
    ]);
    let client = client(base_url);

    client
        .interns()
        .create(
            &CreateInternRequest::builder()
                .name("worker")
                .description("Runs research tasks")
                .instructions("instructions")
                .build()
                .unwrap(),
            Some("issue-251-create"),
        )
        .await
        .unwrap();
    client.interns().get("intern-1").await.unwrap();
    client
        .interns()
        .update(
            "intern-1",
            &UpdateInternRequest::builder()
                .clear_description()
                .clear_model()
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    client
        .interns()
        .delete(
            "intern-1",
            Some(
                &DeleteInternRequest::builder()
                    .acknowledge_workspace_loss(true)
                    .build()
                    .unwrap(),
            ),
        )
        .await
        .unwrap();
    client.interns().delete("intern-1", None).await.unwrap();
    client.interns().provision("intern-1").await.unwrap();
    client.interns().suspend("intern-1").await.unwrap();
    client.vault().list(Some(10), Some(2)).await.unwrap();

    let write = VaultSecretWriteRequest::builder()
        .value("secret")
        .hosts(["api.example.com"])
        .build()
        .unwrap();
    client.vault().store("token", &write).await.unwrap();
    client.vault().delete("token").await.unwrap();
    client
        .vault()
        .store_for_intern("intern-1", "token", &write)
        .await
        .unwrap();
    client
        .vault()
        .delete_for_intern("intern-1", "token")
        .await
        .unwrap();
    client
        .vault()
        .copy_to_intern(
            "intern-1",
            &VaultSecretCopyRequest::builder()
                .names(["token"])
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    let requests: Vec<_> = (0..13)
        .map(|_| rx.recv().expect("capture intern or vault request"))
        .collect();
    let paths: Vec<_> = requests
        .iter()
        .map(|request| request.line.as_str())
        .collect();
    assert_eq!(
        paths,
        [
            "POST /api/v1/interns HTTP/1.1",
            "GET /api/v1/interns/intern-1 HTTP/1.1",
            "PATCH /api/v1/interns/intern-1 HTTP/1.1",
            "DELETE /api/v1/interns/intern-1 HTTP/1.1",
            "DELETE /api/v1/interns/intern-1 HTTP/1.1",
            "POST /api/v1/interns/intern-1/provision HTTP/1.1",
            "POST /api/v1/interns/intern-1/suspend HTTP/1.1",
            "GET /api/v1/vault/secrets?limit=10&offset=2 HTTP/1.1",
            "PUT /api/v1/vault/secrets/token HTTP/1.1",
            "DELETE /api/v1/vault/secrets/token HTTP/1.1",
            "PUT /api/v1/vault/interns/intern-1/secrets/token HTTP/1.1",
            "DELETE /api/v1/vault/interns/intern-1/secrets/token HTTP/1.1",
            "POST /api/v1/vault/interns/intern-1/secrets/copy HTTP/1.1",
        ]
    );
    assert!(requests.iter().all(|request| {
        request
            .headers
            .to_ascii_lowercase()
            .contains("authorization: bearer test-key")
    }));
    assert!(
        requests[0]
            .headers
            .to_ascii_lowercase()
            .contains("idempotency-key: issue-251-create")
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[0].body).unwrap()["description"],
        "Runs research tasks"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[2].body).unwrap(),
        json!({"description": null, "model": null})
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[3].body).unwrap(),
        json!({"acknowledge_workspace_loss": true})
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[4].body).unwrap(),
        json!({})
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[8].body).unwrap()["hosts"],
        json!(["api.example.com"])
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[12].body).unwrap()["names"],
        json!(["token"])
    );
    server.join().expect("finish intern and vault server");
}

#[tokio::test]
async fn intern_chat_handles_streaming_and_steered_responses() {
    let request = InternChatRequest::builder()
        .messages([json!({"role": "user", "content": "hello"})])
        .session_id("session-1")
        .build()
        .expect("build intern chat request");
    let sse = "data: {\"id\":\"chunk-1\"}\n\ndata: [DONE]\n\n";
    let (base_url, rx, server) = spawn_server(sse);
    let result = client(base_url)
        .interns()
        .chat_completion("intern-1", &request)
        .await
        .expect("stream intern chat");
    let mut stream = match result {
        openrouter_rs::api::interns::InternChatResult::Streaming(stream) => stream,
        openrouter_rs::api::interns::InternChatResult::Steered(_) => panic!("expected stream"),
    };
    assert_eq!(stream.next().await.unwrap().unwrap()["id"], "chunk-1");
    let captured = rx.recv().expect("capture intern chat request");
    assert_eq!(
        captured.line,
        "POST /api/v1/interns/intern-1/chat/completions HTTP/1.1"
    );
    assert!(
        captured
            .headers
            .to_ascii_lowercase()
            .contains("x-openrouter-title: openrouter-rs")
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&captured.body).unwrap()["stream"],
        true
    );
    server.join().expect("finish chat server");

    let (base_url, rx, server) =
        spawn_server_with_status(r#"{"session_id":"session-1","status":"steered"}"#, 202);
    let result = client(base_url)
        .interns()
        .chat_completion("intern-1", &request)
        .await
        .expect("steer intern chat");
    assert!(matches!(
        result,
        openrouter_rs::api::interns::InternChatResult::Steered(_)
    ));
    assert!(
        rx.recv()
            .expect("capture steered request")
            .line
            .starts_with("POST /api/v1/interns/intern-1/chat/completions")
    );
    server.join().expect("finish steered server");
}

#[tokio::test]
async fn issue_254_batches_cover_paths_repeated_status_and_errors() {
    use openrouter_rs::api::batches::{BatchRequest, CreateBatchRequest, ListBatchesParams};
    let batch = json!({"id":"batch-1","object":"batch","endpoint":"/v1/responses","model":"test/model","completion_window":"24h","status":"completed","created_at":1,"finalized_at":2,"request_counts":{"total":1,"completed":1,"failed":0},"usage":{"prompt_tokens":1,"completion_tokens":2,"total_tokens":3,"prompt_tokens_details":{"cached_tokens":1}},"error":null,"results":[{"id":"result-1","custom_id":"req-1","response":{"status_code":200,"request_id":"upstream-1","body":{"output":[{"type":"future_output"}]}},"error":null}]});
    let mut listed = batch.clone();
    listed["results"] = json!(null);
    let (base_url, rx, server) = spawn_server_sequence(vec![
        (202, batch.to_string()),
        (200, json!({"object":"list","data":[listed],"first_id":"batch-1","last_id":"batch-1","has_more":false}).to_string()),
        (200, batch.to_string()),
        (200, json!({"id":"batch-1","object":"batch","deletion":{"openrouter":"deleted","upstream":{"provider":"future-provider","status":"unsupported"}}}).to_string()),
        (402, json!({"error":{"code":402,"message":"Add credits to unlock results"},"results":null}).to_string()),
    ]);
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("test-key")
        .x_title("review-tests")
        .http_referer("https://example.com")
        .app_categories(["cli-agent", "cloud-agent"])
        .build()
        .unwrap();
    let body = json!({"input":"hello"}).as_object().unwrap().clone();
    let request = CreateBatchRequest::builder()
        .endpoint("/v1/responses")
        .model("test/model")
        .requests(vec![BatchRequest::new("req-1", body)])
        .build()
        .unwrap();
    let created = client.batches().create(&request).await.unwrap();
    assert_eq!(
        created.usage.unwrap()["prompt_tokens_details"]["cached_tokens"],
        1
    );
    let page = client
        .batches()
        .list(
            &ListBatchesParams::builder()
                .limit(5)
                .after("batch-0")
                .status(["completed", "failed"])
                .created_after("2026-09-28")
                .created_before("1234567890")
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(page.data[0].results.is_none());
    let fetched = client.batches().get("batch/1").await.unwrap();
    let result = fetched.results.unwrap().remove(0).response.unwrap();
    assert_eq!(result.request_id.as_deref(), Some("upstream-1"));
    assert_eq!(result.body["output"][0]["type"], "future_output");
    let deleted = client.batches().delete("batch/1").await.unwrap();
    assert_eq!(deleted.deletion.upstream.unwrap().status, "unsupported");
    assert!(
        matches!(client.batches().get("batch-1").await, Err(openrouter_rs::error::OpenRouterError::Api(error)) if error.status.as_u16() == 402)
    );
    let requests: Vec<_> = (0..5).map(|_| rx.recv().unwrap()).collect();
    assert_eq!(requests[0].line, "POST /api/v1/batches HTTP/1.1");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[0].body).unwrap(),
        json!({"endpoint":"/v1/responses","model":"test/model","requests":[{"custom_id":"req-1","body":{"input":"hello"}}]})
    );
    assert_eq!(
        requests[1].line,
        "GET /api/v1/batches?limit=5&after=batch-0&created_after=2026-09-28&created_before=1234567890&status=completed&status=failed HTTP/1.1"
    );
    assert_eq!(requests[2].line, "GET /api/v1/batches/batch%2F1 HTTP/1.1");
    assert_eq!(
        requests[3].line,
        "DELETE /api/v1/batches/batch%2F1 HTTP/1.1"
    );
    for request in requests {
        let headers = request.headers.to_ascii_lowercase();
        for expected in [
            "x-title: review-tests",
            "x-openrouter-title: review-tests",
            "http-referer: https://example.com",
            "x-openrouter-categories: cli-agent,cloud-agent",
        ] {
            assert!(headers.contains(expected), "missing {expected}: {headers}");
        }
        assert!(
            request
                .headers
                .to_ascii_lowercase()
                .contains("authorization: bearer test-key")
        );
    }
    server.join().unwrap();
    let management_only = OpenRouterClient::builder()
        .management_key("management-key")
        .build()
        .unwrap();
    assert!(matches!(
        management_only.batches().get("batch-1").await,
        Err(openrouter_rs::error::OpenRouterError::KeyNotConfigured)
    ));
    assert!(
        CreateBatchRequest::builder()
            .endpoint("/v1/responses")
            .model("test/model")
            .build()
            .is_err()
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn issue_254_intern_invoke_daemon_and_scim_filters() {
    use openrouter_rs::api::{interns::InternInvokeRequest, scim::ListScimGroupsParams};
    let (base_url, rx, server) = spawn_server_sequence(vec![
        (
            202,
            json!({"session_id":"session-1","status":"started"}).to_string(),
        ),
        (
            202,
            json!({"session_id":"session-1","status":"steered"}).to_string(),
        ),
        (
            200,
            json!({"origin":"https://intern.example.com","token":"secret"}).to_string(),
        ),
        (
            200,
            json!({"origin":"https://intern.example.com","token":"secret"}).to_string(),
        ),
        (200, json!({"data":[],"total_count":0}).to_string()),
        (200, json!({"data":[],"total_count":0}).to_string()),
    ]);
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("test-key")
        .management_key("management-key")
        .x_title("review-tests")
        .http_referer("https://example.com")
        .app_categories(["cli-agent", "cloud-agent"])
        .build()
        .unwrap();
    let request = InternInvokeRequest::builder()
        .input("Investigate ticket")
        .build()
        .unwrap();
    assert_eq!(
        client
            .interns()
            .invoke("intern/1", &request)
            .await
            .unwrap()
            .status,
        "started"
    );
    let request = InternInvokeRequest::builder()
        .input("Continue")
        .session_id("session-1")
        .build()
        .unwrap();
    assert_eq!(
        client
            .interns()
            .invoke("intern/1", &request)
            .await
            .unwrap()
            .status,
        "steered"
    );
    assert_eq!(
        client.interns().daemon("intern/1").await.unwrap().token,
        "secret"
    );
    assert_eq!(
        client
            .interns()
            .daemon_access("intern/1")
            .await
            .unwrap()
            .origin,
        "https://intern.example.com"
    );
    client
        .management()
        .list_scim_groups_with_params(
            &ListScimGroupsParams::builder()
                .offset(2)
                .limit(10)
                .display_name("R&D")
                .external_id("external/1")
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    client
        .management()
        .list_scim_groups(Some(
            openrouter_rs::types::PaginationOptions::with_offset_and_limit(3, 5),
        ))
        .await
        .unwrap();
    let requests: Vec<_> = (0..6).map(|_| rx.recv().unwrap()).collect();
    assert_eq!(
        requests[0].line,
        "POST /api/v1/interns/intern%2F1/invoke HTTP/1.1"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[0].body).unwrap(),
        json!({"input":"Investigate ticket"})
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&requests[1].body).unwrap(),
        json!({"input":"Continue","session_id":"session-1"})
    );
    assert_eq!(
        requests[2].line,
        "GET /api/v1/interns/intern%2F1/daemon HTTP/1.1"
    );
    assert_eq!(
        requests[3].line,
        "GET /api/v1/interns/intern%2F1/daemon-access HTTP/1.1"
    );
    assert_eq!(
        requests[4].line,
        "GET /api/v1/scim/groups?offset=2&limit=10&display_name=R%26D&external_id=external%2F1 HTTP/1.1"
    );
    assert_eq!(
        requests[5].line,
        "GET /api/v1/scim/groups?offset=3&limit=5 HTTP/1.1"
    );
    for (index, request) in requests.iter().enumerate() {
        let headers = request.headers.to_ascii_lowercase();
        for expected in [
            "x-title: review-tests",
            "x-openrouter-title: review-tests",
            "http-referer: https://example.com",
            "x-openrouter-categories: cli-agent,cloud-agent",
        ] {
            assert!(headers.contains(expected), "missing {expected}: {headers}");
        }
        assert!(request.headers.to_ascii_lowercase().contains(if index < 4 {
            "authorization: bearer test-key"
        } else {
            "authorization: bearer management-key"
        }));
    }
    assert!(
        requests[4]
            .headers
            .to_ascii_lowercase()
            .contains("authorization: bearer management-key")
    );
    server.join().unwrap();
}

#[test]
fn issue_254_audio_url_image_and_transcription_fields() {
    use openrouter_rs::api::audio::SpeechInputReference;
    let request = SpeechRequest::builder()
        .model("test/speech")
        .input("hello")
        .input_references([
            SpeechInputReference::audio_url("https://example.com/audio.wav"),
            SpeechInputReference::image("https://example.com/image.png"),
        ])
        .build()
        .unwrap();
    let body = serde_json::to_value(request).unwrap();
    assert_eq!(
        body["input_references"],
        json!([
            {"type":"input_audio","input_audio":{"url":"https://example.com/audio.wav"}},
            {"type":"image_url","image_url":{"url":"https://example.com/image.png"}}
        ])
    );
    let request = TranscriptionRequest::builder()
        .model("test/stt")
        .input_audio(TranscriptionInputAudio::from_url(
            "https://example.com/audio.wav",
        ))
        .diarize(true)
        .keyterms(vec!["OpenRouter".to_string()])
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(request).unwrap(),
        json!({"model":"test/stt","input_audio":{"url":"https://example.com/audio.wav"},"diarize":true,"keyterms":["OpenRouter"]})
    );
    let response: TranscriptionResponse = serde_json::from_value(json!({"text":"John","language_confidence":0.98,"entities":[{"text":"John","type":"name","start_char":0,"end_char":4}],"words":[{"word":"John","start":0.0,"end":1.0,"speaker_label":"speaker_0","channel":0,"type":"word"}]})).unwrap();
    assert_eq!(response.language_confidence, Some(0.98));
    assert_eq!(response.entities.unwrap()[0].entity_type, "name");
    let words = response.words.unwrap();
    assert_eq!(words[0].speaker_label.as_deref(), Some("speaker_0"));
    assert_eq!(words[0].channel, Some(0));
    assert_eq!(words[0].word_type.as_deref(), Some("word"));
    assert!(TranscriptionInputAudio::builder().build().is_err());
    assert!(
        TranscriptionInputAudio::builder()
            .data("base64")
            .build()
            .is_err()
    );
    assert!(
        TranscriptionInputAudio::builder()
            .data("base64")
            .format("wav")
            .url("https://example.com/audio.wav")
            .build()
            .is_err()
    );
    let url = TranscriptionInputAudio::builder()
        .url("https://example.com/audio.wav")
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(url).unwrap(),
        json!({"url":"https://example.com/audio.wav"})
    );
    assert!(
        openrouter_rs::api::audio::SpeechInputAudio::builder()
            .build()
            .is_err()
    );
    let audio = TranscriptionInputAudio::new("base64", "wav");
    assert_eq!(
        serde_json::to_value(audio).unwrap(),
        json!({"data":"base64","format":"wav"})
    );
}

#[test]
fn issue_254_shared_provider_options_and_endpoint_capabilities() {
    use openrouter_rs::{api::models::Endpoint, types::ProviderPreferences};
    let mut provider = ProviderPreferences::default();
    provider.options = Some([("elevenlabs".into(), json!({"style":0.5}))].into());
    let body = openrouter_rs::api::chat::ChatCompletionRequest::builder()
        .model("test/model")
        .messages(vec![openrouter_rs::api::chat::Message::new(
            openrouter_rs::types::Role::User,
            "hello",
        )])
        .provider(provider.clone())
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(body).unwrap()["provider"]["options"],
        json!({"elevenlabs":{"style":0.5}})
    );
    assert_eq!(
        serde_json::from_value::<ProviderPreferences>(serde_json::to_value(provider).unwrap())
            .unwrap()
            .options
            .unwrap()["elevenlabs"]["style"],
        0.5
    );
    let endpoint: Endpoint = serde_json::from_value(json!({"name":"test","context_length":100,"pricing":{"prompt":"0","completion":"0"},"provider_name":"ElevenLabs","supported_parameters":[],"supports_image_reference":true,"supports_multiple_audio_references":false})).unwrap();
    assert_eq!(endpoint.supports_image_reference, Some(true));
    assert_eq!(endpoint.supports_multiple_audio_references, Some(false));
    let minimal = serde_json::to_value(ProviderPreferences::default()).unwrap();
    assert!(minimal.get("options").is_none());
}

#[test]
fn issue_254_audio_builders_reject_empty_url_alongside_data() {
    use openrouter_rs::api::audio::SpeechInputAudio;
    assert!(SpeechInputAudio::builder().url("").build().is_err());
    assert!(TranscriptionInputAudio::builder().url("").build().is_err());
    assert!(
        SpeechInputAudio::builder()
            .data("base64")
            .url("")
            .build()
            .is_err()
    );
    assert!(
        TranscriptionInputAudio::builder()
            .data("base64")
            .format("wav")
            .url("")
            .build()
            .is_err()
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn issue_254_malformed_credential_responses_do_not_leak_payloads() {
    use openrouter_rs::api::auth::OAuthTokenExchangeRequest;
    let (base_url, rx, server) = spawn_server_sequence(vec![
        (200, r#"{"token":"daemon-secret"}"#.into()),
        (200, r#"{"origin":123,"token":"alias-secret"}"#.into()),
        (200, r#"{"token":"truncated-secret""#.into()),
        (200, r#"{"token":"envelope-secret","error":{}}"#.into()),
        (
            200,
            r#"{"access_token":"oauth-secret","expires_in":"serde-secret"}"#.into(),
        ),
        (
            200,
            r#"{"data":{"key":"created-key-secret","limit":"serde-api-secret"}}"#.into(),
        ),
        (
            200,
            r#"{"error":{"code":403,"message":"Forbidden"},"token":"ignored-secret"}"#.into(),
        ),
    ]);
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("test-key")
        .management_key("management-key")
        .build()
        .unwrap();
    let errors = [
        client.interns().daemon("intern-1").await.err().unwrap(),
        client
            .interns()
            .daemon_access("intern-1")
            .await
            .err()
            .unwrap(),
        client.interns().daemon("intern-1").await.err().unwrap(),
        client.interns().daemon("intern-1").await.err().unwrap(),
        client
            .management()
            .exchange_oauth_token(
                &OAuthTokenExchangeRequest::builder()
                    .federation_policy_id("policy-1")
                    .subject_token("test-jwt")
                    .build()
                    .unwrap(),
            )
            .await
            .unwrap_err(),
        client
            .management()
            .create_api_key("test", None)
            .await
            .unwrap_err(),
    ];
    for error in errors {
        assert!(matches!(
            error,
            openrouter_rs::error::OpenRouterError::Unknown(_)
        ));
        for message in [error.to_string(), format!("{error:?}")] {
            assert!(message.contains("status 200"), "{message}");
            assert!(message.contains("error at line"), "{message}");
            assert!(!message.contains("secret"), "{message}");
            assert!(!message.contains("body preview"), "{message}");
        }
    }
    let error = client.interns().daemon("intern-1").await.err().unwrap();
    assert!(
        matches!(error, openrouter_rs::error::OpenRouterError::Api(ref context) if context.status.as_u16() == 403 && context.message == "Forbidden")
    );
    assert!(!format!("{error:?}").contains("ignored-secret"));
    for _ in 0..7 {
        rx.recv().unwrap();
    }
    server.join().unwrap();
}

#[tokio::test]
#[allow(deprecated)]
async fn issue_254_credential_error_statuses_do_not_echo_raw_bodies() {
    use openrouter_rs::api::auth::OAuthTokenExchangeRequest;
    let (base_url, rx, server) = spawn_server_sequence(vec![
        (400, r#"{"origin":"https://intern.example.com","token":"status-secret"}"#.into()),
        (404, r#"{"token":"alias-error-secret""#.into()),
        (502, "plain-secret".into()),
        (401, r#"{"access_token":"oauth-error-secret"}"#.into()),
        (500, r#"{"data":{"key":"key-error-secret"}}"#.into()),
        (429, r#"{"error":{"code":429,"message":"Wait","metadata":{"reason":"rate_limited","retryable":true}}}"#.into()),
    ]);
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("test-key")
        .management_key("management-key")
        .build()
        .unwrap();
    let errors = [
        client.interns().daemon("intern-1").await.err().unwrap(),
        client
            .interns()
            .daemon_access("intern-1")
            .await
            .err()
            .unwrap(),
        client.interns().daemon("intern-1").await.err().unwrap(),
        client
            .management()
            .exchange_oauth_token(
                &OAuthTokenExchangeRequest::builder()
                    .federation_policy_id("policy-1")
                    .subject_token("test-jwt")
                    .build()
                    .unwrap(),
            )
            .await
            .unwrap_err(),
        client
            .management()
            .create_api_key("test", None)
            .await
            .unwrap_err(),
    ];
    for (error, status) in errors.into_iter().zip([400, 404, 502, 401, 500]) {
        assert!(
            matches!(error, openrouter_rs::error::OpenRouterError::Api(ref context) if context.status.as_u16() == status && context.message == "Invalid credential error response (body omitted)")
        );
        assert!(!error.to_string().contains("secret"));
        assert!(!format!("{error:?}").contains("secret"));
    }
    let error = client.interns().daemon("intern-1").await.err().unwrap();
    assert!(
        matches!(error, openrouter_rs::error::OpenRouterError::Api(ref context) if context.status.as_u16() == 429 && context.message == "Wait" && context.is_retryable() && context.metadata.as_ref().unwrap()["reason"] == "rate_limited")
    );
    for _ in 0..6 {
        rx.recv().unwrap();
    }
    server.join().unwrap();
}

#[tokio::test]
async fn issue_254_credential_body_read_failure_preserves_error_context() {
    for status in [429, 503] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}/api/v1", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut chunk = [0; 1024];
            while !request.windows(4).any(|part| part == b"\r\n\r\n") {
                let count = stream.read(&mut chunk).unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&chunk[..count]);
            }
            // Close before the advertised body length to reproduce a transport read failure.
            stream.write_all(format!("HTTP/1.1 {status} Error\r\nContent-Type: application/json\r\nContent-Length: 1000\r\nX-Request-Id: truncated-credential\r\nConnection: close\r\n\r\n{{\"token\":\"partial-secret\"}}").as_bytes()).unwrap();
        });
        let error = client(base_url)
            .interns()
            .daemon("intern-1")
            .await
            .err()
            .unwrap();
        assert!(
            matches!(error, openrouter_rs::error::OpenRouterError::Api(ref context) if context.status.as_u16() == status && context.request_id.as_deref() == Some("truncated-credential") && context.is_retryable() && context.metadata.as_ref().unwrap().get("body_read_error").is_some())
        );
        assert!(!error.to_string().contains("partial-secret"));
        assert!(!format!("{error:?}").contains("partial-secret"));
        server.join().unwrap();
    }
}
