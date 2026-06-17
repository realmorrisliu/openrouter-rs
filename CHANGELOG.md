# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added typed SDK support for the Files API (`GET|POST /files`, `GET|DELETE /files/{file_id}`, and `GET /files/{file_id}/content`) via `api::files` and the canonical `client.files()` surface.
- Added typed management-key SDK support for analytics metadata and query endpoints (`GET /analytics/meta`, `POST /analytics/query`) via `api::analytics` and `client.management().get_analytics_meta(...)` / `query_analytics(...)`.
- Added typed SDK support for `GET /datasets/app-rankings`, `GET /datasets/benchmarks/artificial-analysis`, and `GET /datasets/benchmarks/design-arena` through the `models()` domain client.
- Added typed SDK support for `GET /model/{author}/{slug}` and expanded `GET /models` filtering through `ListModelsParams`.
- Added management-key SDK support for listing, reading, and versioning presets via `GET /presets`, `GET /presets/{slug}`, `GET /presets/{slug}/versions`, and `GET /presets/{slug}/versions/{version}`.
- Added typed SDK support for `GET /datasets/rankings-daily`, including `api::discovery::RankingsDailyResponse` and the canonical `client.models().get_rankings_daily(...)` surface.
- Added management-key SDK support for creating or updating presets from inference request bodies via `POST /presets/{slug}/chat/completions`, `POST /presets/{slug}/responses`, and `POST /presets/{slug}/messages`.
- Added `AnthropicRole::System` and `AnthropicMessage::system(...)` for the refreshed `/messages` role schema.
- Added typed `GenerationData::preset_id` support on `GET /generation` metadata responses.

### Changed
- Accepted the 2026-06-16 OpenAPI drift review, including analytics, files, app rankings, benchmark datasets, singular model lookup, preset read/version endpoints, model filter/schema refreshes, rerank multimodal documents, and video input reference refreshes, restoring the repository snapshot to `81 / 81` official OpenAPI endpoint coverage.
- Changed the opt-in OpenRouter metadata request header sent by chat, Responses, and Messages requests from `X-OpenRouter-Experimental-Metadata` to the upstream `X-OpenRouter-Metadata` spelling while preserving the existing request-builder method names.
- Refreshed model and generation metadata deserialization for nullable model context lengths, model links/benchmarks, default parameters, generation `data_region`, floating-point latency, and integer status values.
- Refreshed rerank response document echoes so successful image-only multimodal rerank results deserialize with optional `text` and `image` fields.
- Accepted the 2026-06-01 OpenAPI drift review, including daily rankings datasets, preset creation endpoints, generation metadata additions, and provider taxonomy refreshes, restoring the repository snapshot to `66 / 66` official OpenAPI endpoint coverage.

## [0.10.0] - 2026-05-20

### Added
- `OpenRouterClientBuilder::http_client(...)` — allow injecting a custom `reqwest::Client`. Enables HTTP/SOCKS proxies (e.g. for geo-restricted routes), custom timeouts, retry/tracing middleware, mTLS, and other transport-layer customization. The internal default client is preserved when `.http_client(...)` is not called, so existing code is unaffected. See `examples/custom_http_client.rs`.
- Added `OpenRouterExperimentalMetadata` request-header opt-in for chat completions, Responses API, and Anthropic-compatible Messages requests, plus typed chat response access to `service_tier` and `openrouter_metadata`.
- Added typed SDK support for BYOK provider credential management (`GET|POST /byok`, `GET|PATCH|DELETE /byok/{id}`) via `api::byok` and `client.management().*_byok_key(...)` methods.
- Added typed SDK support for observability destination management (`GET|POST /observability/destinations`, `GET|PATCH|DELETE /observability/destinations/{id}`) via `api::observability` and `client.management().*_observability_destination(...)` methods.
- Added embedding multimodal media helpers for audio, video, and file content parts through `EmbeddingContentPart::input_audio(...)`, `input_video(...)`, and `input_file(...)`.
- Added typed guardrail content-filter support, including built-in filter entries, custom regex filters, and provider-specific ZDR flags on guardrail create/update requests and guardrail responses.
- Added typed `supported_voices` model discovery fields and `GenerationData::service_tier` for generation metadata responses.
- Added typed SDK support for `POST /audio/transcriptions`, including `api::audio::{TranscriptionRequest, TranscriptionInputAudio, TranscriptionResponse}` and the canonical `client.audio().transcriptions().create(...)` surface.
- Added typed OpenRouter chat-completion usage cost fields via `ResponseUsage::cost`, `ResponseUsage::cost_details`, `ResponseUsage::is_byok`, and `ResponseCostDetails`.
- Added ergonomic constructors for non-exhaustive helper types such as `ResponseUsage::new`, `ToolCall::new`, `FunctionCall::new`, `JsonSchemaConfig::new`, embedding multimodal parts, and provider-options wrappers.

