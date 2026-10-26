## Context

`0.21.0` replaced the reqwest-only transport with a backend-neutral one: endpoint code is written against `transport::client::{HttpClient, RequestBuilder, Response}`, and either `reqwest` (default, native) or `wasip3` (wasm32-wasip2) supplies the bytes. `HttpRequestError` is deliberately a `{ message: String }` wrapper so it stays `Clone + PartialEq + Eq` across backends.

Above that seam, nothing changed. Measured on `0.22.0`:

| Symptom | Count |
| --- | --- |
| `api::*` endpoint free functions | 162 (161 call `transport::new_client()`) |
| Flat `OpenRouterClient` methods behind `#[doc(hidden)]` | 109 |
| `handle_error(response).await?; unreachable!()` pairs | 116 |
| Endpoints repeating the success/error branch | 112 |
| `_with_client` functions repeating the same arguments | 155 |
| `client.rs` | 4818 lines |

Two design constraints shape the fix:

1. Everything outside `src/transport/` must be backend-neutral, and `just check-wasip3` / `just test-wasip3` are release gates.
2. `0.22.0` deliberately made chat decoding lenient (shape-driven `Choice`, optional `ErrorResponse.code`, missing `tool_call.type`), so the decoder is an active area: a shared decode path must not re-introduce strictness.

## Goals / Non-Goals

**Goals:**

- One documented path to each endpoint family, and no public API that can silently discard client configuration.
- One decode path, one configuration value, one declaration per request field.
- Failure classification that works on both backends and keeps the error type comparable.
- One streaming event model that is a superset of the two current ones.
- Forward-compatible enums so an upstream addition does not fail a whole response.

**Non-Goals:**

- No change to endpoint coverage, wire paths, or the OpenAPI baseline.
- No `extra: HashMap<String, Value>` escape hatch, no `schemars` feature gate, no `#![warn(missing_docs)]`.
- No retry execution, timeouts, or middleware inside the SDK.
- No `#[doc(cfg(...))]` attributes: they need nightly `doc_cfg`, and `[package.metadata.docs.rs]` with `all-features` covers the documentation gap on stable.

## Decisions

1. **Failure classification is an enum the backends fill in, not a wrapped `reqwest::Error`.**

   The obvious fix for "callers cannot tell a timeout from a certificate rejection" is to keep the backend error. That is now impossible: `reqwest` is an optional feature, and on `wasm32-wasip2` there is no `reqwest::Error` at all. So `HttpRequestError` gains `kind: HttpRequestErrorKind` and `url: Option<String>`, populated where the backend information actually exists — `transport::reqwest_backend::transport_error` maps `is_timeout/is_connect/is_body/is_decode/is_request`, and `transport::wasip3_backend::failed` maps its host failure kinds. The public type keeps `Clone + PartialEq + Eq`, which callers can use directly; only the *classification* is comparable, not a backend object.

   Rejected: a `source: Box<dyn Error>` field. It destroys `Clone`/`Eq` and cannot be inspected without downcasting to a type that may not be compiled in.

2. **`send_json`/`send_unit` sit at the transport layer, above `Response`.**

   Because both backends already normalize into one `Response`, the helper can own send → decode → normalize once:

   ```rust
   send_json::<T>(builder, context)     // ApiResponse<T> on success, Api error otherwise
   send_unit(builder)                   // no body on success
   parse_or_error(response, context)    // for the two callers that must own the send
   ```

   Rejected: a declarative macro. It would flatten 112 call sites into unreadable fragments and lose the type inference that makes `let parsed: ApiResponse<T> = …` work.

3. **`handle_error` is deleted, not retyped.**

   Its only remaining consumers are the binary, multipart, and SSE endpoints, which do not have a JSON success body. Those call `error_from_response` directly, which returns `OpenRouterError`. Deleting the `Result<(), _>` signature is what removes the `unreachable!()` obligation permanently rather than moving it.

4. **The endpoint free functions are deleted, not deprecated.**

   Their signature has no place to accept a client, so they cannot honor configuration; that is a defect, not a style. Only one in-repo call site exists (a deprecated example) and the WASI fixture uses domain clients only, so the migration is contained. A deprecation window would keep a functionally broken path reachable for another release.

5. **The flat methods are deleted in the same release, with no `#[deprecated]` pass.**

   They are already `#[doc(hidden)]`, so they were never a documented promise. Adding 109 deprecation attributes would produce more compiler noise than migration signal while the surface shrinks by 271 items in one step.

6. **`ChatTool` is a typed field, and the request is derived.**

   The four declarations exist only because `tools` and `server_tools` merge into one wire array and must split again on read. Storing `Option<Vec<ChatTool>>` removes that reason: serialization is a plain derive, and deserialization splits by variant through `#[serde(untagged)]`. `ChatTool` is `#[non_exhaustive]` with `Function`/`Server` variants, and the getters become iterators.

   Rejected: a borrowed mirror struct for serialization. `skip_serializing_if` needs `&Option<T>`, so a borrowing mirror cannot avoid cloning every field.

