# Changelog

Releases are tagged `openrouter-cli-v<version>`. The SDK it builds on has its own
[changelog](../../CHANGELOG.md).

## [0.3.0] - 2026-11-03

Rebuilt on `openrouter-rs` 0.24.0. The previous release pinned `^0.9.0`, so the
CLI had been shipping a transport and decode path from six months earlier.

The command surface is unchanged: same commands, flags and exit codes as 0.2.0.
What changes is what sits underneath them:

- JSON output follows the current SDK response types, so fields track the
  0.9 -> 0.24 schema changes. Anything parsing that output should be re-checked.
- Streaming, chat, Messages and Responses decoding come from the current SDK
  rather than the 0.9 branch.

## [0.2.0] - 2026-04-28

Published before this file existed; see the tag for details.
