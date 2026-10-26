//! # Axum Chat Gateway
//!
//! A minimal server-side integration example that keeps `OpenRouterClient`
//! in application state and exposes JSON and SSE chat endpoints.
//!
//! ## Usage
//!
//! ```bash
//! export OPENROUTER_API_KEY=sk-or-v1-...
//! cargo run --example axum_chat_gateway
//!
//! curl -s http://127.0.0.1:3000/chat \
//!   -H 'content-type: application/json' \
//!   -d '{"prompt":"Summarize why Rust is a good fit for network services."}' | jq
//!
//! curl -N http://127.0.0.1:3000/chat/stream \
//!   -H 'content-type: application/json' \
//!   -d '{"prompt":"Write three short sentences about Rust."}'
//! ```

use std::{convert::Infallible, env, error::Error, time::Duration};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures_util::{Stream, StreamExt, stream};
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    error::OpenRouterError,
    types::{Effort, ReasoningConfig, Role, completion::FinishReason},
};
use serde::{Deserialize, Serialize};
use tokio::time::{Instant, timeout, timeout_at};

const TUTORIAL_MODEL: &str = "deepseek/deepseek-v4.1-flash";

#[derive(Clone)]
struct AppState {
    client: OpenRouterClient,
    default_model: String,
    request_timeout: Duration,
}

#[derive(Debug, Deserialize)]
struct ChatGatewayRequest {
    prompt: String,
    #[serde(default)]
    system: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    reasoning: Option<ReasoningConfig>,
    #[serde(default = "default_max_tokens")]
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ChatGatewayResponse {
    model: String,
    content: String,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct HealthcheckResponse {
    ok: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

fn default_max_tokens() -> u32 {
    512
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key =
        env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY environment variable not set");
    let bind_addr =
        env::var("OPENROUTER_AXUM_BIND").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let default_model = env::var("OPENROUTER_MODEL").unwrap_or_else(|_| TUTORIAL_MODEL.to_string());

    let http_client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()?;
    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_client(http_client)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs axum gateway example")
        .build()?;

    let state = AppState {
        client,
        default_model,
        request_timeout: Duration::from_secs(60),
    };

    let app = app(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    println!("listening on http://{bind_addr}");
    println!("GET  /healthz");
    println!("POST /chat");
    println!("POST /chat/stream");

    axum::serve(listener, app).await?;
    Ok(())
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthcheck))
        .route("/chat", post(chat_completion))
        .route("/chat/stream", post(stream_chat))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .with_state(state)
}

async fn healthcheck() -> Json<HealthcheckResponse> {
    Json(HealthcheckResponse { ok: true })
}

async fn chat_completion(
    State(state): State<AppState>,
    Json(payload): Json<ChatGatewayRequest>,
) -> Result<Json<ChatGatewayResponse>, (StatusCode, Json<ErrorResponse>)> {
    let request = build_request(&state, payload)?;
    let response = timeout(state.request_timeout, state.client.chat().create(&request))
        .await
        .map_err(|_| error_response(StatusCode::GATEWAY_TIMEOUT, "Upstream timed out"))?
        .map_err(map_sdk_error)?;

    let choice = response
        .choices
        .first()
        .ok_or_else(|| error_response(StatusCode::BAD_GATEWAY, "OpenRouter returned no choices"))?;

    Ok(Json(ChatGatewayResponse {
        model: response.model,
        content: choice.content().unwrap_or("").to_string(),
        finish_reason: choice.finish_reason().map(|reason| format!("{reason:?}")),
    }))
}

fn build_request(
    state: &AppState,
    payload: ChatGatewayRequest,
) -> Result<ChatCompletionRequest, (StatusCode, Json<ErrorResponse>)> {
    if payload.prompt.trim().is_empty()
        || payload.prompt.len() + payload.system.as_deref().unwrap_or("").len() > 8 * 1024
        || !(1..=2048).contains(&payload.max_tokens)
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Provide a nonempty prompt, at most 8192 prompt/system bytes, and max_tokens in 1..=2048",
        ));
    }
    let mut messages = Vec::new();
    if let Some(system) = payload.system {
        messages.push(Message::new(Role::System, system));
    }
    messages.push(Message::new(Role::User, payload.prompt));

    let model = payload.model.unwrap_or_else(|| state.default_model.clone());
    let mut builder = ChatCompletionRequest::builder();
    // Explicit caller configuration takes precedence over the tutorial default.
    if let Some(reasoning) = payload.reasoning {
        builder.reasoning(reasoning);
    } else if model == TUTORIAL_MODEL {
        builder.reasoning_effort(Effort::None);
    }
    builder
        .model(model)
        .messages(messages)
        .max_tokens(payload.max_tokens)
        .build()
        .map_err(map_sdk_error)
}

