# Release Targets for openrouter-rs

Use this checklist to avoid missing version updates.

## Required files

1. `Cargo.toml`
- Update `[package].version`.

2. `CHANGELOG.md`
- Keep `## [Unreleased]` at top.
- Add `## [x.y.z] - YYYY-MM-DD` for the release.
- Move relevant entries from Unreleased into the new section.

3. `README.md`
- Installation block: `openrouter-rs = "x.y.z"`.
- Bottom section: `## 📈 Release History`.
- Add `### Version x.y.z *(Latest)*` and remove `*(Latest)*` from prior latest.

## Recommended file

1. `src/lib.rs`
- Check doc example dependency line `openrouter-rs = "..."` and keep it aligned.

## Grep checks

Run these commands from repo root:

```bash
rg -n '^version\s*=\s*"' Cargo.toml
rg -n 'openrouter-rs\s*=\s*"' README.md src/lib.rs
rg -n '^## \[Unreleased\]|^## \[[0-9]+\.[0-9]+\.[0-9]+\]' CHANGELOG.md
rg -n '^## 📈 Release History|^### Version ' README.md
```

## Consistency rule

The release version should be consistent across:

- `Cargo.toml` package version
- README installation snippet
- README bottom latest version header
- latest version section in `CHANGELOG.md`
