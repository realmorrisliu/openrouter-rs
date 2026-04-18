//! # Axum Chat Gateway
//!
//! A minimal server-side integration example that keeps `OpenRouterClient`
//! in application state and exposes one HTTP endpoint for chat completions.
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
//! ```

use std::{env, error::Error};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    error::OpenRouterError,
    types::Role,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    client: OpenRouterClient,
    default_model: String,
}

#[derive(Debug, Deserialize)]
struct ChatGatewayRequest {
    prompt: String,
    #[serde(default)]
    system: Option<String>,
    #[serde(default)]
    model: Option<String>,
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
    let default_model =
        env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());

    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs axum gateway example")
        .build()?;

    let state = AppState {
        client,
        default_model,
    };

    let app = Router::new()
        .route("/healthz", get(healthcheck))
        .route("/chat", post(chat_completion))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    println!("listening on http://{bind_addr}");
    println!("GET  /healthz");
    println!("POST /chat");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthcheck() -> Json<HealthcheckResponse> {
    Json(HealthcheckResponse { ok: true })
}

async fn chat_completion(
    State(state): State<AppState>,
    Json(payload): Json<ChatGatewayRequest>,
) -> Result<Json<ChatGatewayResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut messages = Vec::new();
    if let Some(system) = payload.system {
        messages.push(Message::new(Role::System, system));
    }
    messages.push(Message::new(Role::User, payload.prompt));

    let model = payload.model.unwrap_or_else(|| state.default_model.clone());
    let request = ChatCompletionRequest::builder()
        .model(model)
        .messages(messages)
        .max_tokens(payload.max_tokens)
        .build()
        .map_err(map_sdk_error)?;

    let response = state
        .client
        .chat()
        .create(&request)
        .await
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

fn map_sdk_error(error: OpenRouterError) -> (StatusCode, Json<ErrorResponse>) {
    match error {
        OpenRouterError::KeyNotConfigured | OpenRouterError::ConfigError(_) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
        }
        OpenRouterError::Api(_) => error_response(StatusCode::BAD_GATEWAY, error.to_string()),
        OpenRouterError::HttpRequest(_) => {
            error_response(StatusCode::BAD_GATEWAY, error.to_string())
        }
        OpenRouterError::UninitializedFieldError(_)
        | OpenRouterError::Serialization(_)
        | OpenRouterError::Io(_)
        | OpenRouterError::Unknown(_) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
        }
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
