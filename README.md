# OpenRouter Rust SDK

<div align="center">

A **type-safe**, **async** Rust SDK for the [OpenRouter API](https://openrouter.ai/) - Access 200+ AI models with ease

[![Crates.io](https://img.shields.io/crates/v/openrouter-rs)](https://crates.io/crates/openrouter-rs)
[![Documentation](https://docs.rs/openrouter-rs/badge.svg)](https://docs.rs/openrouter-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[📚 Documentation](https://docs.rs/openrouter-rs) | [🎯 Examples](https://github.com/realmorrisliu/openrouter-rs/tree/main/examples) | [📦 Crates.io](https://crates.io/crates/openrouter-rs)

</div>

## 🌟 What makes this special?

- **🔒 Type Safety**: Leverages Rust's type system for compile-time error prevention
- **⚡ Async/Await**: Built on `tokio` for high-performance concurrent operations
- **🧠 Reasoning Tokens**: Industry-leading chain-of-thought reasoning support
- **🛠️ Tool Calling**: Function calling with typed tools and automatic JSON schema generation
- **🖼️ Vision Support**: Multi-modal content for image analysis with vision models
- **📡 Streaming**: Real-time response streaming with `futures`
- **🧩 Unified Streaming Events**: One event model across chat/responses/messages streams
- **🏗️ Builder Pattern**: Ergonomic client and request construction
- **⚙️ Smart Presets**: Curated model groups for programming, reasoning, and free tiers
- **🎯 Complete Coverage**: All OpenRouter API endpoints supported

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
openrouter-rs = "0.5.2"
tokio = { version = "1", features = ["full"] }
```

### 30-Second Example

```rust
use openrouter_rs::{OpenRouterClient, api::chat::*, types::Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = OpenRouterClient::builder()
        .api_key("your_api_key")
        .build()?;

    // Send chat completion
    let request = ChatCompletionRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .messages(vec![
            Message::new(Role::User, "Explain Rust ownership in simple terms")
        ])
        .build()?;

    let response = client.chat().create(&request).await?;
    println!("{}", response.choices[0].content().unwrap_or(""));

    Ok(())
}
```

## ✨ Key Features

### 🧭 Domain-Oriented Client Surface

Use domain accessors for clearer API boundaries:

```rust
use openrouter_rs::types::PaginationOptions;

let response = client.chat().create(&chat_request).await?;
let models = client.models().list().await?;
let pagination = PaginationOptions::with_offset_and_limit(0, 25);
let keys = client
    .management()
    .list_api_keys(Some(pagination), Some(false))
    .await?;
```

Available domains:
- `client.chat()`
- `client.responses()`
- `client.messages()`
- `client.models()`
- `client.management()`
- `client.legacy()` (requires `legacy-completions` feature)

### 🧩 Unified Streaming Events

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

### 🧱 Legacy Completions (Feature-Gated)

Legacy `POST /completions` support is isolated behind `legacy-completions` and explicit legacy namespace.

```toml
[dependencies]
openrouter-rs = { version = "0.5.2", features = ["legacy-completions"] }
```

```rust
use openrouter_rs::{OpenRouterClient, api::legacy::completion::CompletionRequest};

let request = CompletionRequest::builder()
    .model("deepseek/deepseek-chat-v3-0324:free")
    .prompt("Once upon a time")
    .build()?;

let response = client.legacy().completions().create(&request).await?;
```

Migration mapping:
- `api::completion::CompletionRequest` -> `api::legacy::completion::CompletionRequest`
- `client.send_completion_request(&request)` -> `client.legacy().completions().create(&request)`
- Recommended modern path: `api::chat::ChatCompletionRequest` + `client.chat().create(...)`

### 🔁 0.6 Naming/Pagination Migration

Full migration guide: [MIGRATION.md](./MIGRATION.md)

- `models().count()` -> `models().get_model_count()`
- `models().list_for_user()` -> `models().list_user_models()`
- `management().exchange_code_for_api_key(...)` -> `management().create_api_key_from_auth_code(...)`
- `management().list_guardrails(offset, limit)` -> `management().list_guardrails(Some(PaginationOptions::with_offset_and_limit(offset, limit)))`
- `client.list_api_keys(offset, include_disabled)` -> `management().list_api_keys(Some(PaginationOptions::with_offset(offset)), include_disabled)`

Remaining `0.5.x` transitional aliases (scheduled for removal in `0.6.0`):

| Deprecated (`0.5.x`) | Replacement |
| --- | --- |
| `models().count()` | `models().get_model_count()` |
| `models().list_for_user()` | `models().list_user_models()` |

Migration validation commands for contributors:

```bash
./scripts/check_migration_docs.sh
cargo test --test migration_smoke --all-features
```

### 🧠 Advanced Reasoning Support

Leverage chain-of-thought processing with reasoning tokens:

```rust
use openrouter_rs::types::Effort;

let request = ChatCompletionRequest::builder()
    .model("deepseek/deepseek-r1")
    .messages(vec![Message::new(Role::User, "What's bigger: 9.9 or 9.11?")])
    .reasoning_effort(Effort::High)    // Enable deep reasoning
    .reasoning_max_tokens(2000)        // Control reasoning depth
    .build()?;

let response = client.chat().create(&request).await?;

// Access both reasoning and final answer
println!("🧠 Reasoning: {}", response.choices[0].reasoning().unwrap_or(""));
println!("💡 Answer: {}", response.choices[0].content().unwrap_or(""));
```

### 🛠️ Tool Calling (Function Calling)

Define tools and let models call functions:

```rust
use openrouter_rs::types::Tool;
use serde_json::json;

// Define a tool
let calculator = Tool::builder()
    .name("calculator")
    .description("Perform arithmetic operations")
    .parameters(json!({
        "type": "object",
        "properties": {
            "operation": { "type": "string", "enum": ["add", "subtract", "multiply"] },
            "a": { "type": "number" },
            "b": { "type": "number" }
        },
        "required": ["operation", "a", "b"]
    }))
    .build()?;

// Send request with tools
let request = ChatCompletionRequest::builder()
    .model("anthropic/claude-sonnet-4")
    .messages(vec![Message::new(Role::User, "What's 42 * 17?")])
    .tools(vec![calculator])
    .build()?;

let response = client.chat().create(&request).await?;

// Handle tool calls
if let Some(tool_calls) = response.choices[0].tool_calls() {
    for call in tool_calls {
        println!("Tool: {}, Args: {}", call.function.name, call.function.arguments);
    }
}
```

### 🔧 Typed Tools with Auto Schema Generation

Use Rust structs for type-safe tool definitions:

```rust
use openrouter_rs::types::typed_tool::{TypedTool, TypedToolParams};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WeatherParams {
    /// City and country, e.g. "London, UK"
    pub location: String,
    /// Temperature unit
    pub unit: Option<String>,
}

impl TypedTool for WeatherParams {
    fn name() -> &'static str { "get_weather" }
    fn description() -> &'static str { "Get weather for a location" }
}

// Auto-generates JSON schema from struct
let request = ChatCompletionRequest::builder()
    .model("anthropic/claude-sonnet-4")
    .messages(vec![Message::new(Role::User, "Weather in Paris?")])
    .typed_tool::<WeatherParams>()
    .build()?;
```

### 🖼️ Multi-Modal Vision Support

Send images for analysis with vision models:

```rust
use openrouter_rs::api::chat::ContentPart;

let request = ChatCompletionRequest::builder()
    .model("anthropic/claude-sonnet-4")
    .messages(vec![
        Message::with_parts(Role::User, vec![
            ContentPart::text("What's in this image?"),
            ContentPart::image_url_with_detail("https://example.com/image.jpg", "high"),
        ])
    ])
    .build()?;
```

### 📡 Real-time Streaming

Process responses as they arrive:

```rust
use futures_util::StreamExt;

let stream = client.chat().stream(&request).await?;

stream
    .filter_map(|event| async { event.ok() })
    .for_each(|chunk| async move {
        if let Some(content) = chunk.choices[0].content() {
            print!("{}", content);  // Print as it arrives
        }
    })
    .await;
```

### ⚙️ Smart Model Presets

Use curated model collections:

```rust
use openrouter_rs::config::OpenRouterConfig;

let config = OpenRouterConfig::default();

// Three built-in presets:
// • programming: Code generation and development
// • reasoning: Advanced problem-solving models  
// • free: Free-tier models for experimentation

println!("Available models: {:?}", config.get_resolved_models());
```

### 🛡️ Comprehensive Error Handling

```rust
use openrouter_rs::error::{ApiErrorKind, OpenRouterError};

match client.chat().create(&request).await {
    Ok(response) => println!("Success!"),
    Err(OpenRouterError::Api(api_error)) => match &api_error.kind {
        ApiErrorKind::Moderation { reasons, .. } => {
            eprintln!("Content flagged: {:?}", reasons);
        }
        _ => {
            eprintln!(
                "API error {} (retryable={}): {}",
                api_error.status,
                api_error.is_retryable(),
                api_error.message
            );
        }
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

### 🔐 OAuth PKCE Flow

```rust
use openrouter_rs::{OpenRouterClient, api::auth};

let client = OpenRouterClient::builder()
    .api_key("your_api_key")
    .build()?;

let create = auth::CreateAuthCodeRequest::builder()
    .callback_url("https://myapp.com/auth/callback")
    .code_challenge("your_pkce_code_challenge")
    .code_challenge_method(auth::CodeChallengeMethod::S256)
    .build()?;

let auth_code = client.management().create_auth_code(&create).await?;

let key = client.management()
    .create_api_key_from_auth_code(
        &auth_code.id,
        Some("your_pkce_code_verifier"),
        Some(auth::CodeChallengeMethod::S256),
    )
    .await?;
```

## 📊 API Coverage

| Feature | Status | Module |
|---------|---------|---------|
| Domain-Oriented Client API | ✅ | [`OpenRouterClient`](https://docs.rs/openrouter-rs/latest/openrouter_rs/struct.OpenRouterClient.html) |
| Chat Completions | ✅ | [`api::chat`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/chat/) |
| Legacy Text Completions (`legacy-completions`) | ✅ | `api::legacy::completion` |
| **Tool Calling** | ✅ | [`api::chat`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/chat/) |
| **Typed Tools** | ✅ | [`types::typed_tool`](https://docs.rs/openrouter-rs/latest/openrouter_rs/types/typed_tool/) |
| **Multi-Modal/Vision** | ✅ | [`api::chat`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/chat/) |
| **Reasoning Tokens** | ✅ | [`api::chat`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/chat/) |
| Streaming Responses | ✅ | [`api::chat`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/chat/) |
| Unified Streaming Events | ✅ | [`types::stream`](https://docs.rs/openrouter-rs/latest/openrouter_rs/types/stream/) |
| **Streaming Tool Calls** | ✅ | [`types::stream`](https://docs.rs/openrouter-rs/latest/openrouter_rs/types/stream/) |
| Responses API | ✅ | [`api::responses`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/responses/) |
| Anthropic Messages API | ✅ | [`api::messages`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/messages/) |
| Provider/Activity Discovery | ✅ | [`api::discovery`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/discovery/) |
| Guardrails | ✅ | [`api::guardrails`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/guardrails/) |
| Model Information | ✅ | [`api::models`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/models/) |
| API Key Management | ✅ | [`api::api_keys`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/api_keys/) |
| Credit Management | ✅ | [`api::credits`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/credits/) |
| Authentication | ✅ | [`api::auth`](https://docs.rs/openrouter-rs/latest/openrouter_rs/api/auth/) |

Detailed endpoint-level implementation and test status matrix:
- [`docs/official-endpoint-test-matrix.md`](docs/official-endpoint-test-matrix.md)

`/activity` requires a management key; in this SDK set it with `.management_key(...)`.
`/guardrails*` endpoints also require a management key; in this SDK set it with `.management_key(...)`.
Management-key examples in this repo use `OPENROUTER_MANAGEMENT_KEY`.

## 🎯 More Examples

### Filter Models by Category

```rust
use openrouter_rs::types::ModelCategory;

let models = client
    .models()
    .list_by_category(ModelCategory::Programming)
    .await?;

println!("Found {} programming models", models.len());
```

### Advanced Client Configuration

```rust
let client = OpenRouterClient::builder()
    .api_key("your_key")
    .http_referer("https://yourapp.com")
    .x_title("My AI App")
    .base_url("https://openrouter.ai/api/v1")  // Custom endpoint
    .build()?;
```

### Streaming with Reasoning

```rust
let stream = client.chat().stream(
    &ChatCompletionRequest::builder()
        .model("anthropic/claude-sonnet-4")
        .messages(vec![Message::new(Role::User, "Solve this step by step: 2x + 5 = 13")])
        .reasoning_effort(Effort::High)
        .build()?
).await?;

let mut reasoning_buffer = String::new();
let mut content_buffer = String::new();

stream.filter_map(|event| async { event.ok() })
    .for_each(|chunk| async {
        if let Some(reasoning) = chunk.choices[0].reasoning() {
            reasoning_buffer.push_str(reasoning);
            print!("🧠");  // Show reasoning progress
        }
        if let Some(content) = chunk.choices[0].content() {
            content_buffer.push_str(content);
            print!("💬");  // Show content progress
        }
    }).await;

println!("\n🧠 Reasoning: {}", reasoning_buffer);
println!("💡 Answer: {}", content_buffer);
```

## 📚 Documentation & Resources

- **[📖 API Documentation](https://docs.rs/openrouter-rs)** - Complete API reference
- **[🎯 Examples Repository](https://github.com/realmorrisliu/openrouter-rs/tree/main/examples)** - Comprehensive usage examples
- **[🔧 Configuration Guide](https://docs.rs/openrouter-rs/latest/openrouter_rs/config/)** - Model presets and configuration
- **[⚡ OpenRouter API Docs](https://openrouter.ai/docs)** - Official OpenRouter documentation

### Run Examples Locally

```bash
# Set your API key
export OPENROUTER_API_KEY="your_key_here"

# Basic chat completion
cargo run --example send_chat_completion

# Domain-oriented chat client
cargo run --example domain_chat_completion

# Tool calling (function calling)
cargo run --example basic_tool_calling

# Typed tools with JSON schema generation
cargo run --example typed_tool_calling

# Reasoning tokens demo
cargo run --example chat_with_reasoning

# Streaming responses
cargo run --example stream_chat_completion

# Run with reasoning
cargo run --example stream_chat_with_reasoning

# Streaming with tool calls
cargo run --example stream_chat_with_tools

# Domain-oriented management client
cargo run --example domain_management_api_keys

# Legacy text completions (feature-gated)
cargo run --features legacy-completions --example send_completion_request
```

### Run CLI (Foundation)

```bash
# Show CLI help
cargo run -p openrouter-cli -- --help

# Show resolved profile/config/auth sources
cargo run -p openrouter-cli -- --output json profile show
```

CLI config priority is deterministic:

- flags (`--api-key`, `--management-key`, `--base-url`)
- environment (`OPENROUTER_API_KEY`, `OPENROUTER_MANAGEMENT_KEY`, `OPENROUTER_BASE_URL`)
- profile values in `profiles.toml`
- defaults

See [`crates/openrouter-cli/README.md`](crates/openrouter-cli/README.md) for full config/profile conventions.

### Run Integration Tests

```bash
# Run stable tier (default)
OPENROUTER_API_KEY=sk-or-v1-... cargo test --test integration -- --nocapture

# Run hot-model sweep tier
OPENROUTER_API_KEY=sk-or-v1-... OPENROUTER_INTEGRATION_TIER=hot cargo test --test integration -- --nocapture
```

Model selection and tier env details:

- [`tests/integration/README.md`](tests/integration/README.md)
- [`tests/integration/hot_models.json`](tests/integration/hot_models.json)
- [`scripts/sync_hot_models.sh`](scripts/sync_hot_models.sh)

## 🤝 Community & Support

### 🐛 Found a Bug?

Please [open an issue](https://github.com/realmorrisliu/openrouter-rs/issues/new) with:
- Your Rust version (`rustc --version`)
- SDK version you're using
- Minimal code example
- Expected vs actual behavior

### 💡 Feature Requests

We love hearing your ideas! [Start a discussion](https://github.com/realmorrisliu/openrouter-rs/discussions) to:
- Suggest new features
- Share use cases
- Get help with implementation

### 🛠️ Contributing

Contributions are welcome! Please see our [contributing guidelines](CONTRIBUTING.md):

1. **Fork** the repository
2. **Create** a feature branch
3. **Add** tests for new functionality
4. **Follow** the existing code style
5. **Submit** a pull request

### ⭐ Show Your Support

If this SDK helps your project, consider:
- ⭐ **Starring** the repository
- 🐦 **Sharing** on social media
- 📝 **Writing** about your experience
- 🤝 **Contributing** improvements

## 📋 Requirements

- **Rust**: 1.85+ (2024 edition)
- **Tokio**: 1.0+ (for async runtime)
- **OpenRouter API Key**: [Get yours here](https://openrouter.ai/keys)

## 🗺️ Roadmap

- [ ] **WebSocket Support** - Real-time bidirectional communication
- [ ] **Retry Strategies** - Automatic retry with exponential backoff
- [ ] **Caching Layer** - Response caching for improved performance
- [ ] **CLI Tool** - Command-line interface for quick testing (foundation landed; command groups in progress)
- [ ] **Middleware System** - Request/response interceptors

## 📜 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This is a **third-party SDK** not officially affiliated with OpenRouter. Use at your own discretion.

---

## 📈 Release History

### Version 0.5.2 *(Latest)*

- 🧭 **Added**: Domain-oriented SDK surface and major OpenRouter coverage expansion (`messages`, discovery/activity, guardrails, auth code flow)
- 🛠️ **Added**: `openrouter-cli v0.1` foundation with discovery, management, and usage/billing command groups
- 🔁 **Added**: 0.5.x deprecation bridge + published `0.5.x -> 0.6.0` migration guide and migration smoke harness
- 🌊 **Added**: Unified streaming abstraction across chat/responses/messages and normalized API error model
- ✅ **Improved**: Live integration suite stability (standard `.env` loading, configurable test models, resilient assertions)

### Version 0.5.1

- 🧩 **New**: Multipart text `cache_control` helpers (`text_with_cache_control`, `cacheable_text`, `cacheable_text_with_ttl`)
- 🧠 **Improved**: Reasoning effort now supports `xhigh`, `minimal`, and `none`
- 🛡️ **Security**: Upgraded `bytes` to `1.11.1` (`GHSA-434x-w66g-qw3r`)
- 🔧 **Fixed**: Examples now load API keys at runtime to avoid compile-time `.env` failures in CI

### Version 0.5.0

- 🌊 **New**: Streaming tool calls support with `ToolAwareStream` - automatically accumulates partial tool call fragments
- 🔧 **New**: `PartialToolCall` and `PartialFunctionCall` types for incremental streaming data
- 📡 **New**: `StreamEvent` enum for structured streaming events (`ContentDelta`, `ReasoningDelta`, `Done`, `Error`)
- 🛠️ **New**: `stream_chat_completion_tool_aware()` convenience method on client

### Version 0.4.7

- 🛠️ **New**: Comprehensive tool calling (function calling) support with parallel tool calls
- 🔧 **New**: Typed tools with automatic JSON schema generation via `schemars`
- 🖼️ **New**: Multi-modal content support for vision models (images with detail levels)
- 🐛 **Fixed**: Gemini model compatibility (added missing fields)

### Version 0.4.6

- 🐛 **Fixed**: Grok model deserialization error (Issue #6)
- ➕ **Added**: `index` and `logprobs` fields to Choice structs
- 🧪 **Added**: Grok model integration test and unit tests for response parsing

- 🧠 **New**: Complete reasoning tokens implementation with chain-of-thought support
- ⚙️ **Updated**: Model presets restructured to `programming`/`reasoning`/`free` categories
- 📚 **Enhanced**: Professional-grade documentation with comprehensive examples
- 🏗️ **Improved**: Configuration system with better model management

### Version 0.4.5

- Added: Support for listing models by supported parameters
- Note: OpenRouter API limitations on simultaneous category and parameter filtering

### Version 0.4.4

- Added: Support for listing models by category
- Thanks to OpenRouter team for the API enhancement!

---

<div align="center">

**Made with ❤️ for the Rust community**

[⭐ Star us on GitHub](https://github.com/realmorrisliu/openrouter-rs) | [📦 Find us on Crates.io](https://crates.io/crates/openrouter-rs) | [📚 Read the Docs](https://docs.rs/openrouter-rs)

</div>
