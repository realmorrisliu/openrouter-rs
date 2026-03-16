#!/usr/bin/env bash
set -u -o pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Compatibility wrapper for older local scripts that still reference the
# previous hot-model sync entrypoint.
exec "$ROOT_DIR/scripts/sync_hot_models.sh" "$@"
