## Why

The same endpoint is reachable three ways, and the SDK ships two parallel streaming models, two error vocabularies, and four hand-maintained declarations of one request's fields. The cost is not stylistic:

- **162 `api::<module>::<function>(base_url, api_key, …)` free functions**, 161 of which call `transport::new_client()`. A caller who reaches one gets a fresh connection pool per request plus no timeout, proxy, TLS, or attribution configuration, with no signal that anything was lost. On native targets this is a correctness bug in the function's signature: it has no parameter for a client.
- **109 flat `OpenRouterClient` methods** behind `#[doc(hidden)]`. They are undocumented but still public, so they are still a semver commitment, and they duplicate the domain clients.
- **116 `handle_error(response).await?; unreachable!()` pairs**, because `transport::response::handle_error` is typed `Result<(), OpenRouterError>` but always returns `Err`. 112 JSON endpoints repeat the same success/error branch instead of sharing one decode path.
- **`ChatCompletionRequest` declares its 43 fields four times** (struct, wire struct, `Deserialize`, `Serialize`). Adding a field requires four synchronized edits and a missed one silently drops it from the wire.
- **Two streaming models.** `ToolAwareStream`/`StreamEvent` and `UnifiedStreamEvent` overlap, and the "unified" one cannot replace the other because `Done` carries no assembled tool calls and types usage as `serde_json::Value`.
- **Failures are not diagnosable.** `HttpRequestError` stores only `to_string()`, so a caller cannot distinguish a timeout, a refused connection, and a certificate rejection. `Retry-After` is dropped while `error.rs` documents a hardcoded 60-second sleep. An API error delivered mid-stream surfaces as a deserialization error.
- **Unknown upstream values break whole responses.** `Role`, `ObjectType`, and `AnthropicRole` have no unknown-value arm, and `OpenRouterError` is not `#[non_exhaustive]`.

The 0.21.0 transport rewrite made the fix clearer: `HttpRequestError` is deliberately backend-neutral and must stay so, so failure classification must be an enum each backend fills in rather than a wrapped `reqwest::Error`.

## What Changes

This lands as one change and one release, `0.23.0`. Every dimension is completed in the same slice; nothing in the list below is left partial or deferred.

**One canonical public surface (breaking)**
- Delete the 162 `api::*` endpoint free functions.
- Delete the 109 flat `OpenRouterClient` methods and the `#[doc(hidden)]` impl.
- Migrate every in-repo call site to domain clients (examples, unit tests, integration tests, migration smoke, WASI fixture).

**One decode path and one request configuration value (internal)**
- Replace the 112 endpoint branches with `send_json` / `send_unit` and remove every `unreachable!()`.
- Replace the six-argument transport plumbing with one `RequestContext`.
- Split `src/client.rs` into `client/{mod,domains,management}.rs`.

**Diagnosable failures (breaking where noted)**
- Add `HttpRequestErrorKind` (`Timeout`, `Connect`, `Request`, `Body`, `Decode`, `Other`) plus `url()`, `is_timeout()`, `is_connect()` on `HttpRequestError`, filled in by both backends. The type keeps `Clone`/`PartialEq`/`Eq` so it stays backend-neutral.
- Parse `Retry-After` (delta-seconds form) into `ApiErrorContext::retry_after`.
- Map an API error payload arriving mid-stream to `OpenRouterError::Api`.
- Mark `OpenRouterError` `#[non_exhaustive]`.

**One streaming model (breaking)**
- Move tool-call accumulation into the chat adapter and add `tool_calls: Vec<ToolCall>` to `UnifiedStreamEvent::Done`.
- Change `UnifiedStreamEvent::Done::usage` to `Option<ResponseUsage>`.
- Delete `ToolAwareStream`, `StreamEvent`, `ChatClient::stream_tool_aware`, and the flat equivalent.

**One declaration per request field (breaking)**
- Store chat tools as one `tools: Option<Vec<ChatTool>>` field and serialize the request with derive; delete the hand-written `Serialize` map builder and the duplicate wire struct.
- `tools()` returns an iterator over `ChatTool`; `function_tools()` and `server_tools()` filter it.

**Forward-compatible taxonomies (breaking)**
- Add `Other(String)` to `Role`, `ObjectType`, and `AnthropicRole`.

**Packaging**
- Add `[package.metadata.docs.rs]` so the published docs build with all features.
- Remove the unused `dotenvy_macro` dependency and the empty `src/generated/` placeholder.

## Capabilities

### New Capabilities

- `sdk-public-surface`: what the SDK exposes as its canonical API, how failures are classified for callers, and how streaming is surfaced.
- `sdk-transport-internals`: the internal invariants that keep transport plumbing from duplicating.

### Modified Capabilities

- `sdk-public-type-future-proofing`: extend the non-exhaustive rule to the public error type and add a requirement that upstream taxonomy enums tolerate unknown values.

## Impact

- Affected code: `src/client.rs` (split and reduction), `src/api/**` (visibility, signatures, shared decode path), `src/error.rs`, `src/api/errors.rs`, `src/transport/**` (both backends, shared response path), `src/types/stream.rs`, `src/types/mod.rs`, `src/types/completion.rs`, `src/lib.rs`, `Cargo.toml`.
- Affected docs: `README.md`, `CHANGELOG.md`, `MIGRATION.md`, `docs/operations/official-endpoint-test-matrix.md`, `docs/design/generated-core-architecture.md`.
- Affected tests: every unit and integration transport test moves from free functions to `test_client(...).<domain>()`, plus new tests for failure classification, `Retry-After`, mid-stream API errors, and unified tool-call assembly.
- API impact: source-breaking for callers that used a free function or a flat method, for callers that match `UnifiedStreamEvent::Done` exhaustively, for callers that construct `ChatCompletionRequest` fields directly, and for callers that match `OpenRouterError` exhaustively. Callers using domain clients and builders are otherwise unaffected.
- Backend impact: none on the request/response wire; both `reqwest` and `wasip3` must keep compiling and passing `just check-wasip3` / `just test-wasip3`.
- Validation: `just quality-ci`, `just check-wasip3`, `just test-wasip3`, and a one-off `cargo +nightly semver-checks check-release` whose every reported break is documented in `MIGRATION.md`.

## Out of Scope

- No `extra: HashMap<String, Value>` escape hatch on request types; that widens the expression surface and deserves its own proposal.
- No feature-gating of `schemars` or the typed-tools API.
- No `#![warn(missing_docs)]` (2 000+ existing warnings would drown the diff) and no blanket `#[must_use]`.
- No retry execution, timeouts, or middleware in the SDK; this change makes the information available, policy stays with the caller.
- No change to endpoint coverage, wire paths, or the OpenAPI baseline.
