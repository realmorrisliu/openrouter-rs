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
[docs map](docs/README.md) |
[openrouter-cli](https://github.com/realmorrisliu/openrouter-rs/tree/main/crates/openrouter-cli) |
[contributing](CONTRIBUTING.md) |
[changelog](CHANGELOG.md)

</div>

`openrouter-rs` is a community-maintained Rust SDK for OpenRouter. It exposes a domain-oriented client for chat, responses, messages, rerank, audio speech/transcription, image generation, video generation, models, embeddings, files, presets, analytics, and management APIs, plus a companion CLI in the same repository.

The current repo snapshot implements `87 / 87` official OpenAPI method/path entries, with published live integration coverage tracked in [`docs/operations/official-endpoint-test-matrix.md`](docs/operations/official-endpoint-test-matrix.md).

## Why `openrouter-rs`

- Domain-oriented clients: `chat()`, `responses()`, `messages()`, `rerank()`, `audio().speech()`, `audio().transcriptions()`, `images()`, `videos()`, `files()`, `models()`, `management()`, and opt-in `legacy()`
- Typed request/response models with builder-style ergonomics
- Tokio-native `reqwest + rustls` transport with no `surf` / `curl` dependency chain
- Streaming support for chat, responses, messages, and image generation, including a unified stream abstraction
- Typed tools, manual JSON-schema tools, and multimodal chat content
- Typed chat usage metadata for token counts, OpenRouter cost, provider cost breakdowns, and BYOK status
- Opt-in OpenRouter response metadata for chat, Responses API, and Anthropic-compatible Messages requests
- Discovery, rankings and benchmark datasets, task classifications, rerank, audio speech/transcription, image generation, video generation, files, embeddings, API-key management, preset creation/readback/versioning, analytics, BYOK provider credentials, observability destinations, workspace management, organization members, guardrails, activity, credits, and generation metadata/content coverage
- A companion CLI for profile resolution, discovery, management, and billing/usage workflows

## Installation

```toml
[dependencies]
openrouter-rs = "0.11.1"
tokio = { version = "1", features = ["full"] }
```

Legacy text completions are opt-in:

```toml
[dependencies]
openrouter-rs = { version = "0.11.1", features = ["legacy-completions"] }
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
        .app_categories(["cli-agent"])
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

The canonical public surface is domain-oriented:

| Domain | Canonical methods | Primary endpoints | Auth note |
| --- | --- | --- | --- |
| `chat()` | `create`, `stream`, `stream_tool_aware`, `stream_unified` | `/chat/completions` | API key |
| `responses()` | `create`, `stream`, `stream_unified` | `/responses` | API key |
| `messages()` | `create`, `stream`, `stream_unified` | `/messages` | API key |
| `rerank()` | `create` | `/rerank` | API key |
| `audio().speech()` | `create` | `/audio/speech` (legacy `/tts` fallback) | API key |
| `audio().transcriptions()` | `create` | `/audio/transcriptions` | API key |
| `images()` | `create`, `stream`, `list_models`, `list_model_endpoints` | `/images*` | API key |
| `videos()` | `create`, `list_models`, `get_generation`, `get_content` | `/videos*` | API key |
| `files()` | `list`, `upload`, `get_metadata`, `download_content`, `delete` | `/files*` | API key |
| `models()` | `list`, `list_filtered`, `list_by_category`, `list_by_parameters`, `get`, `list_endpoints`, `list_providers`, `list_user_models`, `get_model_count`, `get_rankings_daily`, `get_app_rankings`, `get_task_classifications`, `get_benchmarks`, `list_zdr_endpoints`, `create_embedding`, `list_embedding_models` | `/model*`, `/models*`, `/providers`, `/datasets/*`, `/classifications/task`, `/benchmarks`, `/endpoints/zdr`, `/embeddings*` | API key |
| `management()` | `create_api_key`, `create_api_key_in_workspace`, `list_api_keys`, `list_api_keys_in_workspace`, `list_presets`, `get_preset`, `list_preset_versions`, `get_preset_version`, `create_chat_completion_preset`, `create_response_preset`, `create_message_preset`, `get_analytics_meta`, `query_analytics`, `list_byok_keys`, `create_byok_key`, `get_byok_key`, `update_byok_key`, `delete_byok_key`, `list_observability_destinations`, `create_observability_destination`, `get_observability_destination`, `update_observability_destination`, `delete_observability_destination`, `create_auth_code`, `create_api_key_from_auth_code`, `list_guardrails`, `list_guardrails_in_workspace`, `create_guardrail`, `list_organization_members`, `list_workspaces`, `create_workspace`, `get_workspace`, `update_workspace`, `delete_workspace`, `list_workspace_budgets`, `upsert_workspace_budget`, `delete_workspace_budget`, `add_workspace_members`, `remove_workspace_members`, `get_activity`, `get_credits`, `create_coinbase_charge`, `get_generation`, `get_generation_content` | `/keys*`, `/presets*`, `/analytics*`, `/byok*`, `/observability/destinations*`, `/auth/keys*`, `/guardrails*`, `/organization/members`, `/workspaces*`, `/activity`, `/credits*`, `/generation*`, `/key` | Governed endpoints require a management key; billing/session endpoints still use the normal API key because that is how OpenRouter authenticates them |
| `legacy()` | `completions().create` | `/completions` | `legacy-completions` feature + API key |

At runtime, the builder/client exposes the values the SDK directly consumes:

- `base_url`
- `api_key`
- `management_key`
- `http_referer`
- `x_title`
- `app_categories`
- `http_client`

Chat, Responses API, and Anthropic-compatible Messages request builders also expose `experimental_metadata(OpenRouterExperimentalMetadata::Enabled)` for OpenRouter's opt-in routing metadata response header.

## Common Workflows

`openrouter-rs` is not just a thin `/chat/completions` wrapper. The repo currently covers:

- chat completions, responses, and Anthropic-compatible messages
- rerank, audio speech generation, audio transcription, image generation, and video generation polling/content retrieval
- unified streaming across chat, responses, and messages
- manual tools and typed tools backed by `schemars`
- multimodal chat content, including image, audio, video, and file parts
- model discovery, provider discovery, app rankings, task classifications, unified benchmarks, embeddings, and ZDR endpoints
- typed generation metadata, model voice/benchmark/link metadata, workspace I/O logging controls, video callback URL support, and file upload/download workflows
- management-key workflows for keys, workspace-scoped keys, preset reads/writes, analytics, BYOK provider credentials, observability destinations, auth codes, organization members, workspaces, workspace budgets, workspace membership, guardrails, guardrail content filters, workspace-scoped guardrails, and activity, plus API-key-authenticated credits and generation metadata/content endpoints

For deeper examples, prefer the runnable examples in [`examples/`](examples) over long README snippets.

## Examples

The repo includes runnable examples for the highest-value workflows:

### Application Patterns

| Example | Focus |
| --- | --- |
| [`examples/axum_chat_gateway.rs`](examples/axum_chat_gateway.rs) | Minimal `axum` server that proxies prompts through `OpenRouterClient` |
| [`examples/typed_tool_agent.rs`](examples/typed_tool_agent.rs) | Practical typed-tool agent loop with explicit tool dispatch |
| [`examples/domain_chat_completion.rs`](examples/domain_chat_completion.rs) | Canonical `chat()` request with the domain-oriented client |
| [`examples/custom_http_client.rs`](examples/custom_http_client.rs) | Inject a custom `reqwest::Client` (proxies, timeouts, middleware) |

### Tokio Streaming

| Example | Focus |
| --- | --- |
| [`examples/stream_chat_completion.rs`](examples/stream_chat_completion.rs) | Stream chat deltas into a Tokio task/stdout |
| [`examples/stream_chat_with_tools.rs`](examples/stream_chat_with_tools.rs) | Tool-aware streaming aggregation plus a second round after execution |
| [`examples/stream_response.rs`](examples/stream_response.rs) | `responses()` streaming |
| [`examples/stream_messages.rs`](examples/stream_messages.rs) | `messages()` streaming |

### API Surface Demos

| Example | Focus |
| --- | --- |
| [`examples/basic_tool_calling.rs`](examples/basic_tool_calling.rs) | Manual tool-calling loop |
| [`examples/typed_tool_calling.rs`](examples/typed_tool_calling.rs) | Typed tools with generated schema |
| [`examples/create_response.rs`](examples/create_response.rs) | `responses()` create |
| [`examples/create_message.rs`](examples/create_message.rs) | `messages()` create |
| [`examples/create_rerank.rs`](examples/create_rerank.rs) | `rerank().create(...)` |
| [`examples/create_speech.rs`](examples/create_speech.rs) | `audio().speech().create(...)` |
| [`examples/create_transcription.rs`](examples/create_transcription.rs) | `audio().transcriptions().create(...)` |
| [`examples/create_image_generation.rs`](examples/create_image_generation.rs) | `images().create(...)` |
| [`examples/create_video_generation.rs`](examples/create_video_generation.rs) | `videos().create(...)` |
| [`examples/create_embedding.rs`](examples/create_embedding.rs) | `models().create_embedding(...)` |
| [`examples/domain_management_api_keys.rs`](examples/domain_management_api_keys.rs) | API-key management via `management()` |
| [`examples/list_byok_keys.rs`](examples/list_byok_keys.rs) | `management().list_byok_keys(...)` |
| [`examples/list_observability_destinations.rs`](examples/list_observability_destinations.rs) | `management().list_observability_destinations(...)` |
| [`examples/list_organization_members.rs`](examples/list_organization_members.rs) | `management().list_organization_members(...)` |
| [`examples/list_workspaces.rs`](examples/list_workspaces.rs) | `management().list_workspaces(...)` |
| [`examples/exchange_code_for_api_key.rs`](examples/exchange_code_for_api_key.rs) | PKCE/auth-code flow |
| [`examples/send_completion_request.rs`](examples/send_completion_request.rs) | Legacy completions (`legacy-completions` required) |

Typical local usage:

```bash
export OPENROUTER_API_KEY=sk-or-v1-...

cargo run --example axum_chat_gateway
cargo run --example domain_chat_completion
cargo run --example stream_chat_completion
cargo run --example typed_tool_agent
cargo run --example typed_tool_calling
cargo run --example create_response
cargo run --example create_message
cargo run --example create_rerank
cargo run --example create_speech
cargo run --example create_transcription
cargo run --example create_image_generation
cargo run --example create_embedding
cargo run --example list_byok_keys
cargo run --example list_observability_destinations
cargo run --example custom_http_client
```

For shell and CI automation recipes built around the companion CLI, see [`docs/operations/cli-automation-workflows.md`](docs/operations/cli-automation-workflows.md).

## CLI Companion

This workspace also contains [`crates/openrouter-cli`](crates/openrouter-cli), a companion CLI for profile resolution, discovery, management, and usage/billing workflows.

Examples:

```bash
cargo run -p openrouter-cli -- --help
cargo run -p openrouter-cli -- profile show
cargo run -p openrouter-cli -- models list --category programming
cargo run -p openrouter-cli -- keys list --include-disabled
cargo run -p openrouter-cli -- workspaces list --limit 20
cargo run -p openrouter-cli -- usage activity --date 2026-03-01
```

See [`crates/openrouter-cli/README.md`](crates/openrouter-cli/README.md) for the full command surface and config/auth precedence rules.
For copy-paste shell/CI recipes, see [`docs/operations/cli-automation-workflows.md`](docs/operations/cli-automation-workflows.md).

## Project Status

- Community-maintained third-party SDK; not affiliated with OpenRouter
- Canonical docs and examples prefer the domain clients over older flat helpers
- Accepted endpoint coverage is tracked against the current OpenAPI snapshot, and the current baseline is fully implemented at the SDK surface (`87 / 87`)
- Live integration coverage and gaps are published in [`docs/operations/official-endpoint-test-matrix.md`](docs/operations/official-endpoint-test-matrix.md)
- Migration guidance for the `0.9.x -> 0.10.0` public-model future-proofing release, the `0.8.x -> 0.9.0` audio speech release, the `0.7.x -> 0.8.0` transport/error-surface release, and the archived `0.5.x -> 0.6.x` naming guide lives in [`MIGRATION.md`](MIGRATION.md)
- Legacy `POST /completions` support remains available behind the `legacy-completions` feature

### 🔁 0.10 Public Model Migration

Full migration guide: [`MIGRATION.md`](MIGRATION.md)

- High-churn public SDK request, response, metadata, usage, pricing, discovery, streaming, and upstream taxonomy types are `#[non_exhaustive]`
- Replace affected struct literals with builders, constructors, helpers, or serde deserialization
- Add wildcard arms when matching affected public enums outside the crate
- Use constructors such as `ResponseUsage::new(...)`, `ToolCall::new(...)`, `FunctionCall::new(...)`, and `JsonSchemaConfig::new(...)` for small helper/test fixtures
- This source-level break lands in `0.10.0`

### 🔁 0.9 Audio Speech Migration

- `client.tts().create(...)` -> `client.audio().speech().create(...)`
- `api::tts::TtsRequest` -> `api::audio::SpeechRequest`
- `api::tts::TtsResponseFormat` -> `api::audio::SpeechResponseFormat`
- `api::tts` remains as a deprecated compatibility module, but new examples and docs use `api::audio`
- Newly added and high-churn public request/response structs use builder construction and may be marked `#[non_exhaustive]`; prefer request builders over struct literals

### 🔁 0.8 Transport/Error Migration

Full migration guide: [`MIGRATION.md`](MIGRATION.md)

- `OpenRouterError::HttpRequest(surf::Error)` -> `OpenRouterError::HttpRequest(HttpRequestError)`
- `ApiErrorContext.status: surf::StatusCode` -> `ApiErrorContext.status: http::StatusCode`
- `openrouter_rs::utils::{with_bearer_auth, with_request_metadata, with_client_request_headers, handle_error}` -> caller-owned transport helpers or direct use of the canonical domain clients
- No API migration is required if you only use `OpenRouterClient` plus `chat()`, `responses()`, `messages()`, `rerank()`, `audio()`, `images()`, `videos()`, `files()`, `models()`, `management()`, and `legacy()`

### 🔁 0.6 Naming/Pagination Migration

Full migration guide: [`MIGRATION.md`](MIGRATION.md)

- `models().count()` -> `models().get_model_count()`
- `models().list_for_user()` -> `models().list_user_models()`
- `management().exchange_code_for_api_key(...)` -> `management().create_api_key_from_auth_code(...)`
- `management().list_guardrails(offset, limit)` -> `management().list_guardrails(Some(PaginationOptions::with_offset_and_limit(offset, limit)))`
- `client.list_api_keys(offset, include_disabled)` -> `management().list_api_keys(Some(PaginationOptions::with_offset(offset)), include_disabled)`

`0.6.0` removed the transitional aliases above; use the canonical method names shown here.

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

## Docs Map

Start with [`docs/README.md`](docs/README.md) for grouped navigation across root docs, `docs/`, `specs/`, and subsystem READMEs.

### Users And Setup

- [`MIGRATION.md`](MIGRATION.md) for upgrade guidance across breaking SDK changes
- [`crates/openrouter-cli/README.md`](crates/openrouter-cli/README.md) for CLI behavior, examples, and auth/config precedence
- [`CHANGELOG.md`](CHANGELOG.md) for release-by-release history

### Contributors And Project Policies

- [`CONTRIBUTING.md`](CONTRIBUTING.md) for contributor workflow and review expectations
- [`docs/policies/maintenance-policy.md`](docs/policies/maintenance-policy.md) for release, MSRV, and breaking-change policy
- [`docs/policies/compatibility-update-policy.md`](docs/policies/compatibility-update-policy.md) for upstream compatibility reporting cadence, templates, and update rules
- [`SECURITY.md`](SECURITY.md) for vulnerability reporting
- [`SUPPORT.md`](SUPPORT.md) for support boundaries and issue-reporting guidance

### Design And Roadmap

- [`docs/design/generated-core-architecture.md`](docs/design/generated-core-architecture.md) for the generated-core plus idiomatic-wrapper design baseline
- [`docs/design/http-transport-migration.md`](docs/design/http-transport-migration.md) for the historical design baseline behind the completed `reqwest + rustls` transport migration

### Operations, Validation, And Distribution

- [`docs/operations/official-endpoint-test-matrix.md`](docs/operations/official-endpoint-test-matrix.md) for endpoint-by-endpoint implementation and test status
- [`docs/operations/openapi-drift-reporting.md`](docs/operations/openapi-drift-reporting.md) for weekly upstream-spec drift detection and baseline refresh workflow
- [`docs/operations/cli-automation-workflows.md`](docs/operations/cli-automation-workflows.md) for JSON-first shell and CI recipes built around `openrouter-cli`
- [`tests/integration/README.md`](tests/integration/README.md) for live test pools and env switches
- [`docs/community/awesome-openrouter/README.md`](docs/community/awesome-openrouter/README.md) for the Awesome OpenRouter submission kit and directory-safe assets

## 📈 Release History

### Unreleased

- No unreleased changes.

### Version 0.11.1 *(Latest)*

- Accepted OpenRouter's new `max` reasoning effort and preserved future provider-defined effort strings in model metadata, keeping `/models` deserialization and CLI model listing forward-compatible with upstream taxonomy additions.
- Hardened scheduled live Responses checks so transient runtime statuses stay service warnings while completed empty outputs, SDK deserialization failures, and non-retryable API errors remain hard failures.

### Version 0.11.0

- Added typed SDK coverage for image generation, files, analytics, app rankings, task classifications, unified benchmarks, singular model lookup, rankings-daily, preset listing/readback/versioning, workspace budgets, and expanded model filters.
- Added automatic prompt caching request support through top-level `cache_control` builders for chat completions and Responses API requests.
- Refreshed multimodal and metadata schemas, including rerank image-only document echoes, nullable model/generation metadata, Responses debug options, and Anthropic Messages system-role helpers.
- Accepted OpenAPI drift through the 2026-06-29 review and restored the tracked endpoint snapshot to `87 / 87`.

### Version 0.10.0

- Marked high-churn public SDK request, response, metadata, usage, pricing, discovery, streaming, and upstream taxonomy types as `#[non_exhaustive]`; use builders, constructors, helpers, serde deserialization, or wildcard enum arms when migrating from `0.9.x`.
- Added typed SDK coverage for audio transcriptions, BYOK provider credentials, observability destinations, multimodal embedding media parts, experimental response metadata, and model/generation metadata refreshes.
- Added `OpenRouterClientBuilder::http_client(...)` for injecting a custom `reqwest::Client` while preserving the default transport when omitted.
- Accepted OpenAPI drift through the 2026-05-19 review and restored the tracked endpoint snapshot to `62 / 62`, while preserving OpenRouter metadata on normalized API errors and Anthropic Messages stream stop events.

### Version 0.9.0

- Added the canonical `audio().speech()` SDK surface for official `/audio/speech`, with deprecated `tts()` compatibility aliases.
- Expanded workspace, workspace-scoped keys/guardrails, workspace I/O logging, generation content/metadata, and video callback coverage while keeping the endpoint snapshot at `51 / 51`.
- Marked high-churn public types as builder-first/future-proof and documented the `0.8.x -> 0.9.0` migration path.

### Version 0.8.1

- Added typed `POST /tts` support with the canonical `tts()` client surface and a runnable example.
- Added live smoke coverage for `POST /rerank` and `GET /organization/members`, and restored the repo snapshot to `43 / 43` accepted OpenAPI endpoints.
- Added `openrouter-cli organization members list` and aligned the CLI/docs surface with the latest management coverage.

### Version 0.8.0

- Completed the SDK transport migration to `reqwest + rustls` and removed the legacy `surf` / `curl` dependency chain.
- Made the public HTTP error surface backend-neutral and documented the `0.7.x -> 0.8.0` breaking changes.
- Expanded the repo snapshot with rerank, video, and organization-member coverage plus refreshed examples and CLI automation docs.

### Version 0.7.0

- Removed the SDK-level config surface and kept file/profile config out of the core crate.
- Standardized the canonical domain-oriented client docs around the `0.7.x` API surface.
- Improved error normalization for `HTTP 200` payloads that actually contain API error bodies.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full contributor workflow.

At a minimum, if you change public API surface, examples, or docs:

- update the relevant README/docs in the same change
- run `just quality`
- run `just quality-ci` if you touched migration docs, CLI behavior, or CI-aligned release/test flows

Related policies:

- [`docs/policies/maintenance-policy.md`](docs/policies/maintenance-policy.md)
- [`SECURITY.md`](SECURITY.md)
- [`SUPPORT.md`](SUPPORT.md)

## License

MIT. See [LICENSE](LICENSE).
