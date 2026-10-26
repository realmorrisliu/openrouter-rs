## Why

Issue #228 asked for OpenRouter server-tool support, especially web search. Current upstream `openapi.json` already includes server-tool schemas in chat, Responses, and Messages request surfaces, so this must be treated as OpenAPI drift acceptance rather than a docs-only helper.

## What Changes

- Add typed SDK support for OpenAPI-defined server tools in chat completions, Responses API, and Anthropic-compatible Messages where the upstream schema exposes them.
- Preserve existing function-tool ergonomics while allowing server tools to be sent in the same wire `tools` array.
- Add support for server-tool-specific tool choice and server-tool output/usage payloads where the OpenAPI schema exposes them.
- Accept the full current actionable drift in the same implementation slice, including the new `GET /workspaces/{id}/members` operation or an explicit documented deferral before baseline refresh.
- Refresh the tracked OpenAPI baseline only after SDK behavior, tests, docs, and drift classification match the accepted upstream surface.

## Capabilities

### New Capabilities

- `openapi-drift-acceptance`: Rules for accepting this upstream OpenAPI drift batch, including server-tool request/response support and baseline refresh discipline.

### Modified Capabilities

- None.

## Impact

- Affected code: `src/types/tool.rs`, chat/responses/messages request builders, Responses output types, workspace management client methods, preset request surfaces that reuse chat/responses bodies, and OpenAPI drift classification.
- Affected docs: `README.md`, `CHANGELOG.md`, `docs/operations/official-endpoint-test-matrix.md`, and examples for server-tool usage.
- API impact: additive public SDK surface for server tools and workspace member listing. Existing function-tool callers should remain source-compatible.
- Validation impact: run `just openapi-refresh-baseline`, `just openapi-drift-check`, `just quality-ci`, and targeted unit/integration tests before opening a PR.
