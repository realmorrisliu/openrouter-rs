## 1. Inventory

- [ ] 1.1 Enumerate all `pub struct` and `pub enum` definitions under `src/api/` and `src/types/`.
- [ ] 1.2 Classify each public type as request/configuration, response/data, stream event, upstream taxonomy, SDK helper, or internal wire type.
- [ ] 1.3 Record which public types are already `#[non_exhaustive]` and which high-churn types remain exhaustive.
- [ ] 1.4 Decide and document any public types that should intentionally remain exhaustive.

## 2. Construction Paths

- [ ] 2.1 Verify every caller-built request/configuration type selected for `#[non_exhaustive]` has a builder, constructor, or helper.
- [ ] 2.2 Add builders or constructors where construction is expected but no ergonomic path exists.
- [ ] 2.3 Update examples and unit tests that currently rely on direct literals for selected caller-built types.

## 3. Non-Exhaustive Implementation

- [ ] 3.1 Add `#[non_exhaustive]` to selected response/data/usage/pricing/discovery model structs.
- [ ] 3.2 Add `#[non_exhaustive]` to selected request/configuration structs after construction paths are verified.
- [ ] 3.3 Add `#[non_exhaustive]` to selected upstream taxonomy enums and update internal matches if needed.
- [ ] 3.4 Keep private wire structs and deliberately stable public helpers exhaustive unless the inventory says otherwise.

## 4. Documentation

- [ ] 4.1 Update `MIGRATION.md` with the 0.10.0 source-level migration guidance for direct struct literals and exhaustive enum matches.
- [ ] 4.2 Update `CHANGELOG.md` with the breaking API-surface change.
- [ ] 4.3 Update `README.md` and examples if they show construction patterns affected by the change.
- [ ] 4.4 Note that this work belongs to `0.10.0`, not a `0.9.x` patch release.

## 5. Verification

- [ ] 5.1 Run `cargo fmt --all --check`.
- [ ] 5.2 Run `cargo check --all-targets`.
- [ ] 5.3 Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] 5.4 Run `cargo test --test unit`.
- [ ] 5.5 Run `cargo test --lib` and `cargo test --doc`.
- [ ] 5.6 Run `just check-migration-docs`.
- [ ] 5.7 Run `cargo test --test migration_smoke --all-features`.
