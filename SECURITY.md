# Security Policy

## Supported Versions

This project currently supports the latest released minor line of `openrouter-rs` and `openrouter-cli`.

| Version line | Status |
| --- | --- |
| Latest released minor line of `openrouter-rs` | Supported |
| Latest released minor line of `openrouter-cli` | Supported |
| Older release lines | Upgrade first, then report if the issue still reproduces |
| Unreleased `main` | Best effort during active development |

## Reporting a Vulnerability

Please do not open a public GitHub issue for a suspected security vulnerability.

Instead:

- use GitHub's private vulnerability reporting for this repository if it is available, or
- email `morrisliu1994@outlook.com`

Please include:

- affected crate or CLI surface
- affected version(s)
- a minimal reproduction or proof of concept
- impact and expected risk
- any relevant request IDs, logs, or screenshots with secrets redacted

## Response Expectations

Security reports are handled on a best-effort basis.

The normal expectation is:

- acknowledge receipt within 5 business days
- request clarification or a reproduction if needed
- coordinate on disclosure timing when the report is valid
- publish a fix and then disclose publicly once users have a reasonable upgrade path

## Scope

This policy covers:

- `openrouter-rs`
- `openrouter-cli`
- release artifacts produced from this repository
- repository documentation or examples that would cause insecure usage if followed as written

This policy does not replace OpenRouter's own security handling for upstream platform or provider incidents unless the issue is specifically caused by this SDK/CLI repository.

## Secrets

- Never include real API keys, management keys, or tokens in reports.
- Redact request IDs, logs, and payloads as needed to avoid leaking sensitive data.
