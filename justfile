set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

check:
    cargo check --all-targets

check-all-features:
    cargo check --all-targets --all-features

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test-unit:
    cargo test --test unit

test-lib:
    cargo test --lib

test-doc:
    cargo test --doc

test-integration-subsets:
    cargo test --test integration model_pool:: -- --nocapture
    cargo test --test integration tools:: -- --nocapture

test-integration:
    test -n "${OPENROUTER_API_KEY:-}"
    cargo test --test integration -- --nocapture

test-live-contract:
    cargo test --test integration contract:: -- --nocapture

test-live-contract-management:
    OPENROUTER_RUN_MANAGEMENT_TESTS=1 cargo test --test integration management:: -- --nocapture

test-cli:
    cargo test -p openrouter-cli

package-cli:
    # Release-only validation: requires the SDK dependency version to exist on crates.io.
    cargo package -p openrouter-cli --locked

check-migration-docs:
    ./scripts/check_migration_docs.sh

test-migration-smoke:
    cargo test --test migration_smoke --all-features

quality: fmt-check check clippy test-unit test-lib test-doc

quality-ci: quality test-integration-subsets test-cli check-migration-docs test-migration-smoke
