# HTTP Transport Migration Plan

> Status: completed in `0.8.0` on `2026-04-18`.
> This document is retained as the design baseline and validation checklist for the landed migration away from `surf`.

This document defines the recommended path for moving `openrouter-rs` away from the current `surf`-based HTTP stack and toward `reqwest` with `rustls`.

It is a follow-up design baseline for [#158](https://github.com/realmorrisliu/openrouter-rs/issues/158), where transport maintenance risk and runtime `libcurl` dependence were called out explicitly.

## Why This Exists

Today the SDK depends on:

- `surf = 2.3.2` in [`Cargo.toml`](../../Cargo.toml)
- `isahc = 0.9.14` transitively through `surf`
- `curl` / `curl-sys` transitively through `surf -> http-client -> isahc -> curl`

That stack still works, but it is not a good long-term default for this repository.

As of 2026-04-18:

- `surf` remains published at `v2.3.2` from 2021-11-01
- the `http-rs/surf` default branch last moved on 2022-05-10
- `isahc` has newer activity, but still centers `curl` and marks itself as passively maintained
- `reqwest` is actively released and maintained, with a current default TLS direction aligned with `rustls`

The key issue is not that every layer is dead. The issue is that the public SDK currently depends on the least active layer in the stack while also carrying unnecessary system-level complexity.

## Goals

- Remove the default dependency chain on runtime `libcurl` / `curl`
- Move the SDK onto a Tokio-native and actively maintained client stack
- Keep the canonical public API domain-oriented: `chat()`, `responses()`, `messages()`, `rerank()`, `audio().speech()`, `videos()`, `models()`, `management()`, and `legacy()`
- Preserve current behavior for JSON requests, auth headers, query encoding, and SSE streaming
- Reduce coupling between public error types and a specific HTTP client implementation
- Make the migration reviewable in a small number of focused PRs instead of one rewrite

## Non-Goals

- Introducing a large pluggable transport framework before the first migration lands
- Supporting multiple first-class HTTP backends in the initial migration
- Rewriting the public request builders or domain client layout
- Changing CLI config/profile behavior as part of the HTTP migration
- Reworking streaming abstractions beyond what is needed to keep behavior equivalent

## Current Constraints

The current codebase has four relevant constraints:

1. The SDK already assumes Tokio at the public usage level.

   - README examples use `#[tokio::main]`
   - the crate already depends on `tokio`
   - the CLI also uses Tokio

2. The code does not centralize an owned HTTP client yet.

   - [`src/client.rs`](../../src/client.rs) stores auth and metadata fields
   - endpoint modules currently build requests ad hoc with `surf::get/post/patch/delete`

3. `surf` leaks into shared utility and error surfaces.

   - [`src/utils.rs`](../../src/utils.rs) accepts and returns `surf::RequestBuilder`, `surf::Response`, and `surf::StatusCode`
   - [`src/error.rs`](../../src/error.rs) exposes `OpenRouterError::HttpRequest(surf::Error)` and stores `surf::StatusCode` in `ApiErrorContext`
   - tests also match against `surf::StatusCode`

4. SSE streaming is concentrated, not pervasive.

   - streaming entrypoints are mainly in chat, responses, and messages
   - the repository already owns the SSE frame parser, which is an advantage during migration

## Decision Summary

- The target stack should be `reqwest + rustls`.
- The migration should not stop at "switch `surf` to a different backend".
- Public error/status types should be de-`surf`-ed as part of the migration path.
- The first internal abstraction should be narrow helper functions, not a general transport trait.
- The migration should land in staged PRs, starting with public-type decoupling and request helpers, then endpoint conversion, then cleanup.

## Options Considered

### Option A: Stay on `surf`

Pros:

- no migration work
- existing tests mostly continue unchanged

Cons:

- keeps the least active layer in the stack as a direct dependency
- keeps `surf` types in the public error surface
- keeps the repo on a less common Tokio-era Rust HTTP stack
- does not reduce long-term maintenance risk

Decision: reject.

### Option B: Keep `surf`, but change its backend to a rustls path

Pros:

- likely the smallest short-term diff
- may remove the immediate `curl` dependency if configured correctly

Cons:

- still leaves the repo depending on `surf`
- still leaves `surf` types in shared utilities and public errors
- still leaves the transport boundary oriented around an effectively stalled abstraction layer

Decision: acceptable only as an emergency stopgap, not as the intended end state.

### Option C: Migrate directly to `isahc`

Pros:

- more active than `surf`
- closer to the current behavior

Cons:

- still tied to the curl-centric ecosystem
- does not solve the main simplification goal
- does not improve the "Tokio-native mainstream stack" story

Decision: reject as the primary migration target.

### Option D: Migrate to `reqwest + rustls`

Pros:

- active maintenance and release cadence
- strong Tokio fit
- simpler explanation for downstream users
- direct support for JSON, headers, query params, and response byte streams
- removes dependence on the stalled `surf` layer

Cons:

- requires touching shared utils, endpoint calls, and public error typing
- SSE line handling must be reworked on top of a byte stream

Decision: adopt.

## Public API And SemVer Considerations

There is one important migration hazard: the repository currently exposes `surf` in public error-related types.

Examples:

- `OpenRouterError::HttpRequest(surf::Error)`
- `ApiErrorContext.status: surf::StatusCode`

That means a full move to `reqwest` cannot honestly be treated as "purely internal" unless those public types are also normalized.

Recommended direction:

- change `ApiErrorContext.status` to `http::StatusCode`
- change `OpenRouterError::HttpRequest(...)` to store a backend-neutral error representation

Reasonable first-pass shapes:

- `HttpRequest(String)` for a minimal, stable representation
- `HttpRequest(Box<dyn std::error::Error + Send + Sync>)` if preserving source chaining matters more than exact type matching
- a small handwritten `HttpRequestError` wrapper type if the repo wants stable helpers like `is_timeout()`

The repository should prefer a backend-neutral public contract even if it means one intentional breaking change.

## Proposed Internal Shape

The migration does not need a large transport framework.

The first useful boundary is a narrow internal module, for example:

```text
src/
  http/
    mod.rs
    request.rs
    response.rs
    sse.rs
```

That module should own:

- constructing a configured `reqwest::Client`
- applying bearer auth and OpenRouter metadata headers
- sending JSON requests
- sending requests with optional query parameters
- turning error responses into the existing normalized API error model
- adapting response byte streams into line-oriented SSE parsing inputs

It should not try to become a public backend plugin system.

## Client Ownership Direction

[`src/client.rs`](../../src/client.rs) should eventually own a reusable `reqwest::Client`.

Recommended additions:

- a lazily or eagerly constructed internal client field
- builder options for future transport configuration only if there is a clear user need

Do not expand the public builder prematurely. The first migration can keep behavior narrow:

- standard headers
- default TLS via `rustls`
- connection reuse through one shared client

Timeout/proxy customization can be added later if there is real demand.

## Streaming Direction

SSE behavior should remain handwritten.

The repository already owns the meaningful streaming behavior:

- `parse_sse_frames`
- tool-aware stream adaptation
- unified stream adaptation across chat, responses, and messages

The migration should preserve that structure:

1. use `reqwest` to obtain a response byte stream
2. convert bytes into a line stream internally
3. keep the existing SSE frame parser semantics
4. keep domain-level stream adapters unchanged where possible

This keeps the risky part of the migration small and local.

## Recommended Rollout

### Phase 1: Public-Type Decoupling

- replace `surf::StatusCode` with `http::StatusCode`
- remove direct `surf::Error` exposure from `OpenRouterError`
- update tests and docs to stop referring to `surf`

This is the highest-leverage phase because it removes the main public coupling.

### Phase 2: Internal `reqwest` Helper Layer

- add a small internal HTTP module
- create a shared `reqwest::Client`
- port shared auth/header/query/JSON helpers from `src/utils.rs`
- keep endpoint behavior identical

### Phase 3: Endpoint Conversion

Migrate endpoints in this order:

1. non-streaming GET/POST/PATCH/DELETE endpoints
2. non-streaming create endpoints with JSON request bodies
3. streaming chat/responses/messages endpoints
4. feature-gated legacy completions

This order puts the simplest and broadest conversions first while isolating SSE risk until later.

### Phase 4: Cleanup

- remove `surf` from dependencies
- remove stale `surf` references from docs and tests
- confirm the lockfile no longer pulls in the curl chain through the SDK
- document the transport decision in release notes and migration guidance if the public error surface changed

## Validation Expectations

At minimum, the implementation follow-up should run:

- `just quality`
- `just test-cli`
- `just test-integration-subsets`
- `just check-migration-docs` if public migration wording changes

Before removing `surf`, review should also confirm:

- request line, auth header, and JSON body unit tests still pass
- SSE tests still parse multi-line `data:` frames and `[DONE]` termination correctly
- live API integration behavior is still normal for chat, responses, messages, models, and management flows

## Acceptance Criteria For The Implementation Follow-Up

- `surf` is no longer a direct dependency of `openrouter-rs`
- the SDK no longer exposes `surf` types in public error/status surfaces
- chat, responses, and messages streaming behavior remains covered by tests
- README/examples remain aligned with the canonical domain-oriented client surface
- the final dependency story is simpler to explain: Tokio runtime plus `reqwest` with `rustls`

## Out Of Scope For The First Migration PR

- alternate HTTP backend selection
- WASM transport support
- retry middleware policy
- advanced proxy, DNS, or HTTP/3 configuration
- transport metrics hooks

Those can be revisited later from a cleaner baseline.
