use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

use openrouter_rs::{
    api::models,
    error::{ApiErrorKind, OpenRouterError},
};
use surf::StatusCode;

fn spawn_error_server(
    status_line: &str,
    body: &str,
    request_id: Option<&str>,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let status_line = status_line.to_string();
    let body = body.to_string();
    let request_id = request_id.map(ToOwned::to_owned);

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

        let mut headers = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        );
        if let Some(request_id) = request_id {
            headers.push_str(&format!("x-request-id: {request_id}\r\n"));
        }
        headers.push_str("\r\n");
        let response = format!("{headers}{body}");
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    (format!("http://{addr}/api/v1"), server)
}

#[tokio::test]
async fn test_normalized_generic_api_error_shape() {
    let (base_url, server) = spawn_error_server(
        "429 Too Many Requests",
        r#"{"error":{"code":429,"message":"Rate limit exceeded"}}"#,
        Some("req_123"),
    );

    let result = models::list_models(&base_url, "test-key", None, None).await;
    let error = result.expect_err("request should fail");
    match error {
        OpenRouterError::Api(api_error) => {
            assert_eq!(api_error.status, StatusCode::TooManyRequests);
            assert_eq!(api_error.api_code, Some(429));
            assert_eq!(api_error.message, "Rate limit exceeded");
            assert_eq!(api_error.request_id.as_deref(), Some("req_123"));
            assert!(matches!(api_error.kind, ApiErrorKind::Generic));
            assert!(api_error.is_retryable());
        }
        other => panic!("expected Api error, got {other:?}"),
    }

    server
        .join()
        .expect("server thread should join in reasonable time");
}

#[tokio::test]
async fn test_normalized_provider_error_shape() {
    let (base_url, server) = spawn_error_server(
        "502 Bad Gateway",
        r#"{
            "error": {
                "code": 502,
                "message": "Provider overloaded",
                "metadata": {
                    "provider_name": "openai",
                    "raw": {"upstream_code":"E_OVERLOAD"}
                }
            }
        }"#,
        Some("req_provider"),
    );

    let result = models::list_models(&base_url, "test-key", None, None).await;
    let error = result.expect_err("request should fail");
    match error {
        OpenRouterError::Api(api_error) => {
            assert_eq!(api_error.status, StatusCode::BadGateway);
            assert_eq!(api_error.api_code, Some(502));
            assert_eq!(api_error.request_id.as_deref(), Some("req_provider"));
            assert!(api_error.is_retryable());
            match api_error.kind {
                ApiErrorKind::Provider { provider_name, raw } => {
                    assert_eq!(provider_name, "openai");
                    assert_eq!(
                        raw.get("upstream_code").and_then(|value| value.as_str()),
                        Some("E_OVERLOAD")
                    );
                }
                other => panic!("expected provider kind, got {other:?}"),
            }
        }
        other => panic!("expected Api error, got {other:?}"),
    }

    server
        .join()
        .expect("server thread should join in reasonable time");
}

#[tokio::test]
async fn test_normalized_moderation_error_shape() {
    let (base_url, server) = spawn_error_server(
        "400 Bad Request",
        r#"{
            "error": {
                "code": 400,
                "message": "Moderation blocked",
                "metadata": {
                    "reasons": ["hate"],
                    "flagged_input": "bad text",
                    "provider_name": "openai",
                    "model_slug": "gpt-4.1"
                }
            }
        }"#,
        Some("req_mod"),
    );

    let result = models::list_models(&base_url, "test-key", None, None).await;
    let error = result.expect_err("request should fail");
    match error {
        OpenRouterError::Api(api_error) => {
            assert_eq!(api_error.status, StatusCode::BadRequest);
            assert_eq!(api_error.api_code, Some(400));
            assert_eq!(api_error.request_id.as_deref(), Some("req_mod"));
            assert!(!api_error.is_retryable());
            match api_error.kind {
                ApiErrorKind::Moderation {
                    reasons,
                    flagged_input,
                    provider_name,
                    model_slug,
                } => {
                    assert_eq!(reasons, vec!["hate"]);
                    assert_eq!(flagged_input, "bad text");
                    assert_eq!(provider_name, "openai");
                    assert_eq!(model_slug, "gpt-4.1");
                }
                other => panic!("expected moderation kind, got {other:?}"),
            }
        }
        other => panic!("expected Api error, got {other:?}"),
    }

    server
        .join()
        .expect("server thread should join in reasonable time");
}

#[tokio::test]
async fn test_plain_text_error_is_still_normalized() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");

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

        let body = "upstream timeout";
        let response = format!(
            "HTTP/1.1 504 Gateway Timeout\r\nContent-Type: text/plain\r\nx-request-id: req_plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let result = models::list_models(&base_url, "test-key", None, None).await;
    let error = result.expect_err("request should fail");
    match error {
        OpenRouterError::Api(api_error) => {
            assert_eq!(api_error.status, StatusCode::GatewayTimeout);
            assert_eq!(api_error.api_code, Some(504));
            assert_eq!(api_error.request_id.as_deref(), Some("req_plain"));
            assert_eq!(api_error.message, "upstream timeout");
            assert!(matches!(api_error.kind, ApiErrorKind::Generic));
            assert!(api_error.is_retryable());
        }
        other => panic!("expected Api error, got {other:?}"),
    }

    server
        .join()
        .expect("server thread should join in reasonable time");
}