### Changed
- Breaking: Marked high-churn public SDK request, response, metadata, usage, pricing, discovery, streaming, and upstream taxonomy types as `#[non_exhaustive]`; construct affected request/configuration types through builders, constructors, or helpers, and include wildcard arms when matching affected public enums outside the crate.
- Accepted the 2026-04-29 OpenAPI drift review, including `stt` generation origins and chat usage cost metadata, and kept the repository snapshot at `51 / 51` official OpenAPI endpoint coverage.
- Accepted the 2026-05-03 OpenAPI drift review, including audio transcription support, and restored the repository snapshot to `52 / 52` official OpenAPI endpoint coverage.
- Accepted the 2026-05-15 OpenAPI drift review, including experimental response metadata, guardrail content filters, model supported voices, and generation service tiers, while keeping the repository snapshot at `52 / 52` official OpenAPI endpoint coverage.
- Accepted the 2026-05-19 OpenAPI drift review, including BYOK provider credentials, observability destinations, embeddings multimodal media parts, and Responses schema refreshes, restoring the repository snapshot to `62 / 62` official OpenAPI endpoint coverage.
- Changed the scheduled OpenAPI drift workflow from daily to weekly while keeping manual `workflow_dispatch` runs available.

### Fixed
- Preserved top-level `openrouter_metadata` and `user_id` fields on normalized API errors so guardrail-blocked responses keep their diagnostic metadata.
- Preserved `openrouter_metadata` and future top-level extras on Anthropic Messages `message_stop` stream events.

## [0.9.0] - 2026-04-28

### Added
- Added typed SDK support for `GET /generation/content`, including `client.get_generation_content(...)` and `client.management().get_generation_content(...)`.
- Added typed `num_fetches` support on `GET /generation` metadata responses via `GenerationData`.
- Added typed `response_cache_source_id` support on `GET /generation` metadata responses.
- Added workspace I/O logging fields on typed workspace responses and create/update requests: `io_logging_api_key_ids` and `io_logging_sampling_rate`.
- Added `callback_url` support on `VideoGenerationRequest` for video completion webhooks.
- Added typed SDK support for `GET /workspaces`, `POST /workspaces`, `GET /workspaces/{id}`, `PATCH /workspaces/{id}`, `DELETE /workspaces/{id}`, `POST /workspaces/{id}/members/add`, and `POST /workspaces/{id}/members/remove`, including canonical `client.management()` methods and a runnable `examples/list_workspaces.rs`.
- Added workspace-aware API-key and guardrail support, including `workspace_id` fields on typed models plus `create_api_key_in_workspace(...)`, `list_api_keys_in_workspace(...)`, and `list_guardrails_in_workspace(...)`.
- Added `openrouter-cli workspaces ...` commands, plus `--workspace-id` support for `openrouter-cli keys list|create` and `openrouter-cli guardrails list|create`.
- Added `openrouter-cli workspaces create|update --io-logging-api-key-id`, `--io-logging-sampling-rate`, and `workspaces update --clear-io-logging-api-key-ids` so CLI workspace I/O logging controls match the SDK request surface.
- Added the canonical SDK audio speech surface: `api::audio::{SpeechRequest, SpeechResponseFormat, SpeechProviderOptions}`, `api::audio::create_speech(...)`, and `client.audio().speech().create(...)`.

