# CLAUDE.md

This file gives repository-specific guidance for coding agents working in `openrouter-rs`.

## Current SDK Shape

The crate is on `0.6.0` and the public documentation treats the domain-oriented surface as canonical:

- `client.chat()`
- `client.responses()`
- `client.messages()`
- `client.models()`
- `client.management()`
- `client.legacy()` behind the `legacy-completions` feature

Hidden flat `OpenRouterClient::*` wrappers still exist in places, but new docs, new examples, and new tests should prefer the domain clients.

## Project Layout

- `src/client.rs`: main client builder plus domain client implementations
- `src/api/`: endpoint modules
  - `chat.rs`
  - `responses.rs`
  - `messages.rs`
  - `models.rs`
  - `embeddings.rs`
  - `discovery.rs`
  - `api_keys.rs`
  - `credits.rs`
  - `auth.rs`
  - `generation.rs`
  - `guardrails.rs`
  - `legacy/completion.rs`
- `src/types/`: shared request/response, stream, pagination, tool, and typed-tool types
- `src/config/`: config loading and built-in model presets
- `crates/openrouter-cli/`: workspace CLI companion
- `tests/unit/`: fast local tests
- `tests/integration/`: live API tests
- `examples/`: runnable examples

## Preferred Commands

Use the `just` recipes when possible:

```bash
just quality
just quality-ci
just test-unit
just test-lib
just test-doc
just test-integration-subsets
just test-integration
just test-live-contract
just test-live-contract-management
just test-cli
just check-migration-docs
just test-migration-smoke
```

Direct cargo commands still used frequently:

```bash
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test unit
OPENROUTER_API_KEY=... cargo test --test integration -- --nocapture
cargo run --example domain_chat_completion
cargo run -p openrouter-cli -- --help
```

## Environment Variables

- `OPENROUTER_API_KEY`: required for examples and most live integration coverage
- `OPENROUTER_MANAGEMENT_KEY`: required for key/guardrail/activity management flows
- `OPENROUTER_RUN_MANAGEMENT_TESTS=1`: opt into live create/update/delete management smoke
- `OPENROUTER_INTEGRATION_TIER=stable|hot`: switch integration model pool tier
- `OPENROUTER_CLI_RUN_LIVE=1`: enable CLI live smoke
- `OPENROUTER_CLI_RUN_LIVE_WRITE=1`: also enable CLI write-path smoke

## Implementation Notes

- Request and client construction use the builder pattern (`derive_builder`).
- The crate uses `surf` for HTTP and `tokio` for async runtime.
- Streaming is exposed in three forms:
  - raw endpoint streams
  - `ToolAwareStream` for assembled tool calls
  - `UnifiedStreamEvent` across chat, responses, and messages
- Built-in config presets come from `src/config/default_config.toml`.
- `legacy-completions` is opt-in in `0.6.x`.

## Documentation Expectations

- Keep `README.md`, `MIGRATION.md`, example code, and CLI docs aligned with the current domain-client design.
- If you touch migration naming or pagination guidance, run:

```bash
./scripts/check_migration_docs.sh
cargo test --test migration_smoke --all-features
```

- If you change CLI behavior, update `crates/openrouter-cli/README.md` and run `cargo test -p openrouter-cli`.
