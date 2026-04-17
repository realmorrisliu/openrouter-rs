# Accepted Source Snapshot

This directory is reserved for the accepted full upstream OpenAPI snapshot that future generated
internal modules will use as input.

It is intentionally separate from the drift baseline in `../openapi-baseline.json`.

- drift baseline: review-oriented coverage checkpoint
- source snapshot: fuller accepted generation input

Refresh the tracked source snapshot with:

```bash
just openapi-refresh-source
```

The command writes `openapi.json` into this directory.