### Changed
- Breaking: `client.tts().create(...)`, `api::tts::TtsRequest`, `api::tts::TtsResponseFormat`, and `api::tts::create_tts(...)` are now deprecated compatibility aliases; new code should use the canonical `client.audio().speech().create(...)` and `api::audio` names.
- Breaking: newly added and high-churn public request/response types for audio speech, workspace management, workspace-aware keys, generation metadata/content, and video generation are marked `#[non_exhaustive]` where appropriate; use builders for request construction instead of public struct literals.
- The canonical audio speech path targets official `POST /audio/speech` and falls back to legacy `POST /tts` only for route-unavailable signals, including generic plain-text `404/405` status pages, so request-level `404/405` errors on the official endpoint are still preserved.
- Accepted the 2026-04-21 OpenAPI drift review, refreshed the tracked compatibility surfaces, and restored the repository snapshot to `51 / 51` official OpenAPI endpoint coverage.
- Accepted the 2026-04-22 OpenAPI drift review, refreshed the tracked baseline, and kept the repository snapshot at `51 / 51` official OpenAPI endpoint coverage.
- Accepted the 2026-04-23 OpenAPI drift review, refreshed the tracked baseline, and kept the repository snapshot at `51 / 51` official OpenAPI endpoint coverage.
- Accepted the 2026-04-28 OpenAPI drift review, refreshed the tracked baseline, and kept the repository snapshot at `51 / 51` official OpenAPI endpoint coverage.
- Nightly OpenAPI drift reports now keep the raw upstream diff but separately classify changes already covered by the SDK's global request-metadata handling, dynamic `String` taxonomy fields, provider options maps, and Responses `Option`/`Value` parsing, reducing false-positive follow-up noise.
- OpenAPI drift classification now also recognizes flexible plugin payloads, Anthropic Messages hosted-tool option payloads, and Responses tool/output `Value` payloads.

## [0.8.1] - 2026-04-21

### Added
- Added typed SDK support for `POST /tts`, including the canonical `client.tts().create(...)` surface, raw audio-byte handling, and a runnable example.
- Added live smoke coverage for `POST /rerank` and read-only management coverage for `GET /organization/members`.
- Added `openrouter-cli organization members list` for management-key-backed organization member discovery.

### Changed
- Added `X-OpenRouter-Categories` request metadata support across the existing attribution-enabled SDK request surfaces.
- Restored the repository snapshot to `43 / 43` official OpenAPI endpoint coverage and aligned the endpoint matrix, README, and docs.rs surface with the newly implemented text-to-speech endpoint.


## [0.8.0] - 2026-04-18

### Added
- Added an `axum` gateway example and a practical typed-tool agent example so the repository covers copyable Rust application patterns instead of only endpoint-level demos.
- Added `docs/cli-automation-workflows.md` with JSON-first shell and CI recipes for discovery, usage reporting, and ephemeral key automation.
- Added `docs/compatibility-update-policy.md` and a reusable upstream-compatibility issue template so upstream OpenRouter changes can be tracked outside normal release cuts.
- Added typed SDK support for `POST /rerank`, `POST /videos`, `GET /videos/models`, `GET /videos/{jobId}`, `GET /videos/{jobId}/content`, and `GET /organization/members`, including canonical `rerank()`, `videos()`, and `management().list_organization_members(...)` surfaces.
- Added runnable examples for rerank requests, video generation submission, and organization-member listing.

### Changed
- Completed the SDK HTTP transport migration from `surf -> isahc -> curl` to a Tokio-native `reqwest + rustls` stack while keeping the canonical domain-oriented client surface and SSE semantics intact.
- Normalized the public HTTP error surface around backend-neutral `HttpRequestError` and `http::StatusCode` so transport details no longer leak through public SDK types.
- Reorganized the README example/docs surface around application patterns, Tokio streaming, and CLI automation workflows.
- Aligned OpenAPI drift follow-up docs and issue wording with the new compatibility-update cadence and reporting surfaces.
- Accepted the 2026-04-18 upstream OpenAPI drift baseline and added typed embedding usage support for `prompt_tokens_details`.
- Restored the repository snapshot to `42 / 42` official endpoint coverage and aligned the endpoint matrix, README, docs.rs surface, and Awesome OpenRouter submission materials with the newly implemented rerank, videos, and organization-member endpoints.

