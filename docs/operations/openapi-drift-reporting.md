# OpenAPI Drift Reporting

`openrouter-rs` tracks a checked-in OpenRouter OpenAPI baseline and compares it against the latest upstream spec on a weekly schedule.

## Why This Exists

The endpoint matrix is useful, but it is static until someone remembers to refresh it. The drift workflow closes that gap:

- it fetches the latest upstream `openapi.json`
- it compares the latest upstream operations against the tracked baseline
- it emits a human-readable report and a machine-readable JSON summary
- it can open or refresh a follow-up GitHub issue when actionable drift is detected

This keeps `openrouter-rs` aligned with upstream changes without blocking releases on every spec delta.

## Tracked Baseline

- Tracked baseline snapshot: `specs/openrouter/openapi-baseline.json`
- Normalized operation snapshot: `specs/openrouter/openapi-baseline.operations.json`
- Weekly workflow: `.github/workflows/openapi-drift.yml`

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

After the raw OpenAPI comparison, the report also applies a small repo-aware classification pass.
Today that pass recognizes changes already handled by the SDK transport and flexible schema
surfaces:

- global request metadata headers (`X-OpenRouter-Title`, `X-Title`, `HTTP-Referer`, and `X-OpenRouter-Categories`)
- dynamic provider-name and output-modality enums surfaced as `String` values by the SDK
- provider-specific passthrough options surfaced as `HashMap<String, Value>`
- flexible plugin payloads surfaced as `Plugin` configuration maps
- Anthropic Messages hosted-tool options surfaced through `AnthropicTool::extra`
- Responses tool and output payloads surfaced as `Value`
- Responses result nullable annotations covered by `Option`/`Value` response parsing

This lets the report separate:

- raw upstream drift
- changed operations that are already covered by the repo's existing handling
- changed operations that still need SDK/docs/test follow-up

This keeps the weekly issue useful when upstream bulk-edits supported metadata or dynamic taxonomy
schemas across many operations without hiding the underlying OpenAPI drift artifacts.

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

The compare command also emits both `has_drift` and `has_actionable_drift` when GitHub Actions
passes a `GITHUB_OUTPUT` file. The weekly workflow keeps uploading the full raw drift artifacts,
but it only opens or refreshes the follow-up issue when `has_actionable_drift=true`.

Refresh the tracked baseline after reviewing and accepting upstream changes:

```bash
just openapi-refresh-baseline
```

That updates:

- `specs/openrouter/openapi-baseline.json`
- `specs/openrouter/openapi-baseline.operations.json`

## Follow-Up Flow

When the weekly workflow reports actionable drift:

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
