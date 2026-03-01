#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
README_PATH="${ROOT_DIR}/README.md"
MIGRATION_PATH="${ROOT_DIR}/MIGRATION.md"

require_pattern() {
  local pattern="$1"
  local file="$2"
  if ! rg -F --quiet "$pattern" "$file"; then
    echo "Missing required migration pattern in ${file}: ${pattern}" >&2
    exit 1
  fi
}

if [[ ! -f "$README_PATH" ]]; then
  echo "README.md not found at ${README_PATH}" >&2
  exit 1
fi

# README must keep the migration-entry section and core rename mappings.
require_pattern "### 🔁 0.6 Naming/Pagination Migration" "$README_PATH"
require_pattern "models().count()" "$README_PATH"
require_pattern "models().get_model_count()" "$README_PATH"
require_pattern "models().list_for_user()" "$README_PATH"
require_pattern "models().list_user_models()" "$README_PATH"
require_pattern "management().exchange_code_for_api_key(...)" "$README_PATH"
require_pattern "management().create_api_key_from_auth_code(...)" "$README_PATH"

# If MIGRATION.md exists (OR-25 and later), validate key structure and snippets.
if [[ -f "$MIGRATION_PATH" ]]; then
  require_pattern "# Migration Guide: 0.5.x -> 0.6.0" "$MIGRATION_PATH"
  require_pattern "## Breaking-Change Mapping" "$MIGRATION_PATH"
  require_pattern "## Top 10 Before/After Recipes" "$MIGRATION_PATH"
  require_pattern "OpenRouterClientBuilder::management_key" "$MIGRATION_PATH"
  require_pattern "api::legacy::completion" "$MIGRATION_PATH"
  require_pattern "client.legacy().completions().create" "$MIGRATION_PATH"

  recipe_count="$(rg -n "^### [0-9]+\\)" "$MIGRATION_PATH" | wc -l | tr -d ' ')"
  if [[ "$recipe_count" -ne 10 ]]; then
    echo "Expected 10 numbered migration recipes in MIGRATION.md, found ${recipe_count}" >&2
    exit 1
  fi
fi

echo "Migration documentation checks passed."
