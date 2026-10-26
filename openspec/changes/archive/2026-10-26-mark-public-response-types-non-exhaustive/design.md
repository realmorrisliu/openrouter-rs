## Context

`openrouter-rs` has grown from chat-only usage into a domain SDK that tracks upstream OpenRouter OpenAPI drift. Several newer high-churn types are already marked `#[non_exhaustive]`, including audio, video, workspace, API-key, and generation models, but older public models remain fully exhaustive.

PR #205 showed the risk: adding typed optional fields to `ResponseUsage` is semantically additive for serde, but source-breaking for users who instantiate the public struct with literals. Since the project is already holding this for `0.10.0`, the next step should be a deliberate public API audit instead of piecemeal attributes.

## Goals / Non-Goals

**Goals:**

- Identify public structs and enums under `src/api/` and `src/types/` that represent upstream API shapes likely to change.
- Mark those high-churn types `#[non_exhaustive]` in a 0.10.0 change.
- Keep caller construction ergonomic for request/configuration types through builders, constructors, or helpers.
- Update tests, examples, and migration docs so the source break is explicit and reviewable.

**Non-Goals:**

- Do not change endpoint behavior, JSON field names, request semantics, or runtime compatibility.
- Do not rewrite all models into private fields or generated code as part of this change.
- Do not mark private wire structs or intentionally stable helper types unless they are part of the public API risk.
- Do not release this as a 0.9.x patch.

## Decisions

1. Audit by public API category, not by file age.

   Classify each `pub struct` and `pub enum` in `src/api/` and `src/types/` as request/configuration, response/data, stream event, upstream taxonomy, builder/helper, or internal wire type. This avoids missing older types such as chat completion, embeddings, discovery, models, messages, tools, provider preferences, and stream events.

2. Prefer non-exhaustive for upstream schema mirrors.

   Public models that mirror OpenRouter/OpenAI/Anthropic request or response JSON should default to `#[non_exhaustive]` unless the audit records a specific reason to keep them exhaustive. These shapes are where additive upstream fields and variants are most likely.

3. Preserve construction paths before adding attributes.

   For caller-built request and configuration types, add or verify builders before marking them non-exhaustive. Response-only types can rely on serde deserialization and do not need public constructors unless tests or examples currently create them manually.

4. Treat enum exhaustiveness as an API promise.

   Upstream taxonomy enums should be marked non-exhaustive when new wire values are plausible. Stable SDK-owned enums can remain exhaustive if exhaustive matching is intended and the type does not mirror upstream drift.

5. Make migration docs part of the implementation, not a follow-up.

   Because this is intentionally source-breaking, `MIGRATION.md`, `CHANGELOG.md`, and examples must be updated in the same change. The docs should call out direct struct literals and exhaustive matches as the two main migration cases.

## Risks / Trade-offs

- Source break is broader than the immediate `ResponseUsage` issue -> Mitigate by targeting 0.10.0 and documenting migration paths clearly.
- Over-applying `#[non_exhaustive]` can make tests and examples noisier -> Mitigate by using builders/helpers where construction is legitimate and keeping intentionally stable helper types exhaustive.
- Under-applying it leaves future OpenAPI drift source-breaking -> Mitigate with an inventory checklist and review every public type in `src/api/` and `src/types/`.
- Enum non-exhaustiveness can frustrate callers who prefer exhaustive matches -> Mitigate by limiting it to upstream taxonomies where new wire values are plausible and documenting wildcard matching.

## Migration Plan

1. Implement on a 0.10.0 branch after the public type inventory is reviewed.
2. Update local tests and examples to use builders/helpers or add wildcard match arms where needed.
3. Run `just quality`, `just check-migration-docs`, and `cargo test --test migration_smoke --all-features`.
4. Include the migration note in the 0.10.0 release notes.
5. If the change proves too broad during implementation, split into response/data types first and request/configuration types second, but keep both under the 0.10.0 release boundary.

## Implementation Audit Outcome

- `Tool`, `FunctionDefinition`, and related tool-choice structs follow the upstream-schema rule because their serialized shape mirrors chat-completions tool payloads and can gain fields over time. Construction remains ergonomic through `Tool::function(...)`, `ToolBuilder`, `ToolChoice` helpers, and `create_tool(...)`.
- SDK-normalized streaming event enums are marked `#[non_exhaustive]` because adding normalized event categories should not source-break callers that match stream events.
- The public type inventory is recorded in `inventory.md` for review, while stable SDK-owned helpers with private fields remain exhaustive there with explicit rationale.

## Open Questions

- None after implementation audit.