### Removed
- Removed `surf` from the SDK dependency graph, including the transitive `curl` chain that came from the legacy transport stack.
- Removed the public `utils` transport shims `with_bearer_auth`, `with_request_metadata`, `with_client_request_headers`, and `handle_error`. Callers that need custom HTTP glue should keep it in their own application code and treat `openrouter-rs` as the typed domain client layer.

## [0.7.0] - 2026-03-16

### Removed
- Removed the SDK-level `config` module and related client surface. `openrouter-rs` no longer exports `OpenRouterConfig` / `ModelConfig`, no longer stores `.config(...)` on `OpenRouterClient`, and no longer treats file/profile config as part of the core SDK API. That behavior now belongs in the companion CLI or the caller's own application layer.

### Changed
- Refreshed the canonical SDK docs around the `0.7.x` domain-oriented client surface and kept file/profile config explicitly outside the SDK core.
- Hot live integration coverage now runs as a Responses-first sweep and validates candidate hot models with a minimal Responses API health check before they enter the pool.

### Fixed
- Assistant `content` values that arrive as structured objects or arrays now surface text correctly in chat completion helpers when they contain text parts.
- `HTTP 200` responses that actually contain `{ "error": ... }` payloads now normalize into `OpenRouterError::Api` with provider/API status information instead of surfacing as generic deserialization failures.

## [0.6.1] - 2026-03-12

### Changed
- Standardized contributor/release verification around `just quality` and `just quality-ci`.
- Added live contract release validation coverage and refreshed the README/CLI docs to match the current canonical `0.6.x` domain-oriented surface.
- `openrouter-cli` release automation now runs through tag-driven GitHub Actions publishing with prebuilt binary assets, and the crate now aligns on `openrouter-rs` `0.6.1`.

### Fixed
- `ToolBuilder` now preserves accumulated fields regardless of setter call order.
- Combined model filters and model resolution ordering now preserve caller intent.
- Chat streaming requests now inherit default client headers.
- SSE frame parsing is more resilient across chat, responses, and messages streaming flows.
- Response parsing failures now normalize into consistent API error context across endpoints.

## [0.6.0] - 2026-03-10

### Removed
- Removed management-key compatibility aliases:
  - `OpenRouterClientBuilder::provisioning_key(...)`
  - `OpenRouterClient::{set_provisioning_key, clear_provisioning_key}`
- Removed deprecated flat API-key pagination shim:
  - `OpenRouterClient::list_api_keys(offset: Option<f64>, include_disabled: Option<bool>)`
- Removed deprecated management auth-code alias:
  - `ManagementClient::exchange_code_for_api_key(...)`
- Removed deprecated legacy-completion compatibility entrypoints:
  - `api::completion::*`
  - `OpenRouterClient::send_completion_request(...)`
- Removed deprecated model-domain aliases:
  - `ModelsClient::list_for_user()`
  - `ModelsClient::count()`

### Changed
- `legacy-completions` is now opt-in (no longer in default crate features).
- Migration docs and smoke coverage now align with the final canonical 0.6 API surface.

## [0.5.2] - 2026-03-01

### Added
- Anthropic-compatible `/messages` API support:
  - `api::messages` module with typed request/response models
  - non-streaming `create_message` and streaming `stream_messages`
  - `OpenRouterClient::{create_message,stream_messages}` wrappers
  - new examples: `create_message.rs` and `stream_messages.rs`
- Discovery and activity endpoint support:
  - `api::discovery` module for `/providers`, `/models/user`, `/models/count`, `/endpoints/zdr`, `/activity`
  - `OpenRouterClient` wrappers for each endpoint
  - management-key requirement documented for `GET /activity` (`.management_key(...)`)
- OAuth auth-code creation support:
  - add `POST /auth/keys/code` request/response types and client wrapper (`create_auth_code`)
  - add PKCE end-to-end doc snippet (`create_auth_code` -> `exchange_code_for_api_key`)
