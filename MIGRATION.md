# Migration Guide

This document keeps historical migration guides intact because the repo still smoke-tests older domain/naming transitions.

> Latest breaking release target: `0.9.x -> 0.10.0`.
> `0.10.0` makes high-churn public SDK model types non-exhaustive so additive upstream OpenRouter fields and taxonomy values do not keep causing accidental source breaks.

## Latest: 0.9.x -> 0.10.0

This release intentionally future-proofs public SDK model types that mirror upstream request, response, metadata, usage, pricing, discovery, streaming, and taxonomy shapes. The runtime JSON behavior is unchanged, but Rust source that constructed affected public structs with literals or matched affected public enums exhaustively may need small edits.

### Quick Checklist For 0.10.0

- Replace direct struct literals for affected SDK model types with builders, constructors, helpers, or serde deserialization.
- Add a wildcard arm (`_ => ...`) when matching affected public enums such as stream events, choices, provider taxonomies, finish reasons, and embedding vector variants outside the crate.
- Prefer request builders for caller-built request types: `ChatCompletionRequest::builder()`, `ResponsesRequest::builder()`, `AnthropicMessagesRequest::builder()`, `EmbeddingRequest::builder()`, and similar endpoint builders.
- Use new constructors where direct literals were mainly test/helper code, including `ResponseUsage::new(...)`, `ToolCall::new(...)`, `FunctionCall::new(...)`, `JsonSchemaConfig::new(...)`, `EmbeddingContentPart::text(...)`, `EmbeddingContentPart::image_url(...)`, and provider-options `new(...)` helpers.
- Keep this migration with the `0.10.0` release boundary; it is broader than the 2026-04-29 `ResponseUsage` cost-field drift.

### Breaking-Change Mapping For 0.10.0

| Area | Old Usage (`0.9.x`) | New Usage (`0.10.0`) |
| --- | --- | --- |
| Response usage test data | `ResponseUsage { prompt_tokens, completion_tokens, total_tokens, ... }` | `ResponseUsage::new(prompt_tokens, completion_tokens, total_tokens)` |
| Tool-call test data | `ToolCall { id, type_, function, index }` | `ToolCall::new(id, name, arguments).with_index(index)` |
| Request construction | `EmbeddingRequest { model, input, ... }` | `EmbeddingRequest::builder().model(model).input(input).build()?` |
| Provider preferences | `ProviderPreferences { sort, order, ... }` | `let mut prefs = ProviderPreferences::default(); prefs.sort = Some(...);` |
| Public enum matching | `match event { StreamEvent::Done { .. } => ..., StreamEvent::Error(e) => ... }` | Add `_ => {}` or another fallback arm |
| Response fixtures | Public response struct literals | Deserialize JSON fixtures with `serde_json::from_value(...)` or use documented constructors |

### Example: response fixture construction

Before:

```rust
use openrouter_rs::types::completion::ResponseUsage;

let usage = ResponseUsage {
    prompt_tokens: 5,
    completion_tokens: 7,
    total_tokens: 12,
    cost: None,
    cost_details: None,
    is_byok: None,
};
```

After:

```rust
use openrouter_rs::types::completion::ResponseUsage;

let usage = ResponseUsage::new(5, 7, 12);
```

### Example: non-exhaustive enum matching

Before:

```rust
use openrouter_rs::types::stream::StreamEvent;

fn handle(event: StreamEvent) {
    match event {
        StreamEvent::ContentDelta(text) => print!("{text}"),
        StreamEvent::Done { .. } => println!("done"),
        StreamEvent::Error(error) => eprintln!("{error}"),
    }
}
```

After:

```rust
use openrouter_rs::types::stream::StreamEvent;

fn handle(event: StreamEvent) {
    match event {
        StreamEvent::ContentDelta(text) => print!("{text}"),
        StreamEvent::Done { .. } => println!("done"),
        StreamEvent::Error(error) => eprintln!("{error}"),
        _ => {}
    }
}
```

## Previous: 0.8.x -> 0.9.0

If you use the new workspace, generation metadata, video generation, or audio speech surfaces, prefer request builders and the canonical domain clients. Compatibility aliases remain for the old `tts` names, but new code should use `audio`.

### Quick Checklist For 0.9.0

