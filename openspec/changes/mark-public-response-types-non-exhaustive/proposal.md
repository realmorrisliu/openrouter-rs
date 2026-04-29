## Why

The 2026-04-29 OpenAPI drift added optional chat usage fields, and exposing them on `ResponseUsage` was source-breaking because external callers can construct that public struct exhaustively. The SDK needs a planned 0.10.0 pass that makes high-churn public API model types future-proof before more upstream response and request metadata lands.

## What Changes

- **BREAKING**: Mark selected high-churn public SDK request, response, metadata, usage, pricing, discovery, streaming, and upstream taxonomy types with `#[non_exhaustive]`.
- Preserve ergonomic construction for caller-built request/configuration types by requiring builders or explicit constructors before marking those types non-exhaustive.
- Audit public structs and enums under `src/api/` and `src/types/` so the decision is systematic rather than limited to the latest drift.
- Update migration notes, release notes, docs, and examples to steer callers away from direct struct literals and exhaustive enum matches where the API can evolve.
- Keep deliberately stable internal/private wire types and low-churn helper types exhaustive when exhaustive construction or matching is part of the intended API.

## Capabilities

### New Capabilities

- `sdk-public-type-future-proofing`: Rules for deciding when public SDK model types are non-exhaustive, how callers construct them, and how breaking migration guidance is documented.

### Modified Capabilities

- None.

## Impact

- Affected code: public structs and enums in `src/api/` and `src/types/`, plus examples and tests that use struct literals or exhaustive matches.
- Affected docs: `CHANGELOG.md`, `MIGRATION.md`, `README.md`, relevant examples, and possibly docs.rs comments.
- API impact: source-breaking in Rust for external callers that instantiate affected structs with literals or match affected enums without a wildcard arm. Runtime JSON compatibility should remain backward-compatible through serde defaults and optional fields.
- Release impact: target this work for `0.10.0`, not a `0.9.x` patch release.
