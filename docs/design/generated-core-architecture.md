# Generated Core + Idiomatic Wrapper Architecture

This document defines the intended architecture for moving `openrouter-rs` toward a generated low-level core without giving up the ergonomic, domain-oriented Rust surface that exists today.

It is the design baseline for [#152](https://github.com/realmorrisliu/openrouter-rs/issues/152) and is meant to inform the narrower scaffold work in `#154`.

## Why This Exists

The repository already has a useful OpenAPI drift loop:

- tracked baseline snapshots under `specs/openrouter/`
- endpoint coverage review via `docs/operations/official-endpoint-test-matrix.md`
- nightly drift reporting in `docs/operations/openapi-drift-reporting.md`

That is enough to detect upstream change, but not enough to define how spec data should eventually drive implementation.

If the project wants to stay aligned with a spec-driven SDK direction, it needs a deliberate boundary between:

- generated, low-level, unstable implementation details
- stable, handwritten, idiomatic Rust APIs that users actually depend on

## Goals

- Keep the canonical public API domain-oriented: `chat()`, `responses()`, `messages()`, `models()`, `management()`, and `legacy()`
- Introduce a generated low-level layer that can follow the upstream OpenAPI more mechanically
- Separate drift detection inputs from generation inputs so review workflows stay useful
- Preserve streaming, typed tools, builder ergonomics, and unified abstractions as handwritten Rust-first surfaces
- Make future generated-module work reviewable in small vertical slices instead of one large rewrite

## Non-Goals

- Replacing the current public API in one pass
- Auto-generating the final user-facing Rust builders and helper methods for every endpoint
- Rewriting streaming abstractions around raw generated SSE payloads
- Moving CLI config/profile resolution into the SDK core
- Treating generated internals as part of the stable public contract

## Current Constraints

Today the repository has three important realities:

1. The stable public entrypoint is already clear.

   - `src/client.rs` exposes the canonical domain clients
   - `src/api/*.rs` contains the domain request/response types and endpoint logic
   - `src/types/*.rs` contains shared ergonomic abstractions such as pagination, streams, tools, and typed tools

2. The tracked OpenAPI baseline is review-oriented, not generation-oriented.

   - `specs/openrouter/openapi-baseline.json` is intentionally seeded from the currently accepted endpoint matrix
   - that baseline is useful for nightly drift and coverage review
   - it is not a good long-term source of truth for generation because it can intentionally lag full upstream coverage

3. The hardest SDK behavior is not plain request serialization.

   - streaming adapters in `src/types/stream.rs`
   - tool-aware aggregation
   - typed tools in `src/types/typed_tool.rs`
   - multimodal request ergonomics
   - domain-level naming and auth boundaries in `src/client.rs`

Those are the areas where a generated core is least likely to be the right public abstraction.

## Decision Summary

- Keep the public SDK surface handwritten.
- Add an internal generated boundary inside the main crate, not a new public crate, for the first pass.
- Keep drift baselines separate from generation snapshots.
- Use overlays as repo-local implementation inputs, not as user-facing configuration.
- Migrate endpoint families one domain at a time, starting with non-streaming domains.

## Proposed Layering

### 1. Spec Inputs And Review Assets

The repository should distinguish between drift-review assets and generation assets.

Proposed direction:

```text
specs/openrouter/
  openapi-baseline.json
  openapi-baseline.operations.json
  source/
    openapi.json
  overlays/
    auth.yaml
    naming.yaml
    rust.yaml
  generator/
    config.yaml
```

Meaning:

- `openapi-baseline.json` and `openapi-baseline.operations.json` stay drift-focused and continue to back nightly detection
- `source/openapi.json` becomes the accepted full upstream snapshot used for generation work
- `overlays/*.yaml` carries repo-reviewed adjustments needed for generation, naming, or schema quirks
- `generator/config.yaml` describes generator inputs, output paths, and reproducibility settings

Important rule:

The drift baseline is not the generation source of truth.

That separation matters because drift review is allowed to say:

- "upstream added six more operations"
- "we are not accepting those into the stable SDK yet"

while generation work may later choose to ingest a fuller accepted upstream snapshot in a separate step.

### 2. Internal Generated Boundary

The first generated boundary should be an internal module inside the existing crate:

```text
src/
  generated/
    mod.rs
    support.rs
    operations/
      mod.rs
      models.rs
      discovery.rs
      credits.rs
      generation.rs
    schemas/
      mod.rs
      shared.rs
```

Why an internal module instead of a new crate first:

- smaller review surface for `#154`
- no new publish/release surface to stabilize
- easier to hide generated churn behind `pub(crate)`
- avoids forcing the CLI or downstream users to learn a second package boundary immediately

This module should be treated as internal implementation detail:

- `pub(crate)` or otherwise not re-exported from `src/lib.rs`
- no compatibility promises across minor releases
- allowed to be regenerated, renamed, or reorganized as the generator evolves

`src/generated/support.rs` is intentionally handwritten. It is the place for small internal helpers that generated code can share without making the generator own everything.

### 3. Handwritten Public Wrapper Layer

The public SDK surface remains where it is today:

- `src/client.rs`
- `src/api/*.rs`
- `src/types/*.rs`

This layer owns:

- domain grouping and method names
- builder ergonomics
- auth-key routing and runtime validation
- unified streaming
- typed tools and schema helpers
- migration shims and deprecation bridges

Generated code is allowed to power this layer internally, but not to define the user-facing API shape by default.

## Handwritten Vs Generated Ownership

| Area | Handwritten | Generated | Notes |
| --- | --- | --- | --- |
| Client entrypoint | `src/client.rs` | None | Domain clients stay the canonical surface |
| Domain request builders | `src/api/*.rs` | Optional internal raw structs | Public builders remain Rust-first |
| Low-level request/response shapes | Minimal glue only | `src/generated/operations/*`, `src/generated/schemas/*` | Best fit for OpenAPI-driven churn |
| Streaming | `src/types/stream.rs`, domain stream methods | Raw chunk payload structs at most | SSE semantics should stay handwritten |
| Typed tools | `src/types/tool.rs`, `src/types/typed_tool.rs` | Underlying JSON-schema fragments only | Rust typing remains a feature, not a codegen side effect |
| CLI config/profile behavior | `crates/openrouter-cli` | None in phase 1 | Do not couple CLI UX to generated internals yet |
| Legacy completions | Handwritten | None initially | Lower priority and feature-gated |

## Source-Of-Truth Flow

The intended flow is:

1. Nightly drift compares upstream against the tracked drift baseline.
2. The endpoint matrix answers whether the repo currently implements and tests that surface.
3. When a change is accepted for deeper integration work, refresh the full generation snapshot separately.
4. Apply repo-reviewed overlays to that accepted source snapshot.
5. Generate internal low-level modules into `src/generated/`.
6. Keep `src/client.rs`, `src/api/*.rs`, and `src/types/*.rs` as the stable wrapper layer over those internals.

That lets the repo say two different things without contradiction:

- "we noticed upstream changed"
- "we have not yet changed the stable Rust surface"

## Public Stability Rules

Stable by default:

- `OpenRouterClient`
- domain client method names such as `chat()` and `responses()`
- public request builders and common response types already documented in README/examples
- shared ergonomic abstractions in `src/types/*`
- CLI behavior documented in `crates/openrouter-cli/README.md`

Unstable by default:

- `src/generated/*`
- codegen templates/config
- overlay file format
- raw generated operation names
- temporary internal conversion glue

Generated internals should be considered implementation detail until explicitly promoted.

## How Idiomatic Rust Surfaces Stay Intact

### Streaming

Streaming should remain handwritten.

OpenAPI can describe payload shapes, but it does not define:

- how chunk streams should be aggregated
- how chat/messages/responses streams should be unified
- how tool-aware streaming should surface partial state

So the generated layer may eventually provide raw chunk structs, but the SDK should keep:

- `stream()`
- `stream_tool_aware()`
- `stream_unified()`

as handwritten wrappers.

### Typed Tools

Typed tools are a Rust feature, not just an API feature.

The SDK should continue to let users define Rust structs and derive JSON Schema from them through `schemars`. The generated core only needs to understand the final tool payload shape, not the user-facing typed-tool authoring model.

### Builders And Domain Naming

The public builders in `src/api/*.rs` should remain intentionally shaped around Rust ergonomics:

- defaults that make sense for Rust callers
- helper constructors
- type-safe enums
- domain-level naming that matches the README and examples

The generated layer can be flatter and more mechanical. That is acceptable because users should not depend on it directly.

## Migration Sequencing

Recommended order:

### Phase 0: Done

- Nightly drift detection and baseline reporting
- Endpoint matrix as explicit review surface

### Phase 1: `#152`

- Land this design doc
- Agree on the directory and module boundary before generating anything

### Phase 2: `#154`

- Add the `specs/openrouter/source/`, `specs/openrouter/overlays/`, and `src/generated/` seam
- Do not move the public API
- Keep the scaffold small and reviewable

### Phase 3: Pilot Non-Streaming Domains

Start with domains that are:

- read-heavy
- non-streaming
- lower-risk
- already close to straight request/response mappings

Best pilot candidates:

- `models`
- `discovery`
- `credits`
- `generation`

These are better first slices than chat/responses/messages.

### Phase 4: Expand To More Complex Management Domains

Potential next candidates:

- `api_keys`
- `auth`
- `guardrails`

Only after the basic generated seam and conversion story are proven.

### Phase 5: Re-evaluate Streaming Domains

Chat, Responses, and Messages should move last, and only if the generated layer clearly reduces maintenance cost without damaging the wrapper ergonomics.

## First-Pass Constraints

The first pass should explicitly avoid:

- rewriting `src/client.rs`
- replacing the canonical domain method names
- exposing generated modules publicly
- generating the final streaming abstractions
- rewriting `crates/openrouter-cli` around generated internals
- using overlays to paper over broad upstream design disagreements

If a proposed step requires any of those, it is too large for the first scaffold pass.

## Risks And Tradeoffs

### Risk: Generated churn pollutes reviews

Response:

- keep generated code internal
- keep generation snapshots separate from drift baselines
- migrate in small vertical slices

### Risk: Overlays become an unreviewed shadow spec

Response:

- keep overlays small and repo-reviewed
- prefer fixing generation config or wrapper logic before expanding overlays
- document why each overlay exists

### Risk: Partial migration duplicates types

Response:

- accept temporary duplication where it protects the public API
- remove duplication only after a domain has a stable generated path

### Risk: Generation pressure degrades ergonomics

Response:

- keep builders, typed tools, and streaming handwritten by default
- measure success by public API clarity, not by percentage of generated files

## Success Criteria For The Next Issue

`#154` should be considered successful if it:

- creates the directory and module seam described here
- keeps the current public SDK surface intact
- documents the local refresh path for the full generation snapshot
- does not bundle a broad endpoint rewrite

That is the right amount of mechanical progress before any real generated endpoint migration begins.
