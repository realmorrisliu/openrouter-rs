//! Transport failure classification and `Retry-After` propagation.

use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use openrouter_rs::{
    OpenRouterClient,
    error::{HttpRequestError, HttpRequestErrorKind, OpenRouterError},
};

/// Serves one fixed HTTP response, then closes the connection.
fn spawn_response(raw_response: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("listener should have addr");
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0_u8; 1024];
            let _ = stream.read(&mut buffer);
            let _ = stream.write_all(raw_response.as_bytes());
        }
    });
    format!("http://{addr}/api/v1")
}

fn client(base_url: &str) -> OpenRouterClient {
    OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("test-key")
        .build()
        .expect("client should build")
}

#[tokio::test]
async fn rate_limit_exposes_the_server_retry_delay() {
    let base_url = spawn_response(
        "HTTP/1.1 429 Too Many Requests\r\ncontent-type: application/json\r\nretry-after: 7\r\ncontent-length: 43\r\nconnection: close\r\n\r\n{\"error\":{\"code\":429,\"message\":\"slow\"}}",
    );
    let error = client(&base_url).models().list().await.unwrap_err();

    match error {
        OpenRouterError::Api(context) => {
            assert!(context.is_retryable(), "429 should be retryable");
            assert_eq!(context.retry_after, Some(Duration::from_secs(7)));
        }
        other => panic!("expected an API error, got {other:?}"),
    }
}

#[tokio::test]
async fn http_date_retry_after_is_reported_as_absent() {
    let base_url = spawn_response(
        "HTTP/1.1 429 Too Many Requests\r\ncontent-type: application/json\r\nretry-after: Wed, 21 Oct 2015 07:28:00 GMT\r\ncontent-length: 43\r\nconnection: close\r\n\r\n{\"error\":{\"code\":429,\"message\":\"slow\"}}",
    );
    let error = client(&base_url).models().list().await.unwrap_err();

    match error {
        OpenRouterError::Api(context) => assert_eq!(
            context.retry_after, None,
            "the HTTP-date form must not be guessed"
        ),
        other => panic!("expected an API error, got {other:?}"),
    }
}

#[tokio::test]
async fn non_retryable_errors_carry_no_delay() {
    let base_url = spawn_response(
        "HTTP/1.1 400 Bad Request\r\ncontent-type: application/json\r\ncontent-length: 43\r\nconnection: close\r\n\r\n{\"error\":{\"code\":400,\"message\":\"bad\"}}",
    );
    let error = client(&base_url).models().list().await.unwrap_err();

    match error {
        OpenRouterError::Api(context) => {
            assert!(!context.is_retryable());
            assert_eq!(context.retry_after, None);
        }
        other => panic!("expected an API error, got {other:?}"),
    }
}

#[tokio::test]
async fn a_refused_connection_is_classified_as_connect() {
    // Bind then drop, so the port is closed when the request runs.
    let addr = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        listener.local_addr().expect("listener should have addr")
    };

    let error = client(&format!("http://{addr}/api/v1"))
        .models()
        .list()
        .await
        .unwrap_err();

    match error {
        OpenRouterError::HttpRequest(http_error) => {
            assert_eq!(http_error.kind(), HttpRequestErrorKind::Connect);
            assert!(http_error.is_connect());
            assert!(!http_error.is_timeout());
            assert!(
                http_error
                    .url()
                    .is_some_and(|url| url.contains("127.0.0.1")),
                "captured url: {:?}",
                http_error.url()
            );
        }
        other => panic!("expected an HTTP request error, got {other:?}"),
    }
}

#[tokio::test]
async fn an_expired_deadline_is_classified_as_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("listener should have addr");
    thread::spawn(move || {
        // Accept and hold the connection open without responding.
        if let Ok((_stream, _)) = listener.accept() {
            thread::sleep(Duration::from_secs(5));
        }
    });

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_millis(50))
        .build()
        .expect("http client should build");
    let client = OpenRouterClient::builder()
        .base_url(format!("http://{addr}/api/v1"))
        .api_key("test-key")
        .http_client(http_client)
        .build()
        .expect("client should build");

    let error = client.vault().list().await.unwrap_err();

    match error {
        OpenRouterError::HttpRequest(http_error) => {
            assert_eq!(http_error.kind(), HttpRequestErrorKind::Timeout);
            assert!(http_error.is_timeout());
            assert!(!http_error.is_connect());
        }
        other => panic!("expected an HTTP request error, got {other:?}"),
    }
}

#[test]
fn classification_is_comparable_and_cloneable() {
    let timeout = HttpRequestError::classified(HttpRequestErrorKind::Timeout, "slow");
    let clone = timeout.clone();
    assert_eq!(timeout, clone);
    assert_ne!(
        timeout,
        HttpRequestError::classified(HttpRequestErrorKind::Connect, "slow")
    );
}
