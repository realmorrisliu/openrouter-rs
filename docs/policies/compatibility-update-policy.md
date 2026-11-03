# Compatibility Update Policy

This document defines how `openrouter-rs` records upstream OpenRouter compatibility changes between crate releases.

It complements:

- [`docs/policies/maintenance-policy.md`](maintenance-policy.md) for release, MSRV, and breaking-change rules
- [`docs/operations/openapi-drift-reporting.md`](../operations/openapi-drift-reporting.md) for weekly spec-drift detection
- [`CHANGELOG.md`](../../CHANGELOG.md) and [`MIGRATION.md`](../../MIGRATION.md) for durable user-facing notes

The goal is to keep upstream-alignment reporting predictable without creating a heavy editorial workflow.

## What Counts As A Compatibility Update

Use a compatibility update when the trigger comes from upstream OpenRouter behavior or from this repo's accepted alignment decision, not just from a normal repository change.

Compatibility updates include:

- OpenAPI method/path additions, removals, or operation-shape drift
- auth, headers, or endpoint-governance changes introduced upstream
- upstream behavior changes found in live contract/integration testing, even if they are not visible in the OpenAPI
- compatibility-bridge decisions, deprecations, or migration guidance updates driven by upstream alignment work
- changes to the endpoint matrix, support status, or documented "supported vs not-yet-supported" claims

A normal release note is enough when the change is purely repo-local and does not materially change the upstream compatibility story.

Some changes are both:

- repo implements or documents an upstream API change
- repo removes or narrows a compatibility bridge because upstream alignment changed

In those cases, keep the compatibility update flow and also update the normal release surfaces.

## Reporting Surfaces

Compatibility updates use different surfaces for different jobs:

| Surface | Role | When to update |
| --- | --- | --- |
| GitHub issue using [`.github/ISSUE_TEMPLATE/upstream-compatibility-update.md`](../../.github/ISSUE_TEMPLATE/upstream-compatibility-update.md) | Active working record for one upstream change or one drift batch | When actionable drift is detected, a non-spec upstream change is noticed, or follow-up work needs coordination |
| [`CHANGELOG.md`](../../CHANGELOG.md) | Durable user-facing summary of accepted compatibility-affecting repo changes | In the same PR that lands a user-visible SDK/docs/test/support change |
| [`MIGRATION.md`](../../MIGRATION.md) | Durable upgrade guidance | When canonical usage, public API names, compatibility bridges, required config, or migration steps changed |
| [`docs/operations/official-endpoint-test-matrix.md`](../operations/official-endpoint-test-matrix.md) | Current implementation and live-test status by operation | When the accepted upstream operation surface changed or support status changed |
| [`docs/operations/openapi-drift-reporting.md`](../operations/openapi-drift-reporting.md) | Detection workflow and baseline refresh rules | When the weekly detection/reporting mechanics change |

This repository does not maintain a separate compatibility newsletter or a second standalone changelog.

The working issue is for triage and coordination. The durable sources of truth remain `CHANGELOG.md`, `MIGRATION.md`, and the endpoint matrix.

## Cadence

The cadence is intentionally small and event-driven:

| Cadence | Trigger | Expected output |
| --- | --- | --- |
| Weekly | Upstream OpenAPI drift check | Report artifact every run; auto-opened or refreshed issue only when repo-aware classification marks the drift as actionable |
| As needed | Non-spec upstream change noticed by maintainers, tests, or users | Manual compatibility update issue using the reusable template |
| Same PR as acceptance | Repo accepts the change and updates code/docs/tests/baseline | `CHANGELOG.md`, `MIGRATION.md`, endpoint matrix, or baseline refresh updated in the same PR as needed |
| Next release cut | A version is published | Release notes summarize already-landed compatibility updates; release notes do not replace the earlier docs updates |

The important rule is: do not wait for a release cut to document a compatibility-relevant change once the repo has accepted it. That rule governs documentation, not publishing.

### Release cadence

Accepting drift and publishing a version are separate decisions. Document the change as soon as it lands; release it on a schedule.

- **Batch drift into scheduled releases.** Aim for at most one release per week. Drift accepted between releases waits in `CHANGELOG.md` under `## [Unreleased]`.
- **One minor bump per batch.** Upstream removals and renames are breaking, so a batch that contains any of them ships as one minor release with one `MIGRATION.md` section. Do not cut a minor per upstream change.
- **Patch releases are for defects, not alignment.** A patch may ship for a security issue, data loss, or a claim in the docs/API that is actively false (for example, documented calls that no longer compile).
- **Compatible drift can wait.** Additive fields and new operations do not need their own release; they ride the next scheduled one.

Why this is written down: between 2026-10-13 and 2026-11-03 the SDK published six versions, five of them breaking minors. Every one was correct in isolation, but a consumer pinning `openrouter-rs = "0.18"` had to absorb five migrations in three weeks to stay current. Batching keeps drift current without making the crate impossible to follow.

## Update Rules

When a compatibility update issue is opened:

1. Record the upstream source and the affected surface.
2. Decide whether the change is accepted now, deferred, or intentionally out of scope.
3. Link the follow-up PR or implementation issue.

When a compatibility update is accepted:

- update [`docs/operations/official-endpoint-test-matrix.md`](../operations/official-endpoint-test-matrix.md) if the operation surface or support status changed
- update [`CHANGELOG.md`](../../CHANGELOG.md) if a user-visible SDK/docs/test/support change landed
- update [`MIGRATION.md`](../../MIGRATION.md) if callers need a concrete upgrade path or if a compatibility bridge changed
- refresh the tracked baseline with `just openapi-refresh-baseline` if the upstream OpenAPI change is accepted as the new reviewed baseline

When no migration path is required, do not force `MIGRATION.md` churn. The threshold for touching `MIGRATION.md` is a real caller-facing action item.

## Reusable Template

Use [`.github/ISSUE_TEMPLATE/upstream-compatibility-update.md`](../../.github/ISSUE_TEMPLATE/upstream-compatibility-update.md) for manual reports and for follow-up issues derived from the weekly drift workflow.

The template deliberately asks for:

- the trigger source
- the upstream link or evidence
- the expected repo follow-up surfaces
- the user impact
- linked implementation work

That keeps compatibility updates consistent whether they originate from the weekly drift bot, a live contract failure, or a maintainer noticing an upstream change in docs/releases.