- Replace `client.tts().create(...)` with `client.audio().speech().create(...)`.
- Replace `api::tts::TtsRequest` with `api::audio::SpeechRequest`.
- Replace `api::tts::TtsResponseFormat` with `api::audio::SpeechResponseFormat`.
- Replace direct struct literals for newly added/high-churn request types with their builders, especially workspace and video-generation requests.
- For CLI workspace I/O logging controls, use `workspaces create|update --io-logging-api-key-id`, `--io-logging-sampling-rate`, and `workspaces update --clear-io-logging-api-key-ids`.

### Breaking-Change Mapping For 0.9.0

| Area | Old Usage (`0.8.x`) | New Usage (`0.9.0`) |
| --- | --- | --- |
| Audio speech domain | `client.tts().create(&request)` | `client.audio().speech().create(&request)` |
| Audio speech imports | `api::tts::{TtsRequest, TtsResponseFormat}` | `api::audio::{SpeechRequest, SpeechResponseFormat}` |
| Audio speech flat helper | `api::tts::create_tts(...)` | `api::audio::create_speech(...)` |
| High-churn request construction | `UpdateWorkspaceRequest { ... }` | `UpdateWorkspaceRequest::builder().name(...).build()?` |
| Workspace CLI key filters | Not exposed | `workspaces create|update --io-logging-api-key-id 101` |
| Workspace CLI sampling rate | Not exposed | `workspaces create|update --io-logging-sampling-rate 0.25` |
| Workspace CLI clear key filters | SDK-only wrapper | `workspaces update ws_123 --clear-io-logging-api-key-ids` |

### Example: audio speech rename

Before:

```rust
use openrouter_rs::{
    api::tts::{TtsRequest, TtsResponseFormat},
    OpenRouterClient,
};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = OpenRouterClient::builder().api_key("sk-or-v1-...").build()?;
let request = TtsRequest::builder()
    .model("elevenlabs/eleven-turbo-v2")
    .input("Hello")
    .voice("alloy")
    .response_format(TtsResponseFormat::Mp3)
    .build()?;
let audio = client.tts().create(&request).await?;
# let _ = audio;
# Ok(())
# }
```

After:

```rust
use openrouter_rs::{
    api::audio::{SpeechRequest, SpeechResponseFormat},
    OpenRouterClient,
};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = OpenRouterClient::builder().api_key("sk-or-v1-...").build()?;
let request = SpeechRequest::builder()
    .model("elevenlabs/eleven-turbo-v2")
    .input("Hello")
    .voice("alloy")
    .response_format(SpeechResponseFormat::Mp3)
    .build()?;
let audio = client.audio().speech().create(&request).await?;
# let _ = audio;
# Ok(())
# }
```

### Example: builder-first workspace updates

Before:

```rust
use openrouter_rs::api::workspaces::UpdateWorkspaceRequest;

let request = UpdateWorkspaceRequest {
    name: Some("Platform".to_string()),
    slug: None,
    description: None,
    default_text_model: None,
    default_image_model: None,
    default_provider_sort: None,
    io_logging_api_key_ids: Some(vec![101]),
    io_logging_sampling_rate: Some(0.25),
    is_data_discount_logging_enabled: None,
    is_observability_broadcast_enabled: None,
    is_observability_io_logging_enabled: None,
};
```

After:

```rust
use openrouter_rs::api::workspaces::UpdateWorkspaceRequest;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let request = UpdateWorkspaceRequest::builder()
    .name("Platform")
    .io_logging_api_key_ids(vec![101])
    .io_logging_sampling_rate(0.25)
    .build()?;
# let _ = request;
# Ok(())
# }
```

## Earlier: 0.7.x -> 0.8.0

If you already use the canonical domain clients (`chat()`, `responses()`, `messages()`, `models()`, `management()`, and opt-in `legacy()`), this upgrade is usually just a version bump plus a recompile. The breaking changes mainly affect callers that coupled themselves to the old transport-facing error and utility surface.

### Quick Checklist For 0.8.0

- Replace `OpenRouterError::HttpRequest(surf::Error)` handling with backend-neutral `OpenRouterError::HttpRequest(HttpRequestError)` handling.
- Replace `surf::StatusCode` comparisons with `http::StatusCode`.
- Stop importing `openrouter_rs::utils::{with_bearer_auth, with_request_metadata, with_client_request_headers, handle_error}`.
- Keep any custom auth/header/request glue in your own application code, or prefer the canonical domain clients directly.

