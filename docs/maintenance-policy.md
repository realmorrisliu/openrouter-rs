# Maintenance Policy

This document describes the repository's contributor-facing maintenance expectations for releases, MSRV, and breaking changes.

## Release Policy

The repository aims to keep `main` in a releasable state.

Contributor expectations:

- run `just quality` for normal changes
- run `just quality-ci` when touching CLI behavior, migration docs, or CI/release-aligned validation flows
- update public docs/examples in the same change when public behavior changes
- update `CHANGELOG.md` for user-visible changes

SDK releases:

- use tags in the form `v<version>`
- the tag must match the root `Cargo.toml` version
- the release workflow verifies formatting, clippy, unit tests, and `cargo package --locked`
- crates.io publishing occurs only when the registry token is configured

CLI releases:

- use tags in the form `openrouter-cli-v<version>`
- the tag must match `crates/openrouter-cli/Cargo.toml`
- the CLI release workflow verifies CLI tests, packages the crate, publishes to crates.io, and builds release archives

When a change affects the public API surface, release-facing docs should be updated in the same pull request rather than deferred.

## MSRV Policy

The current MSRV is Rust `1.85`.

MSRV expectations:

- CI enforces the current MSRV for `cargo check` on all targets with default and all features
- MSRV bumps must update the relevant source of truth, including `Cargo.toml`, CI, and README/docs references
- MSRV bumps should ship in a new minor release line, not a patch release

If a dependency causes an accidental MSRV regression, the preferred response is to pin, update, or otherwise restore the documented MSRV when feasible.

## Breaking-Change Policy

This project follows Semantic Versioning, with explicit pre-`1.0` expectations:

- patch releases within the same `0.x` line should remain backward compatible except for urgent bug or security fixes
- user-visible breaking changes require at least a new minor release line
- when feasible, introduce deprecation bridges before removals instead of removing old APIs immediately

Breaking changes should:

- be called out explicitly in the PR body
- be documented in `CHANGELOG.md`
- update README/examples if canonical usage changed
- update [`MIGRATION.md`](../MIGRATION.md) when a migration path is needed

When a compatibility bridge exists, removals should normally be announced before they are deleted in a later release line.
