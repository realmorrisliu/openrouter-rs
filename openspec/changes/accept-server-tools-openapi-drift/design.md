## Context

The current tracked baseline has `87` method/path entries, while the latest upstream OpenAPI snapshot has `88`. The current drift report shows one added operation and four changed operations:

- `GET /workspaces/{id}/members`
- `POST /chat/completions`
- `POST /presets/{slug}/chat/completions`
- `POST /presets/{slug}/responses`
- `POST /responses`

The upstream schema now includes server-tool components such as `OpenRouterWebSearchServerTool`, `DatetimeServerTool`, `ChatWebSearchShorthand`, `ChatServerToolChoice`, `OutputWebSearchServerToolItem`, and `OutputDatetimeItem`. This makes issue #228 an OpenAPI drift acceptance task, not a docs-only feature request.

Current SDK shape is partially ready but uneven:

- Chat completions currently expose `types::Tool` as a function-tool struct with a required `function` definition.
- Responses requests already accept `tools: Vec<serde_json::Value>`, which is flexible but not ergonomic.
- Messages requests expose `AnthropicTool`, which covers Anthropic-style hosted tools but does not cleanly represent every OpenRouter `openrouter:*` server tool shape.
- The drift classifier already treats Responses and Messages tool payload internals as flexible, but not the chat-completions tool payload.

## Goals / Non-Goals

**Goals:**

- Implement the OpenAPI-defined server-tool surface for chat completions, Responses, Messages, and relevant preset creation endpoints.
- Keep existing function-tool construction and serialization source-compatible for callers.
- Make server tools first-class enough for common use, especially `openrouter:web_search` and `openrouter:datetime`, while preserving raw escape hatches for beta churn.
- Deserialize and expose server-tool output and usage details that appear in OpenAPI response schemas.
- Refresh the OpenAPI baseline after implementation and finish with zero drift.

**Non-Goals:**

- Do not treat server tools as plugins or migrate plugin APIs.
- Do not replace existing function-tool helpers with untyped JSON-only APIs.
- Do not refresh the OpenAPI baseline before the accepted drift is implemented or explicitly deferred.
- Do not implement local execution for server tools; OpenRouter executes these tools server-side.

## Decisions

1. Treat this as OpenAPI drift acceptance.

   The upstream OpenAPI already contains server-tool request and response schemas. The implementation will follow the drift workflow: inspect the current report, implement accepted SDK behavior, refresh the baseline, and verify `just openapi-drift-check` reaches zero drift. A helper-only change without baseline refresh would leave the repository with contradictory sources of truth.

2. Preserve `types::Tool` as the function-tool model.

   Existing callers can construct and inspect `Tool` as a function-tool struct. Replacing it with an enum would be a broad source break. Chat and Messages request builders should instead add additive server-tool helpers and merge function tools plus server tools into the wire `tools` array during serialization.

3. Introduce a flexible server-tool model.

   The server-tool model should carry a string tool type, optional typed parameters, and a flattened extra map. Typed constructors should cover stable/common OpenRouter tools such as web search and datetime. A raw constructor should allow beta tool variants that appear in OpenAPI before the SDK grows dedicated helpers.

4. Keep endpoint-specific wire semantics explicit.

   Chat, Responses, and Messages have different tool schemas. Shared server-tool helpers can provide common constructors, but each endpoint should own the conversion into its wire request shape. This prevents an Anthropic-hosted-tool shape from leaking into generic OpenRouter server tools, or vice versa.

5. Update drift classification only after support exists.

   Once chat and preset request builders can pass the OpenAPI-defined server-tool variants, the drift classifier may mark future item-level additions inside `tools` arrays as already supported. Container-shape changes, new required top-level fields, or new operations must remain actionable.

## Risks / Trade-offs

- Server-tool schemas are beta and may churn -> Use non-exhaustive public types, flexible string enums, flattened extras, and raw constructors.
- Additive helpers could diverge across chat, Responses, and Messages -> Keep endpoint-specific tests that assert exact JSON payloads for each endpoint.
- A broad baseline refresh could hide unrelated drift -> Inspect and implement or explicitly defer every actionable operation before running `just openapi-refresh-baseline`.
- Custom serialization for merged tool arrays can be brittle -> Add unit tests for function-only, server-only, and mixed-tool requests.
- Preset creation endpoints reuse request bodies -> Include preset-specific request-shape tests or update drift classification only when preset paths are demonstrably covered.

## Migration Plan

1. Implement request and response support behind additive helpers.
2. Add unit tests for serialization/deserialization and drift classifier behavior.
3. Add or update examples and public documentation for `openrouter:web_search`.
4. Run `just openapi-refresh-baseline`.
5. Run `just openapi-drift-check` and require `added=0, removed=0, changed=0`.
6. Run `just quality-ci`.

## Open Questions

- None for the OpenSpec boundary. During implementation, the live drift report should decide whether any non-server-tool drift is accepted in this slice or explicitly deferred before baseline refresh.
