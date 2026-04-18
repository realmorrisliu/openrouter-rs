# Docs Map

This repository keeps documentation in a few different places on purpose:

- root-level docs for the main project entrypoints and durable release history
- `docs/` for contributor-facing policy, design, operations, and distribution docs
- `specs/` for OpenAPI baseline and generation input assets
- subsystem READMEs for focused areas such as the CLI and integration test harness

If you are not sure where to start, use the groups below.

## Start Here

- [`README.md`](../README.md) for the SDK overview, canonical API surface, examples, and development entrypoints
- [`MIGRATION.md`](../MIGRATION.md) for user-facing upgrade notes across breaking SDK changes
- [`CHANGELOG.md`](../CHANGELOG.md) for release-by-release history
- [`crates/openrouter-cli/README.md`](../crates/openrouter-cli/README.md) for the companion CLI surface and auth/config behavior

## Contributor Guides

- [`CONTRIBUTING.md`](../CONTRIBUTING.md) for contributor workflow, local setup, and review expectations
- [`maintenance-policy.md`](maintenance-policy.md) for release policy, MSRV expectations, and breaking-change rules
- [`compatibility-update-policy.md`](compatibility-update-policy.md) for how upstream OpenRouter compatibility changes are tracked and documented
- [`SECURITY.md`](../SECURITY.md) for vulnerability reporting
- [`SUPPORT.md`](../SUPPORT.md) for issue triage and support boundaries

## Design Docs

- [`generated-core-architecture.md`](generated-core-architecture.md) for the generated-core plus idiomatic-wrapper direction
- [`http-transport-migration.md`](http-transport-migration.md) for the `surf` to `reqwest + rustls` migration plan

## Operations And Validation

- [`official-endpoint-test-matrix.md`](official-endpoint-test-matrix.md) for accepted endpoint coverage and live-test status
- [`openapi-drift-reporting.md`](openapi-drift-reporting.md) for the nightly upstream-spec drift workflow
- [`specs/openrouter/README.md`](../specs/openrouter/README.md) for baseline, source snapshot, overlays, and generator config organization
- [`tests/integration/README.md`](../tests/integration/README.md) for integration-test pools and environment switches
- [`cli-automation-workflows.md`](cli-automation-workflows.md) for copy-paste CLI shell and CI recipes

## Community And Distribution

- [`community/awesome-openrouter/README.md`](community/awesome-openrouter/README.md) for the Awesome OpenRouter submission kit and related assets

## How To Add New Docs

Use the smallest surface that matches the audience:

- put repo entrypoint and durable user-facing release docs at the root
- put maintainer workflows, policies, design notes, and operational docs in `docs/`
- put spec and generator inputs in `specs/`
- keep subsystem-specific instructions next to the subsystem they describe

Prefer linking to one source of truth instead of copying fast-changing numbers or status claims into multiple documents.
