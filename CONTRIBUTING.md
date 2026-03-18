# Contributing to openrouter-rs

Thanks for improving `openrouter-rs` and `openrouter-cli`.

This repository contains two closely related surfaces:

- `openrouter-rs`: the Rust SDK
- `openrouter-cli`: the workspace CLI companion

The best contributions keep one change focused, keep docs/examples in sync with behavior, and stay aligned with the repo's CI entrypoints.

## Before You Start

- Search existing issues and pull requests before starting overlapping work.
- For larger changes, especially public API changes or behavior changes, open or reference an issue first.
- Keep pull requests tightly scoped. Splitting refactors from behavioral changes makes review much easier.
- Never commit real API keys, management keys, or other secrets.

## Local Setup

Requirements:

- Rust `1.85+`
- `just`
- `OPENROUTER_API_KEY` for live API-backed examples and tests
- `OPENROUTER_MANAGEMENT_KEY` for management-governed live tests

Useful commands:

```bash
just quality
just quality-ci
just test-live-contract
OPENROUTER_MANAGEMENT_KEY=... just test-live-contract-management
```

Prefer the `just` recipes over ad hoc command sequences so local validation stays aligned with CI.

## Validation Expectations

For most changes:

- run `just quality`

Also run `just quality-ci` if you changed:

- CLI behavior or CLI docs
- migration docs or migration shims
- CI/release workflows
- release-facing documentation or validation flows

Live integration suites are valuable, but they require credentials and may exercise billable upstream APIs. If you cannot run them locally, say so clearly in the pull request.

## Change-Type Expectations

If you change public API surface:

- update the relevant README/docs in the same change
- update examples if the canonical usage changed
- update `CHANGELOG.md`
- call out any compatibility risk explicitly in the PR

If you change CLI behavior:

- update [`crates/openrouter-cli/README.md`](crates/openrouter-cli/README.md)
- keep JSON/table output expectations and config resolution behavior explicit

If you change migration-sensitive behavior:

- update [`MIGRATION.md`](MIGRATION.md) when needed
- run `just check-migration-docs`
- run `just test-migration-smoke`

If you change docs/examples only:

- prefer small, concrete examples over aspirational or pseudo-code-heavy docs
- keep README and example names aligned

## Pull Requests

- Link the relevant issue in the PR body.
- Use the existing PR template and fill out the validation section honestly.
- Mark breaking changes clearly.
- Keep unrelated cleanup out of behavior-changing PRs unless the cleanup is required for the change.

Reviewers will usually look for:

- correctness and compatibility risk first
- docs/examples consistency
- tests or validation appropriate to the change
- explicit migration notes when behavior changes are user-visible

## Project Policies

Contributor-facing project policies live here:

- [`docs/maintenance-policy.md`](docs/maintenance-policy.md) for release, MSRV, and breaking-change policy
- [`SECURITY.md`](SECURITY.md) for vulnerability reporting and coordinated disclosure expectations
- [`SUPPORT.md`](SUPPORT.md) for usage questions, bug reports, and support boundaries

## Support Boundaries

This repository can fix SDK/CLI bugs, docs gaps, and compatibility issues in the Rust surfaces it owns.

Upstream platform issues, account issues, provider outages, billing questions, and OpenRouter service incidents may need to be handled by OpenRouter directly unless there is a clear SDK/CLI bug in this repository.
