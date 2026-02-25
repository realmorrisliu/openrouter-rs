# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
