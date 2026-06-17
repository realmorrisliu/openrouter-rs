# Repository Guidelines

## Project Structure And Module Organization

- `src/client.rs` is the main SDK entrypoint and defines the canonical domain clients:
  - `chat()`
  - `responses()`
  - `messages()`
  - `rerank()`
  - `audio()`
  - `videos()`
  - `files()`
  - `models()`
  - `management()`
  - `legacy()` behind `legacy-completions`
- `src/api/` contains endpoint modules:
  - `analytics`
  - `api_keys`
  - `audio`
  - `auth`
  - `byok`
  - `chat`
  - `credits`
  - `discovery`
  - `embeddings`
  - `files`
  - `generation`
  - `guardrails`
  - `legacy/completion`
  - `messages`
  - `models`
  - `observability`
  - `organization`
  - `presets`
  - `rerank`
  - `responses`
  - `videos`
  - `workspaces`
- `src/types/` contains shared request/response, streaming, pagination, tool, and typed-tool types.
- `crates/openrouter-cli/` contains the workspace CLI companion, including file/profile config resolution.
- `tests/unit/` contains fast serde/config/domain tests.
- `tests/integration/` contains live API tests plus shared helpers in `test_utils.rs`.
- `examples/` contains runnable usage samples. Prefer the domain-oriented examples when updating docs.
- `openspec/` contains OpenSpec-managed proposals, spec deltas, designs, and task lists for planned requirements.

## Build, Test, And Development Commands

Prefer the `just` recipes so local workflows stay aligned with CI:

- `just quality`: format check, `cargo check`, clippy, unit tests, lib tests, doctests
- `just quality-ci`: `just quality` plus integration subsets, CLI checks, and migration smoke
- `just test-unit`
- `just test-lib`
- `just test-doc`
- `just test-integration-subsets`
- `just test-integration`
- `just test-live-contract`
- `just test-live-contract-management`
- `just test-cli`
- `just check-migration-docs`
- `just test-migration-smoke`

Direct cargo entrypoints:

- `cargo build`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test unit`
- `OPENROUTER_API_KEY=... cargo test --test integration -- --nocapture`
- `cargo run --example domain_chat_completion`
- `cargo run -p openrouter-cli -- --help`
- `openspec validate <change-name>`

## Coding Style And Naming Conventions

- Follow standard Rust formatting with `rustfmt`.
- Use `snake_case` for modules/functions/tests.
- Use `PascalCase` for structs/enums/traits.
- Use `SCREAMING_SNAKE_CASE` for constants.
- Keep new public API examples aligned with the canonical domain-oriented surface rather than hidden flat wrappers.
- Preserve builder-style request construction and explicit `Result<_, OpenRouterError>` handling.

## Testing Guidelines

- Add unit tests in `tests/unit/*.rs` for parsing, request-shape, or domain-surface changes.
- Add integration tests in `tests/integration/*.rs` for live endpoint behavior changes.
- Live integration requires `OPENROUTER_API_KEY`.
- Management smoke requires `OPENROUTER_MANAGEMENT_KEY` and `OPENROUTER_RUN_MANAGEMENT_TESTS=1`.
- CLI live smoke uses `OPENROUTER_CLI_RUN_LIVE=1`; write smoke also requires `OPENROUTER_CLI_RUN_LIVE_WRITE=1`.
- If you touch migration docs or canonical naming guidance, run `just check-migration-docs` and `cargo test --test migration_smoke --all-features`.
- If you change public SDK request/response types, domain client surfaces, CLI behavior, migration docs, or CI-aligned release/test surfaces, run `just quality-ci` before opening or updating a PR.

## OpenAPI Drift Workflow

- For OpenAPI drift issues, inspect the issue/drift report first, implement accepted SDK surface changes, then run `just openapi-refresh-baseline`.
- Always finish drift work with `just openapi-drift-check`; the expected final state is `added=0, removed=0, changed=0`.
- If upstream OpenAPI changes again before PR checks run, refresh the tracked baseline again only after confirming no new SDK/API work is required.
- Update `README.md`, `CHANGELOG.md`, and `docs/operations/official-endpoint-test-matrix.md` when accepted endpoint coverage or public API surface changes.
- When upstream schema drift changes public SDK field types or default serialization, audit `crates/openrouter-cli/` call sites and snapshot tests that consume those SDK types.

## Commit And Pull Request Guidelines

- Follow the existing commit style: `feat:`, `fix:`, `docs:`, `chore:`.
- PR titles must use the same conventional prefix style (`feat:`, `fix:`, `docs:`, etc.); do not prefix titles with `[codex]`.
- Keep PRs focused and include rationale plus behavior changes.
- Do not close pull requests automatically unless the user explicitly asks for that action.
- When API surface changes, update `README.md`, relevant example(s), and `CHANGELOG.md` in the same change.
- Before opening a PR, run `just quality`.
- If you touched CLI behavior, public SDK request/response types, migration docs, or CI-aligned release/test surfaces, also run `just quality-ci`.
- After opening or updating a PR, run `gh pr checks <pr>` and inspect failing Actions logs before assuming local validation is enough.

## Security And Configuration Tips

- Use environment variables for secrets; never hardcode keys.
- Start from `.env.example` for local setup.
- Management-governed endpoints use `OPENROUTER_MANAGEMENT_KEY`.
- API-key endpoints use `OPENROUTER_API_KEY`.

## Skills

- `openrouter-rs-release`: prepare and publish a new version with synchronized updates to `Cargo.toml`, `CHANGELOG.md`, README release history, and release validation checks. File: `.agents/skills/openrouter-rs-release/SKILL.md`

### Skill Trigger Rule

- Use `openrouter-rs-release` when the user asks to publish, release, or cut a new version.
