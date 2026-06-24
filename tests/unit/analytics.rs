use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::api::analytics::{
    self, AnalyticsFilter, AnalyticsFilterValue, AnalyticsMeta, AnalyticsOrderBy,
    AnalyticsQueryRequest, AnalyticsQueryResponse, AnalyticsTimeRange,
};
use serde_json::json;

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
fn test_analytics_meta_response_deserializes() {
    let raw = r#"{
        "data": {
            "metrics": [{
                "name": "request_count",
                "display_label": "Request Count",
                "is_rate": false,
                "display_format": "number"
            }],
            "dimensions": [{
                "name": "model",
                "display_label": "Model"
            }],
            "operators": [{
                "name": "eq",
                "value_type": "scalar"
            }],
            "granularities": [{
                "name": "day",
                "display_label": "Day"
            }]
        }
    }"#;

    let parsed: openrouter_rs::types::ApiResponse<AnalyticsMeta> =
        serde_json::from_str(raw).expect("analytics meta should deserialize");
    assert_eq!(parsed.data.metrics[0].name, "request_count");
    assert_eq!(parsed.data.granularities[0].name, "day");
}

#[test]
fn test_analytics_query_response_deserializes() {
    let raw = r#"{
        "data": {
            "cachedAt": 1780000000.0,
            "data": [{
                "date__day": "2026-06-15T00:00:00.000Z",
                "request_count": 1500
            }],
            "metadata": {
                "query_time_ms": 42,
                "row_count": 1,
                "truncated": false
            },
            "warnings": ["unresolved api_key_id hash"]
        }
    }"#;

    let parsed: openrouter_rs::types::ApiResponse<AnalyticsQueryResponse> =
        serde_json::from_str(raw).expect("analytics query response should deserialize");
    assert_eq!(parsed.data.metadata.row_count, 1);
    assert_eq!(parsed.data.data[0]["request_count"], json!(1500));
    assert_eq!(
        parsed.data.warnings.as_deref(),
        Some(["unresolved api_key_id hash".to_string()].as_slice())
    );
}

#[tokio::test]
async fn test_get_analytics_meta_path_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"metrics":[],"dimensions":[],"operators":[],"granularities":[]}}"#,
    );

    let meta = analytics::get_analytics_meta(&base_url, "management-key")
        .await
        .expect("analytics meta request should succeed");
    assert!(meta.metrics.is_empty());

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/analytics/meta HTTP/1.1");
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer management-key")
            || request_lower.contains("authorization:bearer management-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_query_analytics_path_body_and_auth_header() {
    let (base_url, rx, server) = spawn_json_server(
        r#"{"data":{"data":[],"metadata":{"query_time_ms":5,"row_count":0,"truncated":false}}}"#,
    );
    let request = AnalyticsQueryRequest::builder()
        .metrics(vec!["request_count".to_string()])
        .dimensions(vec!["model".to_string()])
        .filters(vec![AnalyticsFilter {
            field: "model".to_string(),
            operator: "eq".to_string(),
            value: AnalyticsFilterValue::String("openai/gpt-5".to_string()),
        }])
        .granularity("day")
        .time_range(AnalyticsTimeRange {
            start: "2026-06-01T00:00:00Z".to_string(),
            end: "2026-06-15T00:00:00Z".to_string(),
        })
        .order_by(AnalyticsOrderBy {
            field: "request_count".to_string(),
            direction: "desc".to_string(),
        })
        .limit(100)
        .group_limit(10)
        .build()
        .expect("analytics query request should build");

    let response = analytics::query_analytics(&base_url, "management-key", &request)
        .await
        .expect("analytics query request should succeed");
    assert_eq!(response.metadata.row_count, 0);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "POST /api/v1/analytics/query HTTP/1.1"
    );
    let body: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("request body should be json");
    assert_eq!(body["metrics"][0], "request_count");
    assert_eq!(body["dimensions"][0], "model");
    assert_eq!(body["filters"][0]["value"], "openai/gpt-5");
    assert_eq!(body["time_range"]["start"], "2026-06-01T00:00:00Z");

    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer management-key")
            || request_lower.contains("authorization:bearer management-key"),
        "authorization header should include management key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}
