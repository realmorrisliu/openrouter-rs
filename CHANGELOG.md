# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
