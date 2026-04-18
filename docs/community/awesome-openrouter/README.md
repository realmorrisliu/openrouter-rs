# Awesome OpenRouter Submission Kit

This directory is the repo-side source of truth for submitting this project to [`OpenRouterTeam/awesome-openrouter`](https://github.com/OpenRouterTeam/awesome-openrouter).

## Submission Decision

Submit a single `openrouter-rs` entry and position it as the default community Rust SDK plus companion CLI for OpenRouter.

Recommended wording:

- Name: `openrouter-rs`
- One-line description: `Community-maintained Rust SDK and companion CLI for OpenRouter, with typed clients for chat, responses, messages, models, embeddings, streaming, and management workflows.`
- Tags: `coding`
- Open-source URL: `https://github.com/realmorrisliu/openrouter-rs`

Why this is the right shape:

- The repo, docs.rs package, and most ecosystem recognition are already anchored on `openrouter-rs`.
- `openrouter-cli` is real distribution surface, but it is best presented as a companion inside the same repo rather than as a second listing that splits traction.
- A single entry keeps stars, crates downloads, examples, and maintenance signals consolidated for reviewers.

## Stable Links

Use these links in the external submission:

- Project URL: `https://github.com/realmorrisliu/openrouter-rs`
- Docs URL: `https://github.com/realmorrisliu/openrouter-rs/blob/main/docs/community/awesome-openrouter/README.md`
- Open-source URL: `https://github.com/realmorrisliu/openrouter-rs`

## OpenRouter Setup

This page exists so the Awesome OpenRouter `docs:` field can point to a stable public page with basic OpenRouter setup guidance.

### SDK

Add the crate:

```toml
[dependencies]
openrouter-rs = "0.7.0"
tokio = { version = "1", features = ["full"] }
```

Set your OpenRouter API key:

```bash
export OPENROUTER_API_KEY=sk-or-v1-...
```

Start from the canonical examples:

- SDK README: [`README.md`](../../../README.md)
- docs.rs API docs: <https://docs.rs/openrouter-rs>
- Chat example: [`examples/domain_chat_completion.rs`](../../../examples/domain_chat_completion.rs)
- Responses example: [`examples/create_response.rs`](../../../examples/create_response.rs)
- Messages example: [`examples/create_message.rs`](../../../examples/create_message.rs)
- Typed tools example: [`examples/typed_tool_calling.rs`](../../../examples/typed_tool_calling.rs)

### CLI

The same repo also publishes `openrouter-cli` for profile, discovery, billing, and management workflows:

```bash
cargo install openrouter-cli
export OPENROUTER_API_KEY=sk-or-v1-...
openrouter-cli --help
```

Useful entrypoints:

- CLI README: [`crates/openrouter-cli/README.md`](../../../crates/openrouter-cli/README.md)
- Show active profile: `openrouter-cli profile show`
- List models: `openrouter-cli models list`
- Check credits: `openrouter-cli usage credits`

## Included Assets

- Draft listing entry: [`app.yaml`](app.yaml)
- Draft external PR body: [`pr-draft.md`](pr-draft.md)
- SVG source logo: [`logo.svg`](logo.svg)
- PNG submission logo: [`logo.png`](logo.png)

## Traction Snapshot

Use the following reviewer-facing signals in the Awesome OpenRouter PR body. Snapshot date: `2026-04-17`.

- GitHub repository: `62` stars and `15` forks
- `openrouter-rs` crate: `11,900` total downloads and `4,232` recent downloads on crates.io
- `openrouter-cli` crate: published companion CLI with `59` downloads across its first three releases
- Published API docs on docs.rs and runnable examples in-repo
- The repository publishes an endpoint matrix against the current OpenRouter spec and tracks nightly OpenAPI drift in-repo
- Contributor, security, support, release, MSRV, and breaking-change policies are public in-repo

Refresh the numbers above if submission happens materially later than this snapshot date.

## External Submission Checklist

Before opening the PR against `OpenRouterTeam/awesome-openrouter`:

1. Copy [`app.yaml`](app.yaml) and [`logo.png`](logo.png) into `apps/openrouter-rs/`
2. Reconfirm the traction numbers and `date_added`
3. Use [`pr-draft.md`](pr-draft.md) as the external PR body
4. Do not edit the Awesome OpenRouter README directly
