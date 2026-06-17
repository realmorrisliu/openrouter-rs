use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::discovery::{
        self, ActivityItem, AppRankingsParams, AppRankingsResponse, BenchmarksAAResponse,
        BenchmarksDAResponse, BigNumber, ModelsCountData, Provider, PublicEndpoint,
        RankingsDailyResponse, UserModel,
    },
    types::ApiResponse,
};

struct CapturedRequest {
    request_line: String,
    request_text: String,
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

        let request_text = String::from_utf8_lossy(&request_bytes).to_string();
        let request_line = request_text.lines().next().unwrap_or_default().to_string();
        tx.send(CapturedRequest {
            request_line,
            request_text,
        })
        .expect("server should send request");

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

#[test]
fn test_providers_response_deserialization() {
    let raw = r#"{
        "data": [{
            "name": "OpenAI",
            "slug": "openai",
            "privacy_policy_url": "https://openai.com/privacy",
            "terms_of_service_url": "https://openai.com/terms",
            "status_page_url": "https://status.openai.com"
        }]
    }"#;

    let parsed: ApiResponse<Vec<Provider>> =
        serde_json::from_str(raw).expect("providers response should deserialize");
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].slug, "openai");
    assert_eq!(
        parsed.data[0].status_page_url.as_deref(),
        Some("https://status.openai.com")
    );
}

#[test]
fn test_models_for_user_response_deserialization() {
    let raw = r#"{
        "data": [{
            "id": "openai/gpt-4.1",
            "canonical_slug": "openai/gpt-4.1",
            "hugging_face_id": null,
            "name": "GPT-4.1",
            "created": 1710000000,
            "description": "Test model",
            "pricing": {
                "prompt": "0.000002",
                "completion": 0.000008
            },
            "context_length": 128000,
            "architecture": {
                "tokenizer": "GPT",
                "instruct_type": "chatml",
                "modality": "text->text",
                "input_modalities": ["text"],
                "output_modalities": ["text"]
            },
            "top_provider": {
                "context_length": 128000,
                "max_completion_tokens": 16384,
                "is_moderated": true
            },
            "per_request_limits": null,
            "supported_parameters": ["temperature", "top_p"],
            "supported_voices": ["alloy", "verse"],
            "default_parameters": null,
            "expiration_date": null
        }]
    }"#;

    let parsed: ApiResponse<Vec<UserModel>> =
        serde_json::from_str(raw).expect("models/user response should deserialize");
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].canonical_slug, "openai/gpt-4.1");
    assert_eq!(parsed.data[0].supported_parameters.len(), 2);
    assert_eq!(
        parsed.data[0].supported_voices.as_deref(),
        Some(["alloy".to_string(), "verse".to_string()].as_slice())
    );
    assert!(matches!(
        parsed.data[0].pricing.prompt,
        BigNumber::String(_)
    ));
    assert!(matches!(
        parsed.data[0].pricing.completion,
        BigNumber::Number(_)
    ));
}

#[test]
fn test_models_count_response_deserialization() {
    let raw = r#"{"data":{"count":150}}"#;
    let parsed: ApiResponse<ModelsCountData> =
        serde_json::from_str(raw).expect("models/count response should deserialize");
    assert_eq!(parsed.data.count, 150);
}

#[test]
fn test_zdr_endpoints_response_deserialization() {
    let raw = r#"{
        "data": [{
            "name": "OpenAI: GPT-4.1",
            "model_id": "openai/gpt-4.1-2025-04-14",
            "model_name": "GPT-4.1",
            "context_length": 128000,
            "pricing": {
                "prompt": "0.000002",
                "completion": "0.000008"
            },
            "provider_name": "OpenAI",
            "tag": "openai",
            "quantization": null,
            "max_completion_tokens": 16384,
            "max_prompt_tokens": 128000,
            "supported_parameters": ["temperature", "top_p"],
            "status": 0,
            "uptime_last_30m": 99.9,
            "supports_implicit_caching": true,
            "latency_last_30m": {
                "p50": 0.1,
                "p75": 0.2,
                "p90": 0.3,
                "p99": 0.5
            },
            "throughput_last_30m": null
        }]
    }"#;

    let parsed: ApiResponse<Vec<PublicEndpoint>> =
        serde_json::from_str(raw).expect("endpoints/zdr response should deserialize");
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].model_name, "GPT-4.1");
    assert_eq!(parsed.data[0].status, Some(0));
    assert!(parsed.data[0].throughput_last_30m.is_none());
}

#[test]
fn test_activity_response_deserialization() {
    let raw = r#"{
        "data": [{
            "date": "2025-08-24",
            "model": "openai/gpt-4.1",
            "model_permaslug": "openai/gpt-4.1-2025-04-14",
            "endpoint_id": "550e8400-e29b-41d4-a716-446655440000",
            "provider_name": "OpenAI",
            "usage": 0.015,
            "byok_usage_inference": 0.012,
            "requests": 5,
            "prompt_tokens": 50,
            "completion_tokens": 125,
            "reasoning_tokens": 25
        }]
    }"#;

    let parsed: ApiResponse<Vec<ActivityItem>> =
        serde_json::from_str(raw).expect("activity response should deserialize");
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].date, "2025-08-24");
    assert_eq!(parsed.data[0].requests, 5.0);
}