- Guardrails endpoint support:
  - `api::guardrails` module for `/guardrails` and all guardrail assignment endpoints
  - `OpenRouterClient` wrappers for create/read/update/delete and key/member assignment flows
  - management-key requirement documented for guardrail endpoints (`.management_key(...)`)
- Management-key naming alignment:
  - renamed `OpenRouterClient` builder/config surface from `provisioning_key` to `management_key`
  - renamed management-key helpers to `set_management_key` / `clear_management_key`
  - API-key management and governance endpoints consistently require `management_key`
- Domain-oriented client surface:
  - added domain accessors: `chat()`, `responses()`, `messages()`, `models()`, `management()`
  - added typed domain clients with endpoint methods grouped by API domain
  - added domain-oriented examples for chat and management workflows
- `openrouter-cli` foundation (workspace crate):
  - added command bootstrap with `--help`, `profile show`, and `config show/path`
  - added deterministic config/auth resolution order: flags > env > profile config > defaults
  - added profile/config path conventions and CLI-specific tests
  - added OR-20 discovery commands:
    - `models list|show|endpoints`
    - `providers list`
    - `models list` supports `--category` and `--supported-parameter` filters
  - discovery command output now supports both machine-readable JSON and human-readable table text
  - added OR-21 management commands:
    - `keys`: `list/create/get/update/delete`
    - `guardrails`: `list/create/get/update/delete`
    - `guardrails assignments`: `keys|members` with `list/assign/unassign`
- added OR-22 usage/billing commands:
  - `credits show`
  - `credits charge --amount --sender --chain-id`
  - `usage activity --date`
- `0.5.x` deprecation bridge for planned `0.6.0` removals/renames:
  - restored deprecated `provisioning_key` compatibility aliases:
    - `OpenRouterClientBuilder::provisioning_key(...)`
    - `OpenRouterClient::{set_provisioning_key, clear_provisioning_key}`
  - restored deprecated `api::completion` module alias to `api::legacy::completion`
  - added deprecated domain-method aliases:
    - `models().count()` -> `models().get_model_count()`
    - `models().list_for_user()` -> `models().list_user_models()`
  - `management().exchange_code_for_api_key(...)` -> `management().create_api_key_from_auth_code(...)`
  - `list_api_keys` now accepts legacy `Option<f64>` offset inputs as a deprecated compatibility bridge
  - `legacy-completions` is re-enabled in default features for transitional `0.5.x` compatibility
- Migration guidance for `0.5.x` -> `0.6.0`:
  - added `MIGRATION.md` with old->new API mapping tables
  - documented top migration recipes with before/after snippets across auth, models, management, and legacy completions
  - linked migration guide from README
- Migration verification harness:
  - added `tests/migration_smoke.rs` covering representative flat (`0.5`-style) and domain (`0.6`-style) call paths
  - added `scripts/check_migration_docs.sh` to validate migration mapping sections/snippets in docs
  - CI now runs a dedicated `Migration Smoke Checks` job (`docs check + cargo test --test migration_smoke --all-features`)
  - documented migration validation commands in README for contributors
- Unified streaming abstraction across chat/responses/messages:
  - new `types::stream::{UnifiedStreamEvent, UnifiedStreamSource, UnifiedStream}`
  - adapters: `adapt_chat_stream`, `adapt_responses_stream`, `adapt_messages_stream`
  - new domain methods: `chat().stream_unified(...)`, `responses().stream_unified(...)`, `messages().stream_unified(...)`
- Normalized API error model:
  - new `error::{ApiErrorContext, ApiErrorKind}`
  - `OpenRouterError::Api(...)` now consistently carries status/api_code/message/request_id
  - added retryability helpers via `ApiErrorContext::is_retryable()`
- CI now runs `cargo test -p openrouter-cli` for CLI startup/config coverage

### Changed
- Breaking (planned for `0.6.0`) legacy completions isolation:
  - moved legacy completions to `api::legacy::completion` behind the `legacy-completions` feature
  - added explicit legacy client namespace: `client.legacy().completions().create(...)`
  - updated docs/migration mapping from old completion calls to legacy namespace and modern chat APIs
