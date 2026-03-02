#!/usr/bin/env bash
set -u -o pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_FILE="${1:-$ROOT_DIR/tests/integration/hot_models.json}"
TOP_N="${OPENROUTER_HOT_MODELS_TOP_N:-10}"
RANKINGS_URL="${OPENROUTER_RANKINGS_URL:-https://openrouter.ai/rankings}"
MODELS_URL="${OPENROUTER_MODELS_ENDPOINT:-https://openrouter.ai/api/v1/models}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

log() {
  echo "[sync_hot_models] $*"
}

if ! command -v jq >/dev/null 2>&1; then
  log "jq not found; keeping existing model pool at $OUTPUT_FILE"
  exit 0
fi

fetch_url() {
  local url="$1"
  local destination="$2"

  if [ -n "${OPENROUTER_API_KEY:-}" ]; then
    curl -fsSL --max-time 30 \
      -H "Authorization: Bearer ${OPENROUTER_API_KEY}" \
      "$url" >"$destination"
  else
    curl -fsSL --max-time 30 "$url" >"$destination"
  fi
}

canonicalize_model_url() {
  local url="$1"
  local raw="${url#https://openrouter.ai/}"
  raw="${raw%/}"

  if [[ ! "$raw" =~ ^[A-Za-z0-9._-]+/[A-Za-z0-9._:-]+$ ]]; then
    return 1
  fi

  local provider="${raw%%/*}"
  local slug="${raw#*/}"
  slug="$(echo "$slug" | sed -E 's/-20[0-9]{6}$//')"

  printf '%s/%s\n' "$provider" "$slug"
}

extract_hot_from_rankings() {
  local rankings_html="$1"

  rg -o 'https://openrouter\.ai/[A-Za-z0-9._-]+/[A-Za-z0-9._:-]+' "$rankings_html" \
    | awk '!seen[$0]++' \
    | while read -r url; do
      canonicalize_model_url "$url" || true
    done \
    | awk '!seen[$0]++' \
    | head -n "$TOP_N"
}

extract_hot_from_models_endpoint() {
  local models_json="$1"

  jq -r '.data // [] | .[] | .id // empty' "$models_json" \
    | awk '!seen[$0]++' \
    | head -n "$TOP_N"
}

extract_all_model_ids_from_models_endpoint() {
  local models_json="$1"

  jq -r '.data // [] | .[] | .id // empty' "$models_json" | awk '!seen[$0]++'
}

read_existing_string() {
  local jq_path="$1"
  if [ -f "$OUTPUT_FILE" ]; then
    jq -r "$jq_path // empty" "$OUTPUT_FILE" 2>/dev/null || true
  fi
}

read_existing_array() {
  local jq_path="$1"
  if [ -f "$OUTPUT_FILE" ]; then
    jq -r "$jq_path[]? // empty" "$OUTPUT_FILE" 2>/dev/null || true
  fi
}

mkdir -p "$(dirname "$OUTPUT_FILE")"

rankings_dump="$TMP_DIR/rankings.html"
models_dump="$TMP_DIR/models.json"
source_label="rankings"
source_endpoint="$RANKINGS_URL"
hot_models=()
rankings_candidates=()
models_fetched=false
allowlist_file="$TMP_DIR/model_allowlist.txt"

if fetch_url "$RANKINGS_URL" "$rankings_dump"; then
  while IFS= read -r model; do
    [ -n "$model" ] && rankings_candidates+=("$model")
  done < <(extract_hot_from_rankings "$rankings_dump")
fi

if [ "${#rankings_candidates[@]}" -gt 0 ]; then
  if fetch_url "$MODELS_URL" "$models_dump"; then
    models_fetched=true
    extract_all_model_ids_from_models_endpoint "$models_dump" >"$allowlist_file"

    for model in "${rankings_candidates[@]}"; do
      if rg -F -x -q "$model" "$allowlist_file"; then
        hot_models+=("$model")
      fi
    done
  fi
fi

if [ "${#hot_models[@]}" -eq 0 ]; then
  source_label="models-endpoint-fallback"
  source_endpoint="$MODELS_URL"
  if [ "$models_fetched" = false ] && fetch_url "$MODELS_URL" "$models_dump"; then
    models_fetched=true
  fi

  if [ "$models_fetched" = true ]; then
    while IFS= read -r model; do
      [ -n "$model" ] && hot_models+=("$model")
    done < <(extract_hot_from_models_endpoint "$models_dump")
  fi
fi

if [ "${#hot_models[@]}" -eq 0 ]; then
  log "Unable to refresh hot models from rankings/models endpoint; keeping existing file."
  exit 0
fi

stable_chat="$(read_existing_string '.stable.chat')"
stable_reasoning="$(read_existing_string '.stable.reasoning')"
stable_regression=()
while IFS= read -r model; do
  [ -n "$model" ] && stable_regression+=("$model")
done < <(read_existing_array '.stable.regression')

if [ -z "$stable_chat" ]; then
  stable_chat="x-ai/grok-code-fast-1"
fi
if [ -z "$stable_reasoning" ]; then
  stable_reasoning="deepseek/deepseek-r1"
fi
if [ "${#stable_regression[@]}" -eq 0 ]; then
  stable_regression=(
    "x-ai/grok-code-fast-1"
    "openai/gpt-4o-mini"
    "deepseek/deepseek-r1"
  )
fi

hot_models_json="$(printf '%s\n' "${hot_models[@]}" | jq -R . | jq -s .)"
stable_regression_json="$(printf '%s\n' "${stable_regression[@]}" | jq -R . | jq -s .)"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
tmp_output="$TMP_DIR/hot_models.json"

jq -n \
  --arg generated_at "$generated_at" \
  --arg source_type "$source_label" \
  --arg endpoint "$source_endpoint" \
  --argjson top_n "$TOP_N" \
  --arg stable_chat "$stable_chat" \
  --arg stable_reasoning "$stable_reasoning" \
  --argjson stable_regression "$stable_regression_json" \
  --argjson hot_models "$hot_models_json" \
  '{
    schema_version: 1,
    generated_at: $generated_at,
    source: {
      type: $source_type,
      endpoint: $endpoint,
      top_n: $top_n
    },
    stable: {
      chat: $stable_chat,
      reasoning: $stable_reasoning,
      regression: $stable_regression
    },
    hot: {
      models: $hot_models
    }
  }' >"$tmp_output"

mv "$tmp_output" "$OUTPUT_FILE"
log "Updated $OUTPUT_FILE using $source_label with ${#hot_models[@]} hot models."
