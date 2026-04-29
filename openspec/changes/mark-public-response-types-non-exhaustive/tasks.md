## 1. Inventory

- [x] 1.1 Enumerate all `pub struct` and `pub enum` definitions under `src/api/` and `src/types/`.
- [x] 1.2 Classify each public type as request/configuration, response/data, stream event, upstream taxonomy, SDK helper, or internal wire type.
- [x] 1.3 Record which public types are already `#[non_exhaustive]` and which high-churn types remain exhaustive.
- [x] 1.4 Decide and document any public types that should intentionally remain exhaustive.

## 2. Construction Paths

- [x] 2.1 Verify every caller-built request/configuration type selected for `#[non_exhaustive]` has a builder, constructor, or helper.
- [x] 2.2 Add builders or constructors where construction is expected but no ergonomic path exists.
- [x] 2.3 Update examples and unit tests that currently rely on direct literals for selected caller-built types.

## 3. Non-Exhaustive Implementation

- [x] 3.1 Add `#[non_exhaustive]` to selected response/data/usage/pricing/discovery model structs.
- [x] 3.2 Add `#[non_exhaustive]` to selected request/configuration structs after construction paths are verified.
- [x] 3.3 Add `#[non_exhaustive]` to selected upstream taxonomy enums and update internal matches if needed.
- [x] 3.4 Keep private wire structs and deliberately stable public helpers exhaustive unless the inventory says otherwise.

## 4. Documentation

- [x] 4.1 Update `MIGRATION.md` with the 0.10.0 source-level migration guidance for direct struct literals and exhaustive enum matches.
- [x] 4.2 Update `CHANGELOG.md` with the breaking API-surface change.
- [x] 4.3 Update `README.md` and examples if they show construction patterns affected by the change.
- [x] 4.4 Note that this work belongs to `0.10.0`, not a `0.9.x` patch release.

## 5. Verification

- [x] 5.1 Run `cargo fmt --all --check`.
- [x] 5.2 Run `cargo check --all-targets`.
- [x] 5.3 Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] 5.4 Run `cargo test --test unit`.
- [x] 5.5 Run `cargo test --lib` and `cargo test --doc`.
- [x] 5.6 Run `just check-migration-docs`.
- [x] 5.7 Run `cargo test --test migration_smoke --all-features`.
- [x] 5.8 Run `just quality-ci`.