- Breaking (planned for `0.6.0`) method/pagination consistency:
  - unified `ManagementClient` and `ModelsClient` naming on `create_*`/`get_*`/`list_*`/`delete_*`/`stream_*` conventions
  - introduced shared `types::PaginationOptions` for paginated endpoints
  - updated paginated API signatures (`api_keys`, `guardrails`, client wrappers) to use `PaginationOptions`
- CLI output modes standardized on `table|json` (`table` default, `text` alias retained)
- JSON CLI outputs now use versioned envelopes (`schema_version: "0.1"`) with structured JSON error payloads
- deprecation warnings now point to concrete replacement APIs and planned removal in `0.6.0`
- integration tests now load `.env` automatically and allow model overrides via:
  - `OPENROUTER_TEST_CHAT_MODEL`
  - `OPENROUTER_TEST_REASONING_MODEL`

### Fixed
- Unified messages tool-start events now preserve `content_block_start.index` to keep tool chunks correlatable.
- Unified responses stream now only terminates on `response.completed` (avoids premature close on non-terminal `*.completed` events).
- `guardrails update` now supports explicit allowlist clearing via `--clear-allowed-providers` and `--clear-allowed-models`.
- integration API-key metadata assertions now accept sentinel rate-limit values from live API responses.
- live chat/reasoning integration assertions no longer assume prompt echoing in model outputs.

## [0.5.1] - 2026-02-28

### Added
- Support `cache_control` on multipart text content via `ContentPart::text_with_cache_control`, `ContentPart::cacheable_text`, and `ContentPart::cacheable_text_with_ttl`.

### Changed
- Extended reasoning effort support to include `xhigh`, `minimal`, and `none`.

### Fixed
- Updated examples to read `OPENROUTER_API_KEY` and `OPENROUTER_MANAGEMENT_KEY` at runtime (instead of compile-time `.env` macro expansion), preventing CI/build failures.
- Bumped `bytes` from `1.10.1` to `1.11.1` to address `GHSA-434x-w66g-qw3r` (`CVE-2026-25541`).

## [0.5.0] - 2026-02-25

### Added
- **Streaming tool calls support** ([#15](https://github.com/realmorrisliu/openrouter-rs/pull/15), [@svent](https://github.com/svent))
  - New `ToolAwareStream` wrapper for handling tool calls in streaming responses
  - New `PartialToolCall` and `PartialFunctionCall` types for incremental fragments
  - New `StreamEvent` enum with `ContentDelta`, `ReasoningDelta`, `ReasoningDetailsDelta`, `Done`, `Error` variants
  - New `OpenRouterClient::stream_chat_completion_tool_aware()` convenience method
  - New example `stream_chat_with_tools.rs` demonstrating the feature

### Changed
- **Breaking**: `Delta.tool_calls` changed from `Option<Vec<ToolCall>>` to `Option<Vec<PartialToolCall>>`
- **Breaking**: `Choice::tool_calls()` now returns `None` for streaming responses (use `Choice::partial_tool_calls()` or `ToolAwareStream` instead)

## [0.4.7] - 2025-02-25

### Added
- Documentation updates for v0.4.7 features

### Fixed
- Add missing fields for Gemini model compatibility ([#12](https://github.com/realmorrisliu/openrouter-rs/pull/12))

## [0.4.6] - 2025-02-24

### Added
- Typed tools support with automatic JSON schema generation
- Comprehensive tool calling support for OpenRouter API
- Multi-modal content support for vision models

### Fixed
- Enhanced completion types to support Grok-specific fields, reasoning details, and logprobs

## [0.4.5] - 2025-02-21

### Added
- Complete reasoning tokens implementation
- Support for filtering models by supported parameters

### Fixed
- Fixed all clippy warnings

## [0.4.4] - 2025-02-19

### Added
- Initial implementation of reasoning tokens support

## [0.4.3] - 2025-02-18

### Fixed
- Fixed response deserialization issues with certain models

## [0.4.2] - 2025-02-17

### Fixed
- Fixed streaming response handling

## [0.4.1] - 2025-02-16

### Fixed
- Documentation improvements

## [0.4.0] - 2025-02-15

### Added
- Initial release with async OpenRouter API support
- Chat completions and streaming
- Model listing and filtering
- Builder pattern for ergonomic API usage
