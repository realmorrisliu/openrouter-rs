# Generation Overlays

This directory holds repo-reviewed overlays that adjust future generated output without changing the
public handwritten SDK surface directly.

The first scaffold keeps these overlays intentionally minimal. They exist to define the seam, not
to start a broad overlay-driven rewrite.

Initial overlay buckets:

- `auth.yaml`: auth-specific adjustments and endpoint-key routing metadata
- `naming.yaml`: naming overrides when upstream operation names do not fit the Rust wrapper layer
- `rust.yaml`: Rust-specific generation knobs that should remain reviewable in-repo
