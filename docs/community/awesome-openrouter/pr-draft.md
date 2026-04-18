# PR Title

Add `openrouter-rs` to Awesome OpenRouter

# PR Body

## Summary

Add `openrouter-rs` as the default community Rust SDK entry for OpenRouter, with the companion `openrouter-cli` included in the same project story.

## Why this belongs in Awesome OpenRouter

- It is a public, open-source Rust project built specifically around OpenRouter
- Users bring their own `OPENROUTER_API_KEY`
- It has a stable public docs/setup page for OpenRouter-specific usage
- The repo already publishes both a typed SDK and a companion CLI for account and workflow automation

## Traction / Notability

Snapshot date: `2026-04-17`

- GitHub: `62` stars, `15` forks
- `openrouter-rs` on crates.io: `11,900` total downloads, `4,232` recent downloads
- `openrouter-cli` on crates.io: published companion CLI with `59` downloads across its first three releases
- docs.rs documentation is live and the repo includes runnable SDK and CLI examples
- The repository publishes an endpoint matrix against the current OpenRouter spec and tracks nightly OpenAPI drift in-repo

## Submission Links

- Project URL: <https://github.com/realmorrisliu/openrouter-rs>
- Docs URL: <https://github.com/realmorrisliu/openrouter-rs/blob/main/docs/community/awesome-openrouter/README.md>
- Open source: <https://github.com/realmorrisliu/openrouter-rs>

## Notes For Maintainers

- The listing should be named `openrouter-rs`
- The Rust CLI lives in the same repository as `openrouter-cli`
- No README changes are included here; this PR only adds `apps/openrouter-rs/app.yaml` and `apps/openrouter-rs/logo.png`
