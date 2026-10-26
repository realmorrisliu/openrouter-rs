## 1. Acceptance Gate

- [x] 1.1 Add a full-payload `ChatCompletionRequest` key-set test asserted against `serde_json::Value`.
- [x] 1.2 Add a round-trip test that sets every builder field and asserts each appears on the wire.
- [x] 1.3 Verify `cargo test --test unit` is green before any production change.

## 2. Failure Classification

- [x] 2.1 Add `HttpRequestErrorKind` and `kind`/`url`/`is_timeout`/`is_connect` to `HttpRequestError`, keeping `Clone + PartialEq + Eq`.
- [x] 2.2 Populate `kind` in `transport::reqwest_backend::transport_error` and in `transport::wasip3_backend::failed`.
- [x] 2.3 Add `retry_after: Option<Duration>` to `ApiErrorContext` and parse delta-seconds `Retry-After` in `transport::response::error_from_response`.
- [x] 2.4 Update the `error.rs` rate-limit example to use `retry_after` instead of a fixed sleep.
- [x] 2.5 Add unit tests for timeout/connect classification, `Retry-After` delta-seconds, HTTP-date falling back to `None`, and non-retryable errors.
- [x] 2.6 Verify with `cargo test --test unit` and `just check-wasip3`.

## 3. Shared Decode Path

- [x] 3.1 Add `send_json<T>`, `send_unit`, and `parse_or_error` to `transport::response`, built on the backend-neutral `Response`.
- [x] 3.2 Delete `handle_error`; retarget the binary, multipart, and SSE endpoints at `error_from_response`.
- [x] 3.3 Migrate all 112 JSON endpoints to the shared helper, one module per commit; every site must compile before it is kept.
- [x] 3.4 Confirm `grep -c 'unreachable!()' src/api` and `grep -c 'is_success' src/api` are zero, except the two SSE paths.

## 4. Request Context And Client Split

- [x] 4.1 Add `transport::RequestContext` and `OpenRouterClient::request_context`.
- [x] 4.2 Migrate all 155 `_with_client` functions and their call sites to the context value.
- [x] 4.3 Split `src/client.rs` into `client/{mod,domains,management}.rs` without semantic change.
- [x] 4.4 Verify with `cargo check --all-targets --all-features`.

## 5. Request Shape

- [x] 5.1 Add `ChatTool` and store one `tools: Option<Vec<ChatTool>>` field on `ChatCompletionRequest`.
- [x] 5.2 Serialize with derive; delete the hand-written `Serialize` map builder and the duplicate wire struct.
- [x] 5.3 Replace `tools()`/`server_tools()` with iterators over the merged field, adding `function_tools()`.
- [x] 5.4 Keep tasks 1.1 and 1.2 green; re-run the function-only, server-only, mixed, and merged-array tool tests.

## 6. Streaming

- [x] 6.1 Port `ToolCallAccumulator` into `adapt_chat_stream` and add `tool_calls` to `UnifiedStreamEvent::Done`.
- [x] 6.2 Change `Done::usage` to `Option<ResponseUsage>`; supply empty `tool_calls` from the Responses and Messages adapters.
- [x] 6.3 Port every `ToolAwareStream`/`StreamEvent` test to `stream_unified` and require them green.
- [x] 6.4 Delete `ToolAwareStream`, `StreamEvent`, and `stream_tool_aware` from `ChatClient` and the flat client.
- [x] 6.5 Map a mid-stream API error payload to `OpenRouterError::Api` and add a regression test.

## 7. Forward Compatibility

- [x] 7.1 Add `Other(String)` to `Role`, `ObjectType`, and `AnthropicRole`, with round-trip tests.
- [x] 7.2 Mark `OpenRouterError` `#[non_exhaustive]` and add a test that a wildcard arm is required.

## 8. Canonical Surface

- [x] 8.1 Migrate every in-repo call site of a free function or flat method to the domain client.
- [x] 8.2 Delete the 162 `api::*` endpoint free functions.
- [x] 8.3 Delete the 109 flat methods and the `#[doc(hidden)]` impl.
- [x] 8.4 Verify `cargo doc --all-features` exposes no endpoint function and `crates/openrouter-cli` still builds.

## 9. Documentation And Packaging

- [x] 9.1 Add the `MIGRATION.md` table mapping every removed entry point to its replacement.
- [x] 9.2 Update `README.md`, `CHANGELOG.md`, the endpoint matrix, and `docs/design/generated-core-architecture.md`.
- [x] 9.3 Add `[package.metadata.docs.rs]` with `all-features`.
- [x] 9.4 Remove `dotenvy_macro` and the empty `src/generated/` module.

## 10. Verification

- [x] 10.1 Run `just quality-ci`.
- [x] 10.2 Run `just check-wasip3` and `just test-wasip3`.
- [x] 10.3 Run `cargo +nightly semver-checks check-release` and confirm every reported break appears in `MIGRATION.md`.
  <!-- Reported classes vs 0.22.0: `function_missing` (162 endpoint functions),
       `inherent_method_missing` (109 flat methods, `ChatClient::stream_tool_aware`),
       `struct_missing`/`enum_missing` (`ToolAwareStream`, `StreamEvent`),
       `enum_marked_non_exhaustive` (`OpenRouterError`),
       `enum_discriminants_undefined_non_unit_variant` (`Role`, `ObjectType`, `AnthropicRole`),
       `enum_struct_variant_field_added` (`UnifiedStreamEvent::Done::tool_calls`),
       `constructible_struct_adds_field` (`ApiErrorContext::retry_after`). All documented. -->
- [x] 10.4 Confirm `git diff --check` is clean and no `unreachable!()` remains in `src/api`.