#[test]
fn test_rankings_daily_response_deserialization() {
    let raw = r#"{
        "data": [{
            "date": "2026-05-11",
            "model_permaslug": "openai/gpt-4o-2024-05-13",
            "total_tokens": "12345678"
        }, {
            "date": "2026-05-11",
            "model_permaslug": "other",
            "total_tokens": "4321098"
        }],
        "meta": {
            "as_of": "2026-05-12T02:00:00Z",
            "version": "v1",
            "start_date": "2026-04-12",
            "end_date": "2026-05-11"
        }
    }"#;

    let parsed: RankingsDailyResponse =
        serde_json::from_str(raw).expect("rankings daily response should deserialize");
    assert_eq!(parsed.data.len(), 2);
    assert_eq!(parsed.data[0].total_tokens, "12345678");
    assert_eq!(parsed.data[1].model_permaslug, "other");
    assert_eq!(parsed.meta.version, "v1");
}

#[test]
fn test_app_rankings_response_deserialization() {
    let raw = r#"{
        "data": [{
            "rank": 1,
            "app_id": 12345,
            "app_name": "Cline",
            "total_tokens": "12345678",
            "total_requests": 4321
        }],
        "meta": {
            "as_of": "2026-05-12T02:00:00Z",
            "version": "v1",
            "start_date": "2026-04-12",
            "end_date": "2026-05-11"
        }
    }"#;

    let parsed: AppRankingsResponse =
        serde_json::from_str(raw).expect("app rankings response should deserialize");
    assert_eq!(parsed.data[0].app_name, "Cline");
    assert_eq!(parsed.data[0].total_tokens, "12345678");
    assert_eq!(parsed.meta.start_date, "2026-04-12");
}

#[test]
fn test_benchmark_dataset_responses_deserialize() {
    let artificial_raw = r#"{
        "data": [{
            "model_permaslug": "openai/gpt-4o",
            "aa_name": "GPT-4o",
            "intelligence_index": 71.2,
            "coding_index": 65.8,
            "agentic_index": 58.3,
            "pricing": {
                "prompt": "0.0000025",
                "completion": "0.00001"
            }
        }],
        "meta": {
            "as_of": "2026-06-03T12:00:00Z",
            "version": "v1",
            "source": "artificial-analysis",
            "source_url": "https://artificialanalysis.ai",
            "citation": "Source: Artificial Analysis via OpenRouter.",
            "model_count": 1
        }
    }"#;
    let artificial: BenchmarksAAResponse =
        serde_json::from_str(artificial_raw).expect("AA benchmark response should deserialize");
    assert_eq!(artificial.data[0].aa_name, "GPT-4o");
    assert_eq!(
        artificial.data[0]
            .pricing
            .as_ref()
            .expect("pricing should be present")
            .prompt,
        "0.0000025"
    );

    let design_raw = r#"{
        "data": [{
            "model_permaslug": "anthropic/claude-sonnet-4",
            "display_name": "Claude Sonnet 4",
            "arena": "models",
            "category": "codecategories",
            "elo": 1423,
            "win_rate": 72,
            "avg_generation_time_ms": 3200,
            "tournament_stats": {
                "first_place": 12,
                "second_place": 8,
                "third_place": 5,
                "fourth_place": 2,
                "total": 27
            },
            "pricing": null
        }],
        "meta": {
            "as_of": "2026-06-03T12:00:00Z",
            "version": "v1",
            "source": "design-arena",
            "source_url": "https://www.designarena.ai",
            "citation": "Source: Design Arena via OpenRouter.",
            "model_count": 1,
            "arena": "models",
            "category": null,
            "elo_bounds": {
                "min": 900,
                "max": 1600
            }
        }
    }"#;
    let design: BenchmarksDAResponse =
        serde_json::from_str(design_raw).expect("Design Arena response should deserialize");
    assert_eq!(design.data[0].display_name, "Claude Sonnet 4");
    assert_eq!(design.meta.elo_bounds.max, 1600.0);
}