### Breaking-Change Mapping For 0.8.0

| Area | Old Usage (`0.7.x` and earlier) | New Usage (`0.8.0`) |
| --- | --- | --- |
| HTTP request errors | `OpenRouterError::HttpRequest(surf::Error)` | `OpenRouterError::HttpRequest(HttpRequestError)` |
| API status comparisons | `api_error.status == surf::StatusCode::BadRequest` | `api_error.status == http::StatusCode::BAD_REQUEST` |
| Public transport helpers | `openrouter_rs::utils::with_bearer_auth(...)`, `with_request_metadata(...)`, `with_client_request_headers(...)`, `handle_error(...)` | Use `OpenRouterClient` domain methods or caller-owned transport/header helpers |
| SDK transport stack | `surf -> isahc -> curl` | `reqwest + rustls` |

### Example: HTTP request error matching

Before:

```rust
use openrouter_rs::error::OpenRouterError;

fn describe(error: OpenRouterError) {
    match error {
        OpenRouterError::HttpRequest(surf_error) => {
            eprintln!("transport failed: {}", surf_error);
        }
        other => eprintln!("{other}"),
    }
}
```

After:

```rust
use openrouter_rs::error::OpenRouterError;

fn describe(error: OpenRouterError) {
    match error {
        OpenRouterError::HttpRequest(request_error) => {
            eprintln!("transport failed: {}", request_error);
        }
        other => eprintln!("{other}"),
    }
}
```

### Example: API status comparisons

Before:

```rust
if api_error.status == surf::StatusCode::BadRequest {
    eprintln!("bad request");
}
```

After:

```rust
if api_error.status == http::StatusCode::BAD_REQUEST {
    eprintln!("bad request");
}
```

### Example: removing `utils` transport imports

Before:

```rust
use openrouter_rs::utils::{
    handle_error,
    with_bearer_auth,
    with_client_request_headers,
    with_request_metadata,
};
```

After:

```rust
use openrouter_rs::OpenRouterClient;

// Prefer the SDK's domain clients, or keep any custom transport glue local
// to your application instead of importing transport-specific SDK utilities.
```

## Historical: 0.5.x -> 0.6.0

This guide is for users upgrading from the `0.5.x` API surface to `0.6.0`.
All "Before" snippets in this document represent compatibility aliases removed in `0.6.0`.

## Quick Checklist

1. Replace `provisioning_key` naming with `management_key` (`OpenRouterClientBuilder::management_key`, `OpenRouterClient::set_management_key`, `OpenRouterClient::clear_management_key`).
2. Move legacy completions usage to `api::legacy::completion` and `client.legacy().completions()`.
3. Move from flat client calls to domain clients (`chat()`, `responses()`, `messages()`, `models()`, `management()`).
4. Replace old pagination shapes with `types::PaginationOptions`.
5. Update renamed domain methods (`count`, `list_for_user`, `exchange_code_for_api_key`).

## Removed in 0.6.0

These compatibility aliases were removed and must be migrated:

| Removed API | Replacement |
| --- | --- |
| `OpenRouterClientBuilder::provisioning_key(...)` | `OpenRouterClientBuilder::management_key(...)` |
| `OpenRouterClient::set_provisioning_key(...)` | `OpenRouterClient::set_management_key(...)` |
| `OpenRouterClient::clear_provisioning_key()` | `OpenRouterClient::clear_management_key()` |
| `OpenRouterClient::list_api_keys(Some(offset_f64), include_disabled)` | `client.management().list_api_keys(Some(PaginationOptions::with_offset(offset_u32)), include_disabled)` |
| `ManagementClient::exchange_code_for_api_key(...)` | `ManagementClient::create_api_key_from_auth_code(...)` |
| `api::completion::*` | `api::legacy::completion::*` |
| `OpenRouterClient::send_completion_request(...)` | `client.legacy().completions().create(...)` |
| `ModelsClient::list_for_user()` | `ModelsClient::list_user_models()` |
| `ModelsClient::count()` | `ModelsClient::get_model_count()` |

## Breaking-Change Mapping

