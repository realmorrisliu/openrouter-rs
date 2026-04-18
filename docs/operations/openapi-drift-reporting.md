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

The comparison is operation-level (`METHOD /path`). It first resolves local `#/components/...`
references, including referenced Path Item objects, then folds in effective defaults before hashing:

- Path Item inheritance: `parameters`, `servers`
- OpenAPI root defaults: `servers`, `security`
- Referenced security scheme definitions for the effective `security` requirements
- Order-insensitive canonicalization for `parameters` and `security` requirement lists
- Order-insensitive canonicalization for known unordered JSON Schema collections such as `required`, `enum`, `type`, `allOf`, `anyOf`, and `oneOf`

It intentionally ignores docs-only fields:

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

1. Keep the generated issue open as the active compatibility-update record.
2. Review the generated report and candidate operations snapshot artifact.
3. Decide whether the drift is accepted now, deferred, or intentionally out of scope.
4. Update `docs/operations/official-endpoint-test-matrix.md` if the reviewed upstream operation surface changed.
5. Update `CHANGELOG.md` and `MIGRATION.md` when the accepted change modifies user-visible behavior or migration expectations.
6. If the change is accepted as the new baseline, run `just openapi-refresh-baseline` and commit the refreshed artifacts.

For the reporting cadence and surface-selection rules behind those steps, see [`docs/policies/compatibility-update-policy.md`](../policies/compatibility-update-policy.md).

## Relationship To The Endpoint Matrix

The drift workflow does not replace [`docs/operations/official-endpoint-test-matrix.md`](official-endpoint-test-matrix.md).

Their roles are different:

- Drift reporting answers: "Did the upstream OpenAPI change?"
- The endpoint matrix answers: "Do we implement and test that surface today?"

When the drift report changes, the matrix is the next review surface.