#[tokio::test]
async fn test_list_models_for_user_request_path() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<String>();

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
        tx.send(request_line)
            .expect("server should send request line");

        let body = r#"{"data":[]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let models = discovery::list_models_for_user(&base_url, "test-key")
        .await
        .expect("models/user request should succeed");
    assert!(models.is_empty());

    let request_line = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request line");
    assert_eq!(request_line, "GET /api/v1/models/user HTTP/1.1");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_activity_with_date_query_and_auth_header() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<String>();

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

        let request_text = String::from_utf8_lossy(&request_bytes).to_string();
        tx.send(request_text).expect("server should send request");

        let body = r#"{"data":[]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let items = discovery::get_activity(&base_url, "mgmt-key", Some("2025-08-24"))
        .await
        .expect("activity request should succeed");
    assert!(items.is_empty());

    let request_text = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    let mut lines = request_text.lines();
    let request_line = lines.next().unwrap_or_default();
    assert_eq!(
        request_line,
        "GET /api/v1/activity?date=2025-08-24 HTTP/1.1"
    );

    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer mgmt-key")
            || request_lower.contains("authorization:bearer mgmt-key"),
        "authorization header should include management key, request:\n{request_text}"
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_providers_request_path_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[]}"#);

    let providers = discovery::list_providers(&base_url, "api-key")
        .await
        .expect("providers request should succeed");
    assert!(providers.is_empty(), "response payload should parse");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/providers HTTP/1.1");

    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include API key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_count_models_request_path_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":{"count":123}}"#);

    let count = discovery::count_models(&base_url, "api-key")
        .await
        .expect("models/count request should succeed");
    assert_eq!(count.count, 123);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/models/count HTTP/1.1");

    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include API key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_zdr_endpoints_request_path_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[]}"#);

    let endpoints = discovery::list_zdr_endpoints(&base_url, "api-key")
        .await
        .expect("endpoints/zdr request should succeed");
    assert!(endpoints.is_empty(), "response payload should parse");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/endpoints/zdr HTTP/1.1");

    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include API key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_rankings_daily_with_date_query_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[],"meta":{"as_of":"2026-05-12T02:00:00Z","version":"v1","start_date":"2026-04-12","end_date":"2026-05-11"}}"#,
    );

    let rankings =
        discovery::get_rankings_daily(&base_url, "api-key", Some("2026-04-12"), Some("2026-05-11"))
            .await
            .expect("rankings daily request should succeed");
    assert!(rankings.data.is_empty(), "response payload should parse");
    assert_eq!(rankings.meta.start_date, "2026-04-12");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/datasets/rankings-daily?start_date=2026-04-12&end_date=2026-05-11 HTTP/1.1"
    );

    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include API key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_app_rankings_path_query_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[],"meta":{"as_of":"2026-05-12T02:00:00Z","version":"v1","start_date":"2026-04-12","end_date":"2026-05-11"}}"#,
    );
    let params = AppRankingsParams::builder()
        .category("coding")
        .subcategory("cli-agent")
        .sort("trending")
        .start_date("2026-04-12")
        .end_date("2026-05-11")
        .limit(10)
        .offset(2)
        .build()
        .expect("app rankings params should build");

    let rankings = discovery::get_app_rankings(&base_url, "api-key", Some(&params))
        .await
        .expect("app rankings request should succeed");
    assert!(rankings.data.is_empty());

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/datasets/app-rankings?category=coding&subcategory=cli-agent&sort=trending&start_date=2026-04-12&end_date=2026-05-11&limit=10&offset=2 HTTP/1.1"
    );
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include API key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_benchmark_dataset_paths_and_queries() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[],"meta":{"as_of":"2026-06-03T12:00:00Z","version":"v1","source":"artificial-analysis","source_url":"https://artificialanalysis.ai","citation":"Source","model_count":0}}"#,
    );
    let aa = discovery::get_benchmarks_artificial_analysis(&base_url, "api-key", Some(25))
        .await
        .expect("AA benchmark request should succeed");
    assert!(aa.data.is_empty());
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture AA request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/datasets/benchmarks/artificial-analysis?max_results=25 HTTP/1.1"
    );
    server.join().expect("AA server thread should finish");

    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":[],"meta":{"as_of":"2026-06-03T12:00:00Z","version":"v1","source":"design-arena","source_url":"https://www.designarena.ai","citation":"Source","model_count":0,"arena":"models","category":"codecategories","elo_bounds":{"min":0,"max":0}}}"#,
    );
    let da = discovery::get_benchmarks_design_arena(
        &base_url,
        "api-key",
        Some("models"),
        Some("codecategories"),
        Some(20),
    )
    .await
    .expect("Design Arena benchmark request should succeed");
    assert!(da.data.is_empty());
    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture Design Arena request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/datasets/benchmarks/design-arena?arena=models&category=codecategories&max_results=20 HTTP/1.1"
    );
    server
        .join()
        .expect("Design Arena server thread should finish");
}

#[tokio::test]
async fn test_get_activity_without_date_uses_base_path() {
    let (base_url, rx, server) = spawn_json_server(r#"{"data":[]}"#);

    let items = discovery::get_activity(&base_url, "mgmt-key", None)
        .await
        .expect("activity request without date should succeed");
    assert!(items.is_empty(), "response payload should parse");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/activity HTTP/1.1");

    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer mgmt-key")
            || request_lower.contains("authorization:bearer mgmt-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}