| Area | Old Usage | New Usage (`0.6.0` target) |
| --- | --- | --- |
| Management key naming | `builder().provisioning_key(...)` | `builder().management_key(...)` |
| Runtime management key naming | `set_provisioning_key`, `clear_provisioning_key` | `set_management_key`, `clear_management_key` |
| Legacy completions module | `api::completion::*` | `api::legacy::completion::*` |
| Legacy completion call path | `client.send_completion_request(...)` | `client.legacy().completions().create(...)` |
| Models user list method | `models().list_for_user()` | `models().list_user_models()` |
| Models count method | `models().count()` | `models().get_model_count()` |
| PKCE exchange in management domain | `management().exchange_code_for_api_key(...)` | `management().create_api_key_from_auth_code(...)` |
| API key management pagination | `client.list_api_keys(Some(offset_f64), include_disabled)` | `client.management().list_api_keys(Some(PaginationOptions::with_offset(offset_u32)), include_disabled)` |
| Guardrails access path | `client.list_guardrails(...)` | `client.management().list_guardrails(...)` |
| Flat endpoint calls | `client.send_chat_completion`, `client.stream_response`, `client.create_api_key`, ... | `client.chat().create`, `client.responses().stream`, `client.management().create_api_key`, ... |

## Top 10 Before/After Recipes

### 1) Builder management key rename

Before:

```rust
let client = OpenRouterClient::builder()
    .provisioning_key("or-mgmt-key")
    .build()?;
```

After:

```rust
let client = OpenRouterClient::builder()
    .management_key("or-mgmt-key")
    .build()?;
```

### 2) Runtime management key rename

Before:

```rust
client.set_provisioning_key("or-mgmt-key");
client.clear_provisioning_key();
```

After:

```rust
client.set_management_key("or-mgmt-key");
client.clear_management_key();
```

### 3) Legacy completions import path

Before:

```rust
use openrouter_rs::api::completion::CompletionRequest;
```

After:

```rust
use openrouter_rs::api::legacy::completion::CompletionRequest;
```

### 4) Legacy completions call path

Before:

```rust
let response = client.send_completion_request(&request).await?;
```

After:

```rust
let response = client.legacy().completions().create(&request).await?;
```

### 5) Flat chat call -> domain chat call

Before:

```rust
let response = client.send_chat_completion(&request).await?;
```

After:

```rust
let response = client.chat().create(&request).await?;
```

### 6) Flat response stream -> domain response stream

Before:

```rust
let stream = client.stream_response(&request).await?;
```

After:

```rust
let stream = client.responses().stream(&request).await?;
```

### 7) Models renamed methods

Before:

```rust
let user_models = client.models().list_for_user().await?;
let count = client.models().count().await?;
```

After:

```rust
let user_models = client.models().list_user_models().await?;
let count = client.models().get_model_count().await?;
```

### 8) PKCE exchange renamed in management domain

Before:

```rust
let auth = client
    .management()
    .exchange_code_for_api_key("code", Some("verifier"), None)
    .await?;
```

After:

```rust
let auth = client
    .management()
    .create_api_key_from_auth_code("code", Some("verifier"), None)
    .await?;
```

### 9) API keys pagination migration

Before:

```rust
let keys = client.list_api_keys(Some(0.0), Some(false)).await?;
```

After:

```rust
use openrouter_rs::types::PaginationOptions;

let keys = client
    .management()
    .list_api_keys(Some(PaginationOptions::with_offset(0)), Some(false))
    .await?;
```

### 10) Guardrails flat call -> management domain

Before:

```rust
use openrouter_rs::types::PaginationOptions;

let guardrails = client
    .list_guardrails(Some(PaginationOptions::with_offset_and_limit(0, 25)))
    .await?;
```

After:

```rust
use openrouter_rs::types::PaginationOptions;

let guardrails = client
    .management()
    .list_guardrails(Some(PaginationOptions::with_offset_and_limit(0, 25)))
    .await?;
```

## Feature Notes

- Legacy text completions require the `legacy-completions` feature.
- On the target `0.6.x` line, enable it explicitly if needed:

```toml
[dependencies]
openrouter-rs = { version = "0.8", features = ["legacy-completions"] }
```

## Validation Tips

- Compile your migration branch with strict checks:
  - `cargo check --all-targets --all-features`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Re-run your integration suite with real keys after API call-path updates.
