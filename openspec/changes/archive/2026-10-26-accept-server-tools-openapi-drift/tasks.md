## 1. Drift Review

- [x] 1.1 Fetch the latest upstream OpenAPI snapshot and generate the drift report.
- [x] 1.2 Record every actionable item in the current drift batch, including server-tool schema changes and `GET /workspaces/{id}/members`.
- [x] 1.3 Decide whether each actionable drift item is accepted in this slice or explicitly deferred before baseline refresh.

## 2. Tool Request Surface

- [x] 2.1 Add a flexible server-tool model with typed helpers for `openrouter:web_search` and `openrouter:datetime` plus a raw escape hatch.
- [x] 2.2 Add chat-completions helpers that serialize function tools and server tools into one wire `tools` array.
- [x] 2.3 Add chat-completions `tool_choice` support for OpenAPI `ChatServerToolChoice`.
- [x] 2.4 Add Responses API typed helpers or conversions for accepted server-tool request variants while preserving raw `Value` support.
- [x] 2.5 Add Messages API support for accepted OpenRouter server-tool shapes without regressing existing Anthropic tool support.
- [x] 2.6 Ensure preset chat/responses creation paths can carry the same accepted server-tool request payloads.

## 3. Response And Management Coverage

- [x] 3.1 Add or update Responses output deserialization for server-tool output items such as web search and datetime.
- [x] 3.2 Verify chat and Messages response usage/server-tool blocks deserialize accepted server-tool usage fields.
- [x] 3.3 Add `GET /workspaces/{id}/members` SDK support or document explicit deferral before baseline refresh.

## 4. Tests And Drift Tooling

- [x] 4.1 Add unit tests for chat function-only, server-only, and mixed-tool serialization.
- [x] 4.2 Add unit tests for Responses and Messages server-tool serialization/deserialization.
- [x] 4.3 Add unit tests for server-tool `tool_choice` serialization.
- [x] 4.4 Add OpenAPI drift classifier tests so supported tool item additions are already supported and structural changes remain actionable.
- [x] 4.5 Add or update live/integration coverage where API keys and stable upstream behavior make it practical.

## 5. Documentation And Examples

- [x] 5.1 Update README or examples with a minimal `openrouter:web_search` server-tool request.
- [x] 5.2 Update `CHANGELOG.md` with the accepted OpenAPI drift and new SDK surface.
- [x] 5.3 Update `docs/operations/official-endpoint-test-matrix.md` for changed endpoint coverage.
- [x] 5.4 Reference issue #228 in the PR description and closeout notes.

## 6. Baseline And Verification

- [x] 6.1 Run `just openapi-refresh-baseline` after accepted drift is implemented.
- [x] 6.2 Run `just openapi-drift-check` and require `added=0, removed=0, changed=0`.
- [x] 6.3 Run `just quality-ci`.
- [x] 6.4 Inspect `git diff --check` and keep the PR scoped to the accepted drift plus documentation.
