---
name: openrouter-rs-release
description: Prepare and publish a new openrouter-rs version with consistent version updates and release notes. Use when asked to cut a release, bump crate versions, publish to crates.io, update changelog entries, refresh README installation version, and update the README bottom Release History latest section.
---

# OpenRouter-rs Release

## Overview

Use this skill to run the full `openrouter-rs` release workflow without missing version sync points.
Always update `Cargo.toml`, `CHANGELOG.md`, and README release sections together before tagging.

## Release Inputs

Collect these values first:

- Target version (for example `0.5.1`)
- Release date in `YYYY-MM-DD`
- Summary bullets grouped by `Added`, `Changed`, `Fixed` (reuse from `CHANGELOG.md` when possible)

## Update Versioned Files

Apply updates in this order to avoid drift.

1. Update package version in `Cargo.toml`.
2. Update README installation snippet version (`openrouter-rs = "..."`).
3. Update any outdated docs snippet that pins a crate version (notably `src/lib.rs` doctext if present).
4. Update `CHANGELOG.md`:
- Move finalized items from `## [Unreleased]` into a new version section.
- Create `## [<version>] - <date>` with `### Added`/`### Changed`/`### Fixed` headings as needed.
- Keep `## [Unreleased]` at the top for next cycle.
5. Update README bottom `## 📈 Release History`:
- Insert a new `### Version <version> *(Latest)*` section at the top of the history list.
- Remove `*(Latest)*` marker from the previous latest version.
- Copy concise release bullets aligned with the changelog section.

Use [references/release-targets.md](references/release-targets.md) to verify exact files and grep checks.

## Validate Before Commit

Run local checks in this order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --test unit`
4. `bash .agents/skills/openrouter-rs-release/scripts/verify_release_sync.sh <version>`

If `verify_release_sync.sh` reports mismatch, fix files before commit/tag.

## Commit, Tag, and Publish

When all checks pass:

1. Commit release prep changes.
2. Push to `main` via PR according to repo policy.
3. Create and push the release tag: `v<version>`.
4. Confirm GitHub Actions `Release` workflow passes:
- verify job
- crates.io publish (requires `CARGO_REGISTRY_TOKEN`)
- GitHub release creation

## Release Note Composition

Use `CHANGELOG.md` as the source of truth.

- Keep GitHub release notes and README bottom latest bullets semantically aligned.
- Prefer concise bullets that explain user-visible impact.
- Mention breaking changes explicitly.

## Quick Execution Checklist

- [ ] `Cargo.toml` version bumped
- [ ] README installation version updated
- [ ] `src/lib.rs` pinned crate snippet updated (if present)
- [ ] `CHANGELOG.md` new version section created
- [ ] README bottom Release History latest section updated
- [ ] local checks passed (`fmt`, `clippy`, `unit`)
- [ ] version sync script passed
- [ ] tag `v<version>` pushed
- [ ] release workflow green
