# Support

## Where to Go

Use GitHub issues in this repository for:

- SDK bugs or regressions
- CLI bugs or regressions
- docs or example problems
- feature requests for the Rust SDK or CLI
- migration and compatibility questions specific to this repository

OpenRouter platform, account, billing, or provider-status issues may need to be handled by OpenRouter directly unless there is a clear bug in this repository's Rust surfaces.

## Before Opening an Issue

Please check:

- [`README.md`](README.md)
- [`MIGRATION.md`](MIGRATION.md)
- [`tests/integration/README.md`](tests/integration/README.md)
- [`crates/openrouter-cli/README.md`](crates/openrouter-cli/README.md)
- existing issues and recent changelog entries

## What to Include

For bug reports, include as much of the following as you can:

- crate or CLI version
- Rust version
- operating system
- endpoint or command involved
- model name, if relevant
- minimal reproduction
- expected behavior
- actual behavior
- relevant request IDs or logs with secrets removed

For CLI issues, also include:

- the exact command you ran
- whether you used flags, env vars, or profile config
- whether output mode was `table` or `json`

## Best-Effort Support

Support in this repository is best effort.

Maintainers may ask you to:

- upgrade to the latest release line
- reduce the report to a minimal reproduction
- move an upstream platform issue to the appropriate OpenRouter support channel

## Security Issues

For security-sensitive reports, do not file a public issue. Follow [`SECURITY.md`](SECURITY.md) instead.