async fn stream_chat(
    State(state): State<AppState>,
    Json(payload): Json<ChatGatewayRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, Json<ErrorResponse>)> {
    let deadline = Instant::now() + state.request_timeout;
    let request = build_request(&state, payload)?;
    let upstream = timeout_at(deadline, state.client.chat().stream(&request))
        .await
        .map_err(|_| error_response(StatusCode::GATEWAY_TIMEOUT, "Upstream timed out"))?
        .map_err(map_sdk_error)?;

    // The response body owns the upstream stream; no detached task drains it.
    let events = stream::unfold(Some((upstream, false)), move |state| async move {
        let (mut upstream, mut finished) = state?;
        let (event, next) = match timeout_at(deadline, upstream.next()).await {
            Ok(Some(Ok(chunk))) => {
                if chunk.choices.iter().any(|choice| {
                    choice.error().is_some()
                        || matches!(choice.finish_reason(), Some(FinishReason::Error))
                }) {
                    (
                        Event::default()
                            .event("error")
                            .data("upstream_stream_failed"),
                        None,
                    )
                } else {
                    finished |= chunk
                        .choices
                        .iter()
                        .any(|choice| choice.finish_reason().is_some());
                    match Event::default().event("chunk").json_data(&chunk) {
                        Ok(event) => (event, Some((upstream, finished))),
                        Err(_) => (
                            Event::default().event("error").data("serialization_failed"),
                            None,
                        ),
                    }
                }
            }
            Ok(Some(Err(_))) => (
                Event::default()
                    .event("error")
                    .data("upstream_stream_failed"),
                None,
            ),
            Ok(None) if finished => (Event::default().event("done").data("{}"), None),
            Ok(None) => (Event::default().event("error").data("unexpected_eof"), None),
            Err(_) => (
                Event::default().event("error").data("upstream_timeout"),
                None,
            ),
        };
        Some((Ok(event), next))
    });

    Ok(Sse::new(events).keep_alive(KeepAlive::default()))
}

fn map_sdk_error(error: OpenRouterError) -> (StatusCode, Json<ErrorResponse>) {
    match error {
        OpenRouterError::KeyNotConfigured | OpenRouterError::ConfigError(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Gateway configuration or request error",
        ),
        OpenRouterError::Api(_) => {
            error_response(StatusCode::BAD_GATEWAY, "OpenRouter request failed")
        }
        OpenRouterError::HttpRequest(_) => {
            error_response(StatusCode::BAD_GATEWAY, "OpenRouter request failed")
        }
        OpenRouterError::UninitializedFieldError(_)
        | OpenRouterError::Serialization(_)
        | OpenRouterError::Io(_)
        | OpenRouterError::Unknown(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Gateway configuration or request error",
        ),
        // The error enum is non-exhaustive, so a future variant must still map
        // to a safe response.
        _ => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Gateway configuration or request error",
        ),
    }
}

