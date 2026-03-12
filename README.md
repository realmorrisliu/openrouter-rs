# OpenRouter Rust SDK

<div align="center">

Type-safe, async Rust bindings for the OpenRouter API.

[![Crates.io](https://img.shields.io/crates/v/openrouter-rs)](https://crates.io/crates/openrouter-rs)
[![Documentation](https://docs.rs/openrouter-rs/badge.svg)](https://docs.rs/openrouter-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[docs.rs](https://docs.rs/openrouter-rs) |
[examples](https://github.com/realmorrisliu/openrouter-rs/tree/main/examples) |
[crate](https://crates.io/crates/openrouter-rs)

</div>

`openrouter-rs` is built around the canonical `0.6.x` domain-oriented client surface:

- `client.chat()` for `POST /chat/completions`
- `client.responses()` for `POST /responses`
- `client.messages()` for Anthropic-compatible `POST /messages`
- `client.models()` for model discovery and embeddings
- `client.management()` for auth-code, API-key, guardrail, activity, and account-management flows
- `client.legacy()` for `POST /completions` when `legacy-completions` is explicitly enabled

The crate ships typed request/response models, builder-based ergonomics, streaming support, typed tools, multimodal chat content, and complete endpoint coverage for the current repository snapshot. The implementation map and live-test status live in [`docs/official-endpoint-test-matrix.md`](docs/official-endpoint-test-matrix.md).

## Installation

```toml
[dependencies]
openrouter-rs = "0.6.1"
tokio = { version = "1", features = ["full"] }
```

Legacy text completions are opt-in:

```toml
[dependencies]
openrouter-rs = { version = "0.6.1", features = ["legacy-completions"] }
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::Role,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenRouterClient::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY")?)
        .http_referer("https://yourapp.example")
        .x_title("my-openrouter-app")
        .build()?;

    let request = ChatCompletionRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .messages(vec![Message::new(
            Role::User,
            "Explain Rust ownership in plain English.",
        )])
        .build()?;

    let response = client.chat().create(&request).await?;
    println!("{}", response.choices[0].content().unwrap_or(""));

    Ok(())
}
```

## Design Overview

The main design change in `0.6.x` is that public documentation and examples treat the domain clients as the canonical surface. Flat `OpenRouterClient::*` helpers still exist in places, but they are not the recommended path for new code.

| Domain | Canonical methods | Primary endpoints | Auth note |
| --- | --- | --- | --- |
| `chat()` | `create`, `stream`, `stream_tool_aware`, `stream_unified` | `/chat/completions` | API key |
| `responses()` | `create`, `stream`, `stream_unified` | `/responses` | API key |
| `messages()` | `create`, `stream`, `stream_unified` | `/messages` | API key |
| `models()` | `list`, `list_by_category`, `list_by_parameters`, `list_endpoints`, `list_providers`, `list_user_models`, `get_model_count`, `list_zdr_endpoints`, `create_embedding`, `list_embedding_models` | `/models*`, `/providers`, `/endpoints/zdr`, `/embeddings*` | API key |
| `management()` | `create_api_key`, `list_api_keys`, `create_auth_code`, `create_api_key_from_auth_code`, `list_guardrails`, `get_activity`, `get_credits`, `create_coinbase_charge`, `get_generation` | `/keys*`, `/auth/keys*`, `/guardrails*`, `/activity`, `/credits*`, `/generation`, `/key` | Governed endpoints require a management key; billing/session endpoints still use the normal API key because that is how OpenRouter authenticates them |
| `legacy()` | `completions().create` | `/completions` | `legacy-completions` feature + API key |

The client builder exposes the runtime options the SDK directly consumes:

- `base_url`
- `api_key`
- `management_key`
- `http_referer`
- `x_title`

At runtime you can also call `set_api_key`, `clear_api_key`, `set_management_key`, and `clear_management_key`.

## Core Workflows

### Streaming

The SDK exposes three useful streaming layers:

1. Raw endpoint streams:
   - `chat().stream(...)`
   - `responses().stream(...)`
   - `messages().stream(...)`
2. Tool-aware chat aggregation:
   - `chat().stream_tool_aware(...)`
3. A unified event model across chat, responses, and messages:
   - `chat().stream_unified(...)`
   - `responses().stream_unified(...)`
   - `messages().stream_unified(...)`

```rust
use futures_util::StreamExt;
use openrouter_rs::types::stream::UnifiedStreamEvent;

let mut stream = client.chat().stream_unified(&request).await?;

while let Some(event) = stream.next().await {
    match event {
        UnifiedStreamEvent::ContentDelta(text) => print!("{text}"),
        UnifiedStreamEvent::ReasoningDelta(text) => eprint!("[reasoning]{text}"),
        UnifiedStreamEvent::Done { .. } => break,
        UnifiedStreamEvent::Error(err) => {
            eprintln!("stream error: {err}");
            break;
        }
        _ => {}
    }
}
```

### Tool Calling And Typed Tools

Manual tool schemas and typed tools are both first-class:

- `types::Tool` for explicit JSON-schema tool definitions
- `types::typed_tool::{TypedTool, TypedToolParams}` for Rust-typed tools backed by `schemars`

```rust
use openrouter_rs::types::typed_tool::{TypedTool, TypedToolParams};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
struct WeatherParams {
    location: String,
    unit: Option<String>,
}

impl TypedTool for WeatherParams {
    fn name() -> &'static str { "get_weather" }
    fn description() -> &'static str { "Fetch weather for a location" }
}

let request = ChatCompletionRequest::builder()
    .model("anthropic/claude-sonnet-4")
    .messages(vec![Message::new(Role::User, "Weather in Paris?")])
    .typed_tool::<WeatherParams>()
    .build()?;
```

### Multimodal Chat Content

`api::chat::ContentPart` now covers text, image URLs, audio input, video input, and file payloads.

```rust
use openrouter_rs::api::chat::ContentPart;

let request = ChatCompletionRequest::builder()
    .model("anthropic/claude-sonnet-4")
    .messages(vec![Message::with_parts(
        Role::User,
        vec![
            ContentPart::text("Describe this image."),
            ContentPart::image_url_with_detail("https://example.com/image.jpg", "high"),
        ],
    )])
    .build()?;
```

### Responses, Messages, And Embeddings

The current repo has dedicated typed surfaces for the non-chat APIs as well:

```rust
use openrouter_rs::{
    api::{
        embeddings::EmbeddingRequest,
        messages::{AnthropicMessage, AnthropicMessagesRequest},
        responses::ResponsesRequest,
    },
};
use serde_json::json;

let responses_request = ResponsesRequest::builder()
    .model("openai/gpt-5")
    .input(json!([{ "role": "user", "content": "Say hello." }]))
    .build()?;
let _responses = client.responses().create(&responses_request).await?;

let messages_request = AnthropicMessagesRequest::builder()
    .model("anthropic/claude-sonnet-4")
    .max_tokens(256)
    .messages(vec![AnthropicMessage::user("Say hello.")])
    .build()?;
let _message = client.messages().create(&messages_request).await?;

let embedding_request = EmbeddingRequest::builder()
    .model("openai/text-embedding-3-large")
    .input("OpenRouter Rust SDK")
    .build()?;
let _embedding = client.models().create_embedding(&embedding_request).await?;
```

### Discovery And Management

Discovery and governance endpoints are intentionally separated:

```rust
use openrouter_rs::{
    api::guardrails::CreateGuardrailRequest,
    types::{ModelCategory, PaginationOptions},
};

let programming_models = client
    .models()
    .list_by_category(ModelCategory::Programming)
    .await?;

let providers = client.models().list_providers().await?;
let zdr_endpoints = client.models().list_zdr_endpoints().await?;

let keys = client
    .management()
    .list_api_keys(Some(PaginationOptions::with_offset_and_limit(0, 25)), Some(false))
    .await?;

let guardrail = client
    .management()
    .create_guardrail(
        &CreateGuardrailRequest::builder()
            .name("ci-budget-cap")
            .limit_usd(25.0)
            .enforce_zdr(true)
            .build()?,
    )
    .await?;
```

Management-key reminders:

- `/activity` requires `.management_key(...)`
- `/keys*` requires `.management_key(...)`
- `/auth/keys*` requires `.management_key(...)`
- `/guardrails*` requires `.management_key(...)`

API-key reminders:

- `/credits`
- `/credits/coinbase`
- `/generation`
- `/key`

Those endpoints are grouped under `management()` for discoverability, but the underlying upstream auth model is still API-key based.

## Client Setup

The SDK intentionally keeps setup narrow: configure runtime values on `OpenRouterClient::builder()`, then choose the final `model` on each request builder.

```rust
use openrouter_rs::OpenRouterClient;

let client = OpenRouterClient::builder()
    .api_key("your_api_key")
    .http_referer("https://yourapp.example")
    .x_title("My App")
    .build()?;

# Ok::<(), openrouter_rs::error::OpenRouterError>(())
```

File/profile config resolution belongs to the companion CLI or to the caller's application layer, not to the SDK core.

## Legacy Completions

Legacy `POST /completions` support is isolated behind the `legacy-completions` feature and the explicit legacy namespace.

```rust
use openrouter_rs::{OpenRouterClient, api::legacy::completion::CompletionRequest};

let request = CompletionRequest::builder()
    .model("deepseek/deepseek-chat-v3-0324:free")
    .prompt("Once upon a time")
    .build()?;

let response = client.legacy().completions().create(&request).await?;
```

For new applications, prefer `chat()` or `responses()`.

### 🔁 0.6 Naming/Pagination Migration

Full migration guide: [MIGRATION.md](./MIGRATION.md)

- `models().count()` -> `models().get_model_count()`
- `models().list_for_user()` -> `models().list_user_models()`
- `management().exchange_code_for_api_key(...)` -> `management().create_api_key_from_auth_code(...)`
- `management().list_guardrails(offset, limit)` -> `management().list_guardrails(Some(PaginationOptions::with_offset_and_limit(offset, limit)))`
- `client.list_api_keys(offset, include_disabled)` -> `management().list_api_keys(Some(PaginationOptions::with_offset(offset)), include_disabled)`

`0.6.0` removes the transitional aliases above; use the canonical method names shown in the mapping list.

Migration validation commands for contributors:

```bash
./scripts/check_migration_docs.sh
cargo test --test migration_smoke --all-features
```

## Examples

The repo includes runnable examples for the canonical flows:

| Example | Focus |
| --- | --- |
| `examples/domain_chat_completion.rs` | Canonical `chat()` usage |
| `examples/basic_tool_calling.rs` | Manual tool-calling loop |
| `examples/typed_tool_calling.rs` | Typed tools with generated schema |
| `examples/chat_with_reasoning.rs` | Reasoning controls |
| `examples/stream_chat_completion.rs` | Raw chat streaming |
| `examples/stream_chat_with_tools.rs` | `ToolAwareStream` |
| `examples/create_response.rs` | `responses()` create |
| `examples/stream_response.rs` | `responses()` streaming |
| `examples/create_message.rs` | `messages()` create |
| `examples/stream_messages.rs` | `messages()` streaming |
| `examples/create_embedding.rs` | `models().create_embedding(...)` |
| `examples/domain_management_api_keys.rs` | API-key management via `management()` |
| `examples/exchange_code_for_api_key.rs` | PKCE/auth-code flow |
| `examples/send_completion_request.rs` | Legacy completions (`legacy-completions` required) |

Local commands:

```bash
export OPENROUTER_API_KEY=sk-or-v1-...

cargo run --example domain_chat_completion
cargo run --example basic_tool_calling
cargo run --example typed_tool_calling
cargo run --example chat_with_reasoning
cargo run --example stream_chat_completion
cargo run --example stream_chat_with_tools
cargo run --example create_response
cargo run --example stream_response
cargo run --example create_message
cargo run --example stream_messages
cargo run --example create_embedding
cargo run --features legacy-completions --example send_completion_request
```

## CLI Companion

This workspace also contains [`crates/openrouter-cli`](crates/openrouter-cli), a companion CLI for profile resolution, discovery, management, and usage/billing workflows.

Useful entrypoints:

```bash
cargo run -p openrouter-cli -- --help
cargo run -p openrouter-cli -- profile show
cargo run -p openrouter-cli -- models list --category programming
cargo run -p openrouter-cli -- keys list --include-disabled
cargo run -p openrouter-cli -- usage activity --date 2026-03-01
```

CLI auth/config precedence is deterministic:

1. Flags: `--api-key`, `--management-key`, `--base-url`
2. Environment: `OPENROUTER_API_KEY`, `OPENROUTER_MANAGEMENT_KEY`, `OPENROUTER_BASE_URL`
3. Profile config values from `profiles.toml`
4. Default `base_url`

See [`crates/openrouter-cli/README.md`](crates/openrouter-cli/README.md) for the full command surface.

## Testing And Quality

Prefer the `just` recipes so local work stays aligned with CI:

```bash
just quality
just quality-ci
just test-live-contract
OPENROUTER_MANAGEMENT_KEY=... just test-live-contract-management
```

Focused commands:

- `just test-unit`
- `just test-lib`
- `just test-doc`
- `just test-integration-subsets`
- `just test-cli`
- `just check-migration-docs`
- `just test-migration-smoke`
- `just test-integration`

Environment and model-pool details live in [`tests/integration/README.md`](tests/integration/README.md). A starter env file lives at [`.env.example`](.env.example).

## Repo Docs

- [MIGRATION.md](MIGRATION.md) for `0.5.x -> 0.6.0`
- [docs/official-endpoint-test-matrix.md](docs/official-endpoint-test-matrix.md) for endpoint-by-endpoint implementation/test status
- [tests/integration/README.md](tests/integration/README.md) for live test pools and env switches
- [crates/openrouter-cli/README.md](crates/openrouter-cli/README.md) for CLI behavior and examples
- [CHANGELOG.md](CHANGELOG.md) for release-by-release changes

## Contributing

When you change API surface or examples:

- update the relevant README/docs in the same change
- run `just quality`
- run `just quality-ci` if you touched migration docs, CLI behavior, or CI-aligned release/test flows

## Requirements

- Rust `1.85+`
- Tokio `1.x`
- An `OPENROUTER_API_KEY` for API-backed examples and live tests
- An `OPENROUTER_MANAGEMENT_KEY` for management-governed examples/tests

## 📈 Release History

### Version 0.6.1 *(Latest)*

- Fixed `ToolBuilder` field loss when setters are called in different orders.
- Preserved combined model filters and model resolution ordering, and propagated default headers to chat streaming requests.
- Hardened SSE frame parsing, normalized response parsing errors across endpoints, and aligned release validation around `just` plus live contract checks.

### Version 0.6.0

- Removed `0.5.x` compatibility aliases, made `legacy-completions` opt-in, and standardized the canonical domain-client documentation around `chat()`, `responses()`, `messages()`, `models()`, and `management()`.

### Version 0.5.2

- Added `/messages`, discovery/activity, guardrails, auth-code flows, unified streaming, CLI foundation, and the `0.5.x -> 0.6.0` migration bridge.

See [CHANGELOG.md](CHANGELOG.md) for the full history.

## License

MIT. See [LICENSE](LICENSE).

## Disclaimer

This is a third-party SDK and is not affiliated with OpenRouter.
