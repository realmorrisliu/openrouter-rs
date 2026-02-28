# Repository Guidelines

## Project Structure & Module Organization
- `src/` contains library code.
- `src/client.rs` is the main SDK client and builder entrypoint.
- `src/api/` holds endpoint modules (`chat`, `completion`, `models`, `api_keys`, `credits`, `auth`, `generation`).
- `src/types/` contains shared request/response types, including streaming and tool-call types.
- `src/config/` contains configuration models and `default_config.toml` presets.
- `tests/unit/` contains fast, local deserialization/config tests.
- `tests/integration/` contains live API tests and shared helpers in `test_utils.rs`.
- `examples/` contains runnable end-to-end usage samples.

## Build, Test, and Development Commands
- `cargo build` builds the crate.
- `cargo check` validates compileability quickly.
- `cargo fmt --all` formats code using `rustfmt`.
- `cargo clippy --all-targets --all-features` runs lints (keep warnings at zero).
- `cargo test --test unit` runs unit tests.
- `OPENROUTER_API_KEY=... cargo test --test integration -- --nocapture` runs live integration tests.
- `cargo run --example send_chat_completion` runs a reference example (see `examples/` for more).

## Coding Style & Naming Conventions
- Follow standard Rust formatting (4-space indentation, `rustfmt` output).
- Use `snake_case` for modules/functions/tests, `PascalCase` for structs/enums/traits, and `SCREAMING_SNAKE_CASE` for constants.
- Preserve existing patterns: builder-style APIs, typed request/response models, and explicit error handling via `Result<_, OpenRouterError>`.
- Keep module boundaries clear (`api` for endpoint logic, `types` for data contracts).

## Testing Guidelines
- Add unit tests for parsing/typing behavior changes in `tests/unit/*.rs`.
- Add integration tests for endpoint behavior changes in `tests/integration/*.rs`.
- Name tests with `test_*` and keep assertions specific.
- Integration tests require `OPENROUTER_API_KEY`; some examples/features also use `OPENROUTER_PROVISIONING_KEY`.

## Commit & Pull Request Guidelines
- Follow existing commit style: `feat:`, `fix:`, `docs:`, `chore:` + concise summary.
- Keep PRs focused; include rationale, behavior changes, and linked issues/PRs.
- For API changes, update examples and docs (`README.md`, `CHANGELOG.md`) in the same PR.
- Before opening a PR, run: `cargo fmt --all`, `cargo clippy --all-targets --all-features`, and relevant tests.

## Security & Configuration Tips
- Use environment variables for secrets; never hardcode API keys.
- Start from `.env.example` for local setup and keep `.env` out of commits.

## Skills
- `openrouter-rs-release`: prepare and publish a new version with synchronized updates to `Cargo.toml`, `CHANGELOG.md`, README release history, and release validation checks. (file: `.agents/skills/openrouter-rs-release/SKILL.md`)

### Skill Trigger Rule
- Use `openrouter-rs-release` when the user asks to publish/release/cut a new version.