fn error_response(
    status: StatusCode,
    message: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: message.into(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, response::IntoResponse};
    use serde_json::{Value, json};
    use std::sync::Arc;
    use tokio::{sync::Notify, task::JoinHandle};

    struct Server {
        url: String,
        task: JoinHandle<()>,
    }

    impl Drop for Server {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    async fn serve(router: Router) -> Server {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        Server { url, task }
    }

    async fn gateway(upstream: &Server, duration: Duration) -> Server {
        serve(app(AppState {
            client: OpenRouterClient::builder()
                .api_key("test-key")
                .base_url(&upstream.url)
                .http_client(reqwest::Client::builder().no_proxy().build().unwrap())
                .build()
                .unwrap(),
            default_model: TUTORIAL_MODEL.into(),
            request_timeout: duration,
        }))
        .await
    }

    async fn request(server: &Server, path: &str, body: Value) -> reqwest::Response {
        reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap()
            .post(format!("{}{path}", server.url))
            .json(&body)
            .send()
            .await
            .unwrap()
    }

    fn frame(content: &str, reason: Option<&str>, usage: Option<Value>) -> String {
        format!(
            "data: {}\n\n",
            json!({"id":"mock-generation", "object":"chat.completion.chunk", "created":1,
                "model":"mock", "choices":[{"index":0,"delta":{"content":content},"finish_reason":reason}],
                "usage":usage})
        )
    }

    async fn fixed_upstream(body: String) -> Server {
        serve(Router::new().route(
            "/chat/completions",
            post(move || {
                let body = body.clone();
                async move { ([("content-type", "text/event-stream")], body) }
            }),
        ))
        .await
    }

    #[test]
    fn reasoning_policy_follows_the_effective_model() {
        for (default_model, override_model, expected_model, disabled) in [
            (TUTORIAL_MODEL, None, TUTORIAL_MODEL, true),
            (
                TUTORIAL_MODEL,
                Some("openai/o3-mini"),
                "openai/o3-mini",
                false,
            ),
            ("openai/o3-mini", None, "openai/o3-mini", false),
            ("openai/o3-mini", Some(TUTORIAL_MODEL), TUTORIAL_MODEL, true),
        ] {
            let state = AppState {
                client: OpenRouterClient::builder()
                    .api_key("test-key")
                    .build()
                    .unwrap(),
                default_model: default_model.into(),
                request_timeout: Duration::from_secs(60),
            };
            let payload = ChatGatewayRequest {
                prompt: "hello".into(),
                system: None,
                model: override_model.map(str::to_string),
                reasoning: None,
                max_tokens: 128,
            };
            let request = serde_json::to_value(build_request(&state, payload).unwrap()).unwrap();
            assert_eq!(request["model"], expected_model);
            if disabled {
                assert_eq!(request["reasoning"], json!({"effort":"none"}));
            } else {
                assert!(request.get("reasoning").is_none(), "{request}");
            }
        }
    }

    #[tokio::test]
    async fn explicit_reasoning_reaches_both_routes() {
        for reasoning in [json!({"effort":"low"}), json!({"enabled":true}), json!({})] {
            let expected = reasoning.clone();
            let upstream = serve(Router::new().route("/chat/completions", post(move |Json(payload): Json<Value>| {
                assert_eq!(payload["reasoning"], expected);
                async move {
                    if payload["stream"] == true {
                        ([("content-type", "text/event-stream")], frame("hi", Some("stop"), None)).into_response()
                    } else {
                        Json(json!({"id":"mock", "object":"chat.completion", "created":1,
                            "model":"mock", "choices":[{"index":0,"message":{"role":"assistant","content":"hi"},"finish_reason":"stop"}]})).into_response()
                    }
                }
            }))).await;
            let gateway = gateway(&upstream, Duration::from_secs(2)).await;
            for path in ["/chat", "/chat/stream"] {
                let response = request(
                    &gateway,
                    path,
                    json!({"prompt":"hi", "reasoning":reasoning}),
                )
                .await;
                assert_eq!(response.status(), StatusCode::OK);
                assert!(response.text().await.unwrap().contains("hi"));
            }
        }
    }

    #[tokio::test]
    async fn streams_before_completion_and_keeps_final_usage() {
        let release = Arc::new(Notify::new());
        let producer = release.clone();
        let upstream = serve(Router::new().route(
            "/chat/completions",
            post(move |Json(payload): Json<Value>| {
                assert_eq!(payload["stream"], true);
                assert_eq!(payload["reasoning"]["effort"], "none");
                let release = producer.clone();
                async move {
                    let frames = vec![
                        frame("hello", None, None),
                        frame(" world", Some("stop"), None),
                        frame("", Some("stop"), Some(json!({
                            "prompt_tokens":3,"completion_tokens":2,"total_tokens":5,"cost":0.001
                        }))),
                        "data: [DONE]\n\n".into(),
                    ];
                    let body = stream::iter(frames.into_iter().enumerate()).then(move |(i, frame)| {
                        let release = release.clone();
                        async move {
                            if i == 1 {
                                release.notified().await;
                            }
                            Ok::<_, Infallible>(frame)
                        }
                    });
                    ([("content-type", "text/event-stream")], Body::from_stream(body))
                }
            }),
        ))
        .await;
        let gateway = gateway(&upstream, Duration::from_secs(4)).await;
        let mut response = request(&gateway, "/chat/stream", json!({"prompt":"hello"})).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/event-stream")
        );
        let mut text = String::new();
        timeout(Duration::from_secs(2), async {
            while !text.contains("hello") {
                text.push_str(
                    std::str::from_utf8(&response.chunk().await.unwrap().unwrap()).unwrap(),
                );
            }
        })
        .await
        .expect("first chunk must arrive while upstream completion is blocked");
        assert!(!text.contains("event: done"));
        release.notify_one();
        text.push_str(&response.text().await.unwrap());
        assert_eq!(text.matches("event: chunk").count(), 3);
        assert!(text.contains("\"cost\":0.001"));
        assert!(text.contains("event: done"));
        assert!(!text.contains("event: error"));
    }

    #[tokio::test]
    async fn upstream_http_error_is_sanitized() {
        let upstream = serve(Router::new().route(
            "/chat/completions",
            post(|| async {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error":{"code":401,"message":"private upstream detail"}})),
                )
            }),
        ))
        .await;
        let gateway = gateway(&upstream, Duration::from_secs(2)).await;
        for path in ["/chat", "/chat/stream"] {
            let response = request(&gateway, path, json!({"prompt":"hello"})).await;
            assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
            let body = response.text().await.unwrap();
            assert!(!body.contains("private upstream detail"));
            assert!(body.contains("OpenRouter request failed"));
        }
    }

    #[tokio::test]
    async fn midstream_errors_and_premature_eof_are_not_success() {
        for (body, code) in [
            (
                frame("private detail", Some("error"), None),
                "upstream_stream_failed",
            ),
            ("data: {malformed}\n\n".into(), "upstream_stream_failed"),
            (frame("partial", None, None), "unexpected_eof"),
        ] {
            let upstream = fixed_upstream(body).await;
            let gateway = gateway(&upstream, Duration::from_secs(2)).await;
            let response = request(&gateway, "/chat/stream", json!({"prompt":"hello"})).await;
            assert_eq!(response.status(), StatusCode::OK);
            let text = response.text().await.unwrap();
            assert!(text.contains("event: error"));
            assert!(text.contains(code));
            assert!(!text.contains("event: done"));
            assert!(!text.contains("private detail"));
        }
    }

    #[tokio::test]
    async fn bounds_input_before_contacting_upstream() {
        let upstream = serve(Router::new()).await;
        let gateway = gateway(&upstream, Duration::from_secs(2)).await;
        for path in ["/chat", "/chat/stream"] {
            for payload in [
                json!({"prompt":" "}),
                json!({"prompt":"hi","max_tokens":0}),
                json!({"prompt":"hi","max_tokens":2049}),
                json!({"prompt":"x".repeat(8193)}),
            ] {
                assert_eq!(
                    request(&gateway, path, payload).await.status(),
                    StatusCode::BAD_REQUEST
                );
            }
            assert_eq!(
                request(&gateway, path, json!({"prompt":"x".repeat(17000)}))
                    .await
                    .status(),
                StatusCode::PAYLOAD_TOO_LARGE
            );
        }
    }

    #[tokio::test]
    async fn header_and_stream_timeouts_are_bounded() {
        let upstream = serve(Router::new().route(
            "/chat/completions",
            post(|| async {
                std::future::pending::<()>().await;
                StatusCode::OK
            }),
        ))
        .await;
        let gateway = gateway(&upstream, Duration::from_millis(100)).await;
        for path in ["/chat", "/chat/stream"] {
            assert_eq!(
                request(&gateway, path, json!({"prompt":"hi"}))
                    .await
                    .status(),
                StatusCode::GATEWAY_TIMEOUT
            );
        }
        let upstream = serve(Router::new().route(
            "/chat/completions",
            post(|| async {
                let body = stream::once(async { Ok::<_, Infallible>(frame("hi", None, None)) })
                    .chain(stream::pending());
                (
                    [("content-type", "text/event-stream")],
                    Body::from_stream(body),
                )
            }),
        ))
        .await;
        let gateway = self::gateway(&upstream, Duration::from_millis(100)).await;
        let text = request(&gateway, "/chat/stream", json!({"prompt":"hi"}))
            .await
            .text()
            .await
            .unwrap();
        assert!(text.contains("upstream_timeout"));
        assert!(!text.contains("event: done"));
    }

    #[tokio::test]
    async fn downstream_disconnect_drops_upstream_response_body() {
        struct DropSignal(Arc<Notify>);
        impl Drop for DropSignal {
            fn drop(&mut self) {
                self.0.notify_one();
            }
        }
        let dropped = Arc::new(Notify::new());
        let signal = dropped.clone();
        let upstream = serve(Router::new().route(
            "/chat/completions",
            post(move || {
                let guard = DropSignal(signal.clone());
                async move {
                    let body = stream::unfold(guard, |guard| async move {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        Some((Ok::<_, Infallible>(frame("hi", None, None)), guard))
                    });
                    (
                        [("content-type", "text/event-stream")],
                        Body::from_stream(body),
                    )
                        .into_response()
                }
            }),
        ))
        .await;
        let gateway = gateway(&upstream, Duration::from_secs(4)).await;
        let mut response = request(&gateway, "/chat/stream", json!({"prompt":"hi"})).await;
        assert!(response.chunk().await.unwrap().is_some());
        drop(response);
        timeout(Duration::from_secs(2), dropped.notified())
            .await
            .expect("disconnect must release the upstream HTTP body before the request deadline");
    }
}
