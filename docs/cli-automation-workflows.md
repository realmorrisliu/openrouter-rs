# CLI Automation Workflows

`openrouter-cli` is most useful in automation when you treat it as a JSON-emitting boundary around the SDK.

All examples below assume:

- `jq` is available
- `--output json` is set explicitly
- JSON responses use the versioned `{ "schema_version": "0.1", "data": ... }` envelope

## Verify Profile Resolution In CI

This is a cheap preflight before any real API call:

```bash
openrouter-cli --output json profile show \
  | jq '.data | {profile, base_url, api_key_present, management_key_present}'
```

Use it when you want CI logs to show which profile/base URL won without exposing secrets.

## Export Tool-Capable Models For Agent Jobs

This turns discovery output into a tab-separated inventory that can feed a matrix job or a generated config file:

```bash
openrouter-cli \
  --api-key "$OPENROUTER_API_KEY" \
  --output json \
  models list \
  --supported-parameter tools \
  | jq -r '.data[] | [.id, .context_length, .pricing.prompt] | @tsv'
```

If you only want the IDs:

```bash
openrouter-cli \
  --api-key "$OPENROUTER_API_KEY" \
  --output json \
  models list \
  --supported-parameter tools \
  | jq -r '.data[].id'
```

## Build A Daily Usage Rollup

This example aggregates per-model usage for a given UTC day and sorts the result by spend:

```bash
day="${1:-$(date -u +%F)}"

openrouter-cli \
  --management-key "$OPENROUTER_MANAGEMENT_KEY" \
  --output json \
  usage activity \
  --date "$day" \
  | jq '
      .data
      | group_by(.model)
      | map({
          model: .[0].model,
          requests: (map(.requests) | add),
          usage: (map(.usage) | add),
          prompt_tokens: (map(.prompt_tokens) | add),
          completion_tokens: (map(.completion_tokens) | add)
        })
      | sort_by(-.usage)
    '
```

That output is stable enough to upload as a CI artifact or feed into a scheduled reporting job.

## Create And Revoke An Ephemeral CI Key

Use a management key to mint a short-lived API key with a hard budget cap, then revoke it after the job:

```bash
key_json="$(
  openrouter-cli \
    --management-key "$OPENROUTER_MANAGEMENT_KEY" \
    --output json \
    keys create \
    --name "ci-${GITHUB_RUN_ID:-local}" \
    --limit 5
)"

export OPENROUTER_API_KEY="$(jq -r '.data.key' <<<"$key_json")"
export OPENROUTER_KEY_HASH="$(jq -r '.data.hash' <<<"$key_json")"

cargo run --example domain_chat_completion

openrouter-cli \
  --management-key "$OPENROUTER_MANAGEMENT_KEY" \
  --output json \
  keys delete "$OPENROUTER_KEY_HASH" \
  --yes
```

`data.key` is only returned on creation, so capture it immediately if the automation needs the raw token.
