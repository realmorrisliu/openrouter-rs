# OpenAPI Drift Reporting

`openrouter-rs` tracks a checked-in OpenRouter OpenAPI baseline and compares it against the latest upstream spec on a nightly schedule.

## Why This Exists

The endpoint matrix is useful, but it is static until someone remembers to refresh it. The drift workflow closes that gap:

- it fetches the latest upstream `openapi.json`
- it compares the latest upstream operations against the tracked baseline
- it emits a human-readable report and a machine-readable JSON summary
- it can open or refresh a follow-up GitHub issue when drift is detected

This keeps `openrouter-rs` aligned with upstream changes without blocking releases on every spec delta.

## Tracked Baseline

- Tracked baseline snapshot: `specs/openrouter/openapi-baseline.json`
- Normalized operation snapshot: `specs/openrouter/openapi-baseline.operations.json`
- Nightly workflow: `.github/workflows/openapi-drift.yml`

The comparison is operation-level (`METHOD /path`). It resolves local `#/components/...`
references before hashing and intentionally ignores docs-only fields:

- `summary`
- `description`
- `title`
- `example`
- `examples`
- `externalDocs`

That keeps the report focused on compatibility-relevant API changes rather than upstream prose edits.

The initial tracked baseline is intentionally seeded from the accepted endpoint matrix rather
than silently fast-forwarded to whatever the latest upstream spec happens to contain. Baseline
refresh is an explicit review step, not an automatic side effect of detection.

## Local Commands

Compare the tracked baseline against the latest upstream spec:

```bash
just openapi-drift-check
```

This writes a report to:

- `/tmp/openrouter-openapi-drift-report.md`
- `/tmp/openrouter-openapi-drift-report.json`
- `/tmp/openrouter-openapi-latest.operations.json`

Refresh the tracked baseline after reviewing and accepting upstream changes:

```bash
just openapi-refresh-baseline
```

That updates:

- `specs/openrouter/openapi-baseline.json`
- `specs/openrouter/openapi-baseline.operations.json`

## Follow-Up Flow

When the nightly workflow reports drift:

1. Review the generated report and candidate operations snapshot artifact.
2. Update `docs/official-endpoint-test-matrix.md` if the upstream operation surface changed.
3. Decide whether the SDK, tests, docs, or migration notes need follow-up work.
4. If the change is accepted as the new baseline, run `just openapi-refresh-baseline` and commit the refreshed artifacts.

## Relationship To The Endpoint Matrix

The drift workflow does not replace [`docs/official-endpoint-test-matrix.md`](official-endpoint-test-matrix.md).

Their roles are different:

- Drift reporting answers: "Did the upstream OpenAPI change?"
- The endpoint matrix answers: "Do we implement and test that surface today?"

When the drift report changes, the matrix is the next review surface.
