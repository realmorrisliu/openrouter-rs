# Compatibility Update Policy

This document defines how `openrouter-rs` records upstream OpenRouter compatibility changes between crate releases.

It complements:

- [`docs/maintenance-policy.md`](maintenance-policy.md) for release, MSRV, and breaking-change rules
- [`docs/openapi-drift-reporting.md`](openapi-drift-reporting.md) for nightly spec-drift detection
- [`CHANGELOG.md`](../CHANGELOG.md) and [`MIGRATION.md`](../MIGRATION.md) for durable user-facing notes

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
| GitHub issue using [`.github/ISSUE_TEMPLATE/upstream-compatibility-update.md`](../.github/ISSUE_TEMPLATE/upstream-compatibility-update.md) | Active working record for one upstream change or one drift batch | When drift is detected, a non-spec upstream change is noticed, or follow-up work needs coordination |
| [`CHANGELOG.md`](../CHANGELOG.md) | Durable user-facing summary of accepted compatibility-affecting repo changes | In the same PR that lands a user-visible SDK/docs/test/support change |
| [`MIGRATION.md`](../MIGRATION.md) | Durable upgrade guidance | When canonical usage, public API names, compatibility bridges, required config, or migration steps changed |
| [`docs/official-endpoint-test-matrix.md`](official-endpoint-test-matrix.md) | Current implementation and live-test status by operation | When the accepted upstream operation surface changed or support status changed |
| [`docs/openapi-drift-reporting.md`](openapi-drift-reporting.md) | Detection workflow and baseline refresh rules | When the nightly detection/reporting mechanics change |

This repository does not maintain a separate compatibility newsletter or a second standalone changelog.

The working issue is for triage and coordination. The durable sources of truth remain `CHANGELOG.md`, `MIGRATION.md`, and the endpoint matrix.

## Cadence

The cadence is intentionally small and event-driven:

| Cadence | Trigger | Expected output |
| --- | --- | --- |
| Nightly | Upstream OpenAPI drift check | Auto-opened or refreshed drift issue plus report artifact |
| As needed | Non-spec upstream change noticed by maintainers, tests, or users | Manual compatibility update issue using the reusable template |
| Same PR as acceptance | Repo accepts the change and updates code/docs/tests/baseline | `CHANGELOG.md`, `MIGRATION.md`, endpoint matrix, or baseline refresh updated in the same PR as needed |
| Next release cut | A version is published | Release notes summarize already-landed compatibility updates; release notes do not replace the earlier docs updates |

The important rule is: do not wait for a release cut to document a compatibility-relevant change once the repo has accepted it.

## Update Rules

When a compatibility update issue is opened:

1. Record the upstream source and the affected surface.
2. Decide whether the change is accepted now, deferred, or intentionally out of scope.
3. Link the follow-up PR or implementation issue.

When a compatibility update is accepted:

- update [`docs/official-endpoint-test-matrix.md`](official-endpoint-test-matrix.md) if the operation surface or support status changed
- update [`CHANGELOG.md`](../CHANGELOG.md) if a user-visible SDK/docs/test/support change landed
- update [`MIGRATION.md`](../MIGRATION.md) if callers need a concrete upgrade path or if a compatibility bridge changed
- refresh the tracked baseline with `just openapi-refresh-baseline` if the upstream OpenAPI change is accepted as the new reviewed baseline

When no migration path is required, do not force `MIGRATION.md` churn. The threshold for touching `MIGRATION.md` is a real caller-facing action item.

## Reusable Template

Use [`.github/ISSUE_TEMPLATE/upstream-compatibility-update.md`](../.github/ISSUE_TEMPLATE/upstream-compatibility-update.md) for manual reports and for follow-up issues derived from the nightly drift workflow.

The template deliberately asks for:

- the trigger source
- the upstream link or evidence
- the expected repo follow-up surfaces
- the user impact
- linked implementation work

That keeps compatibility updates consistent whether they originate from the nightly drift bot, a live contract failure, or a maintainer noticing an upstream change in docs/releases.