7. **Tool-call accumulation moves into `adapt_chat_stream`.**

   The two streaming models differ only in that one assembles tool calls. Moving `ToolCallAccumulator` into the adapter makes the unified model a superset, after which `ToolAwareStream`, `StreamEvent`, and `stream_tool_aware` have no consumers. `ChatClient::stream` stays public: it is the raw chunk stream that backs the adapter, not a competing abstraction.

8. **`UnifiedStreamEvent::Done::usage` becomes `Option<ResponseUsage>`.**

   It is currently produced by `serde_json::to_value(usage)` — typed data converted to JSON so the caller can convert it back. The Responses adapter parses its usage object into `ResponseUsage`; the Messages adapter reports `None` where Anthropic's usage fields have no faithful mapping, because a lossy conversion is worse than an absent one.

9. **Only receive-side taxonomy enums gain `Other(String)`.**

   `Role`, `ObjectType`, and `AnthropicRole` appear in responses, so an unknown value would fail the containing message. Caller-supplied enums (`Modality`, `ModelCategory`, `SupportedParameters`, quantizations, and similar) stay strict: an unrecognized value there is a caller error, not upstream drift. The audit is recorded in the design so a future reader does not have to re-derive it.

10. **`RequestContext` is introduced once, while signatures must change anyway.**

    Only 31 of 155 `_with_client` functions carry the six-argument attribution clump; the rest take three configuration arguments. Doing it now, in the same change that makes those functions crate-private, means each signature changes exactly once.

## Enum Forward-Compatibility Audit

**Now tolerant of unknown values** (receive-side; an upstream addition would otherwise fail the containing message): `types::Role`, `types::completion::ObjectType`, `api::messages::AnthropicRole`. `types::completion::FinishReason` already carried `Other(String)`.

**Left strict on purpose** (caller-supplied, so an unrecognized value is a caller error): `Modality`, `CacheControlType`, `PromptCacheMode`, `EmbeddingEncodingFormat`, `CodeChallengeMethod`, `ModelCategory`, `SupportedParameters`, `ProviderSortBy`, `DataCollectionPolicy`, `Quantization` (already models `Unknown`).

**Content unions** (`Content`, `Choice`, `AnthropicMessageContent`, `EmbeddingInput`) are shape-dispatched: an unrecognized shape is a genuinely new content semantic that a string escape hatch could not represent.

## Risks / Trade-offs

- **271 public items disappear in one release.** Mitigate with a `MIGRATION.md` table mapping every removed entry point to its domain-client replacement, a `CHANGELOG.md` breaking section, and a `cargo semver-checks` report whose every entry is accounted for. This is the third consecutive breaking minor (0.21 MSRV/backend, 0.22 decoder shapes); the release notes must say so plainly.
- **The derive migration can silently drop a field.** Mitigate by writing the full-payload key-set test and the round-trip test *before* touching the serializer, and by treating any snapshot change as a defect.
- **`send_json` does not fit every endpoint.** Binary downloads, multipart uploads, and SSE need their own decode step; they stay on `error_from_response`. Recorded as task deviation instead of being forced into the helper.
- **Merging the streaming models could lose behavior the tool-aware wrapper has.** Mitigate by porting every `ToolAwareStream` test to `stream_unified` and requiring them green before deleting the old types. Those tests currently assert on `StreamEvent::ToolDelta` ordering, which the unified model also emits.
- **A shared decode path could re-tighten what 0.22 loosened.** Mitigate by keeping `parse_json_response`'s behavior identical — it already prefers an embedded API error over a deserialization error — and by keeping the 0.22 lenient-decode tests as the gate.
- **`RequestContext` touches 155 signatures.** Mitigate by transforming per module and running `cargo check --all-targets` after each; a mechanical rename is reviewable when each module is a few hundred lines.
- **`--no-default-features` builds have no backend except on WASI.** Mitigate by running `just check-wasip3` and `just test-wasip3` for every stage that touches `transport/`, `error.rs`, or `api/**`.

## Migration Plan

1. Establish the acceptance gate: full-payload serializer snapshot and round-trip tests, plus failure-classification, `Retry-After`, and mid-stream-error tests written against the current behavior where it is correct.
2. Failure classification and `retry_after` in `error.rs`, both backends, and `api/errors.rs`.
3. `send_json`/`send_unit`/`parse_or_error` in `transport/response.rs`; migrate all 112 JSON endpoints; delete `handle_error` and every `unreachable!()`.
4. `RequestContext` and the `_with_client` migration; split `src/client.rs`.
5. `ChatTool` single field with derive serialization.
6. Streaming unification and mid-stream error mapping; delete the old stream types.
7. `Other(String)` variants and `#[non_exhaustive]` on `OpenRouterError`.
8. Migrate in-repo call sites, then delete the 162 free functions and 109 flat methods.
9. Documentation: `MIGRATION.md`, `CHANGELOG.md`, `README.md`, endpoint matrix, `docs.rs` metadata; remove `dotenvy_macro` and `src/generated/`.
10. Verification: `just quality-ci`, `just check-wasip3`, `just test-wasip3`, `cargo +nightly semver-checks check-release`, and reconcile its report with `MIGRATION.md`.

## Open Questions

- None blocking.
