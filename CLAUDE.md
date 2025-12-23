# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## OpenRouter Rust SDK

This is a third-party Rust SDK for the OpenRouter API, providing type-safe and memory-safe integration. The project uses the builder pattern, supports streaming responses, comprehensive error handling, and asynchronous operations.

## Project Architecture

### Core Module Structure
- `src/client.rs` - Main client implementation using derive_builder pattern
- `src/api/` - OpenRouter API endpoint implementations
  - `chat.rs` - Chat completions and streaming responses
  - `models.rs` - Model management and filtering
  - `api_keys.rs` - API key management
  - `completion.rs` - Text completions
  - `credits.rs` - Credit management
  - `generation.rs` - Generation data
  - `auth.rs` - Authentication
  - `errors.rs` - API error handling
- `src/types/` - Type definitions and data structures
  - `completion.rs` - Completion response types
  - `provider.rs` - Provider information
  - `response_format.rs` - Response format definitions
- `src/config/` - Configuration management
  - `model.rs` - Model configuration structures
  - `default_config.toml` - Default configuration file with preset model lists
- `src/error.rs` - Unified error type definitions
- `src/utils.rs` - Utility functions

### Key Design Patterns
1. **Builder Pattern**: Both client and requests use builder pattern for creation
2. **Async Streaming**: Uses `BoxStream` and `futures_util` for streaming responses
3. **Type Safety**: Strongly typed request/response structures with model category and parameter filtering
4. **Error Handling**: Custom error types covering API errors, network errors, and validation errors

## Development Commands

### Basic Build and Test
```bash
# Build project
cargo build

# Run unit tests
cargo test --test unit

# Run integration tests (requires API key)
OPENROUTER_API_KEY=your_key cargo test --test integration -- --nocapture

# Run all tests
cargo test
```

### Running Examples
```bash
# Basic chat completion
cargo run --example send_chat_completion

# Streaming chat completion
cargo run --example stream_chat_completion

# Reasoning tokens (new in 0.4.5)
cargo run --example chat_with_reasoning
cargo run --example stream_chat_with_reasoning

# Get model list
cargo run --example list_models

# List models by category
cargo run --example list_models_by_category

# Filter models by parameters
cargo run --example list_models_by_parameters

# API key management
cargo run --example list_api_keys
cargo run --example get_current_api_key_info

# Credit management
cargo run --example get_credits
```

### Code Formatting and Checks
```bash
# Format code
cargo fmt

# Check code
cargo check

# Clippy linting
cargo clippy
```

## Configuration Management

### Environment Variables
- `OPENROUTER_API_KEY` - OpenRouter API key (required for tests and examples)

### Default Configuration
The project uses `src/config/default_config.toml` to define default models and presets:
- `default_model` - Default model to use
- `models.presets` - Model preset groups (programming, reasoning, free)

## API Usage Patterns

### Client Creation (Recommended Builder Pattern)
```rust
let client = OpenRouterClient::builder()
    .api_key("your_api_key")
    .http_referer("https://yourdomain.com")
    .x_title("Your App Name")
    .build()?;
```

### Request Building
```rust
let request = ChatCompletionRequest::builder()
    .model("anthropic/claude-sonnet-4")
    .messages(vec![Message::new(Role::User, "Hello")])
    .temperature(0.7)
    .max_tokens(200)
    .build()?;
```

### Reasoning Tokens (New in 0.4.5)
```rust
use openrouter_rs::types::Effort;

let request = ChatCompletionRequest::builder()
    .model("deepseek/deepseek-r1")
    .messages(vec![Message::new(Role::User, "What's bigger: 9.9 or 9.11?")])
    .reasoning_effort(Effort::High)
    .reasoning_max_tokens(1000)
    .build()?;

let response = client.send_chat_completion(&request).await?;
println!("Reasoning: {}", response.choices[0].reasoning().unwrap_or(""));
println!("Answer: {}", response.choices[0].content().unwrap_or(""));
```

### Streaming Response Handling
Use `futures_util::StreamExt` to process streaming data, filtering errors and extracting delta content.

## Test Structure

### Integration Tests (`tests/integration/`)
- `chat.rs` - Chat completion tests
- `models.rs` - Model management tests
- `api_keys.rs` - API key tests
- `test_utils.rs` - Test utility functions

### Unit Tests (`tests/unit/`)
- `config.rs` - Configuration loading tests

## Version Information

Current version: 0.4.6
- 🐛 **Fixed**: Grok model deserialization error (Issue #6)
- ➕ **Added**: `index` and `logprobs` fields to Choice structs
- 🧪 **Added**: Grok model integration test and unit tests for response parsing
- Previous: Complete reasoning tokens implementation, model presets restructuring

## Development Guidelines

1. **API Key Security**: Example code uses `dotenvy_macro::dotenv!` to read from environment variables, avoiding hardcoding
2. **Async Processing**: All API calls are asynchronous using `tokio` runtime
3. **Error Handling**: Uses `thiserror` for structured error type definitions
4. **HTTP Client**: Uses `surf` as the HTTP client
5. **Serialization**: Uses `serde` for JSON serialization/deserialization

## Current Development Focus

Main areas of focus:
- WebSocket support for real-time communication
- Retry strategies with exponential backoff
- Response caching layer
- CLI tool development
- Middleware system for request/response interceptors