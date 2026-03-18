# openrouter-rs

<div align="center">

Type-safe, async Rust SDK for the OpenRouter API.

[![Crates.io](https://img.shields.io/crates/v/openrouter-rs)](https://crates.io/crates/openrouter-rs)
[![Documentation](https://docs.rs/openrouter-rs/badge.svg)](https://docs.rs/openrouter-rs)
[![CI](https://github.com/realmorrisliu/openrouter-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/realmorrisliu/openrouter-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[docs.rs](https://docs.rs/openrouter-rs) |
[examples](https://github.com/realmorrisliu/openrouter-rs/tree/main/examples) |
[crate](https://crates.io/crates/openrouter-rs) |
[openrouter-cli](https://github.com/realmorrisliu/openrouter-rs/tree/main/crates/openrouter-cli) |
[contributing](CONTRIBUTING.md) |
[endpoint matrix](docs/official-endpoint-test-matrix.md) |
[changelog](CHANGELOG.md)

</div>

`openrouter-rs` is a community-maintained Rust SDK for OpenRouter. It exposes a domain-oriented client for chat, responses, messages, models, embeddings, and management APIs, plus a companion CLI in the same repository.

The current repo snapshot implements `36 / 36` official OpenAPI method/path entries, with published live integration coverage tracked in [`docs/official-endpoint-test-matrix.md`](docs/official-endpoint-test-matrix.md).

## Why `openrouter-rs`

- Domain-oriented clients: `chat()`, `responses()`, `messages()`, `models()`, `management()`, and opt-in `legacy()`
- Typed request/response models with builder-style ergonomics
- Streaming support for chat, responses, and messages, including a unified stream abstraction
- Typed tools, manual JSON-schema tools, and multimodal chat content
- Discovery, embeddings, API-key management, guardrails, activity, credits, and generation coverage
- A companion CLI for profile resolution, discovery, management, and billing/usage workflows

## Installation

```toml
[dependencies]
openrouter-rs = "0.7.0"
tokio = { version = "1", features = ["full"] }
```

Legacy text completions are opt-in:

```toml
[dependencies]
openrouter-rs = { version = "0.7.0", features = ["legacy-completions"] }
tokio = { version = "1", features = ["full"] }
```

Requirements:

- Rust `1.85+`
- Tokio `1.x`
- `OPENROUTER_API_KEY` for API-backed examples and live tests
- `OPENROUTER_MANAGEMENT_KEY` for management-governed examples and tests

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

The SDK keeps setup intentionally narrow: configure runtime values on `OpenRouterClient::builder()`, then choose the final `model` on each request builder. File/profile config resolution belongs in the companion CLI or in your application layer, not in the SDK core.

## API Surface

The canonical public surface in `0.7.x` is domain-oriented:

| Domain | Canonical methods | Primary endpoints | Auth note |
| --- | --- | --- | --- |
| `chat()` | `create`, `stream`, `stream_tool_aware`, `stream_unified` | `/chat/completions` | API key |
| `responses()` | `create`, `stream`, `stream_unified` | `/responses` | API key |
| `messages()` | `create`, `stream`, `stream_unified` | `/messages` | API key |
| `models()` | `list`, `list_by_category`, `list_by_parameters`, `list_endpoints`, `list_providers`, `list_user_models`, `get_model_count`, `list_zdr_endpoints`, `create_embedding`, `list_embedding_models` | `/models*`, `/providers`, `/endpoints/zdr`, `/embeddings*` | API key |
| `management()` | `create_api_key`, `list_api_keys`, `create_auth_code`, `create_api_key_from_auth_code`, `list_guardrails`, `get_activity`, `get_credits`, `create_coinbase_charge`, `get_generation` | `/keys*`, `/auth/keys*`, `/guardrails*`, `/activity`, `/credits*`, `/generation`, `/key` | Governed endpoints require a management key; billing/session endpoints still use the normal API key because that is how OpenRouter authenticates them |
| `legacy()` | `completions().create` | `/completions` | `legacy-completions` feature + API key |

At runtime, the builder/client exposes the values the SDK directly consumes:

- `base_url`
- `api_key`
- `management_key`
- `http_referer`
- `x_title`

## Common Workflows

`openrouter-rs` is not just a thin `/chat/completions` wrapper. The repo currently covers:

- chat completions, responses, and Anthropic-compatible messages
- unified streaming across chat, responses, and messages
- manual tools and typed tools backed by `schemars`
- multimodal chat content, including image, audio, video, and file parts
- model discovery, provider discovery, embeddings, and ZDR endpoints
- management-key workflows for keys, auth codes, guardrails, activity, credits, and generation

For deeper examples, prefer the runnable examples in [`examples/`](examples) over long README snippets.

## Examples

The repo includes runnable examples for the highest-value workflows:

| Example | Focus |
| --- | --- |
| [`examples/domain_chat_completion.rs`](examples/domain_chat_completion.rs) | Canonical `chat()` usage |
| [`examples/stream_chat_completion.rs`](examples/stream_chat_completion.rs) | Raw chat streaming |
| [`examples/stream_chat_with_tools.rs`](examples/stream_chat_with_tools.rs) | Tool-aware streaming aggregation |
| [`examples/basic_tool_calling.rs`](examples/basic_tool_calling.rs) | Manual tool-calling loop |
| [`examples/typed_tool_calling.rs`](examples/typed_tool_calling.rs) | Typed tools with generated schema |
| [`examples/create_response.rs`](examples/create_response.rs) | `responses()` create |
| [`examples/stream_response.rs`](examples/stream_response.rs) | `responses()` streaming |
| [`examples/create_message.rs`](examples/create_message.rs) | `messages()` create |
| [`examples/stream_messages.rs`](examples/stream_messages.rs) | `messages()` streaming |
| [`examples/create_embedding.rs`](examples/create_embedding.rs) | `models().create_embedding(...)` |
| [`examples/domain_management_api_keys.rs`](examples/domain_management_api_keys.rs) | API-key management via `management()` |
| [`examples/exchange_code_for_api_key.rs`](examples/exchange_code_for_api_key.rs) | PKCE/auth-code flow |
| [`examples/send_completion_request.rs`](examples/send_completion_request.rs) | Legacy completions (`legacy-completions` required) |

Typical local usage:

```bash
export OPENROUTER_API_KEY=sk-or-v1-...

cargo run --example domain_chat_completion
cargo run --example stream_chat_completion
cargo run --example typed_tool_calling
cargo run --example create_response
cargo run --example create_message
cargo run --example create_embedding
```

## CLI Companion

This workspace also contains [`crates/openrouter-cli`](crates/openrouter-cli), a companion CLI for profile resolution, discovery, management, and usage/billing workflows.

Examples:

```bash
cargo run -p openrouter-cli -- --help
cargo run -p openrouter-cli -- profile show
cargo run -p openrouter-cli -- models list --category programming
cargo run -p openrouter-cli -- keys list --include-disabled
cargo run -p openrouter-cli -- usage activity --date 2026-03-01
```

See [`crates/openrouter-cli/README.md`](crates/openrouter-cli/README.md) for the full command surface and config/auth precedence rules.

## Project Status

- Community-maintained third-party SDK; not affiliated with OpenRouter
- Canonical docs and examples prefer the domain clients over older flat helpers
- Full endpoint coverage is tracked against the current OpenAPI snapshot
- Live integration coverage and gaps are published in [`docs/official-endpoint-test-matrix.md`](docs/official-endpoint-test-matrix.md)
- Migration guidance for the `0.5.x -> 0.6.x` transition lives in [`MIGRATION.md`](MIGRATION.md)
- Legacy `POST /completions` support remains available behind the `legacy-completions` feature

## Development

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

## Repository Docs

- [`MIGRATION.md`](MIGRATION.md) for migration guidance
- [`CONTRIBUTING.md`](CONTRIBUTING.md) for contributor workflow and review expectations
- [`docs/maintenance-policy.md`](docs/maintenance-policy.md) for release, MSRV, and breaking-change policy
- [`SECURITY.md`](SECURITY.md) for vulnerability reporting
- [`SUPPORT.md`](SUPPORT.md) for support boundaries and issue-reporting guidance
- [`docs/official-endpoint-test-matrix.md`](docs/official-endpoint-test-matrix.md) for endpoint-by-endpoint implementation and test status
- [`tests/integration/README.md`](tests/integration/README.md) for live test pools and env switches
- [`crates/openrouter-cli/README.md`](crates/openrouter-cli/README.md) for CLI behavior and examples
- [`CHANGELOG.md`](CHANGELOG.md) for release-by-release changes

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full contributor workflow.

At a minimum, if you change public API surface, examples, or docs:

- update the relevant README/docs in the same change
- run `just quality`
- run `just quality-ci` if you touched migration docs, CLI behavior, or CI-aligned release/test flows

Related policies:

- [`docs/maintenance-policy.md`](docs/maintenance-policy.md)
- [`SECURITY.md`](SECURITY.md)
- [`SUPPORT.md`](SUPPORT.md)

## License

MIT. See [LICENSE](LICENSE).
