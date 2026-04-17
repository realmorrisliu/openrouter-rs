# OpenRouter Spec Baseline

This directory contains the tracked upstream OpenRouter OpenAPI baseline used by the nightly drift-detection workflow.

Files:

- `openapi-baseline.json`: checked-in baseline snapshot used for path+operation drift comparison
- `openapi-baseline.operations.json`: normalized operation summary derived from the raw baseline

The initial baseline is seeded from the currently accepted endpoint matrix in this repo. That is
intentional: drift detection should surface newly added upstream operations instead of silently
accepting them before docs, tests, and implementation are reviewed.

Refresh the baseline locally with:

```bash
just openapi-refresh-baseline
```

Compare the tracked baseline against the latest upstream spec with:

```bash
just openapi-drift-check
```

This directory is intentionally limited to baseline tracking for now. Generated-module seams, overlays, and broader spec tooling remain future work under the generated-core scaffold roadmap described in [`docs/generated-core-architecture.md`](../../docs/generated-core-architecture.md).
