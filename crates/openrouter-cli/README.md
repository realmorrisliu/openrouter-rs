# openrouter-cli

`openrouter-cli` is a workspace CLI companion for `openrouter-rs`.

## Current Scope

- OR-19: command bootstrap and config/profile resolution
- OR-20: discovery commands for models/providers/endpoints
- OR-21: management commands for API keys and guardrails
- OR-22: usage and billing commands with stable output contracts

## Config And Profile Convention

By default, config is loaded from:

- `$XDG_CONFIG_HOME/openrouter/profiles.toml`, or
- `$HOME/.config/openrouter/profiles.toml`

You can override with:

- `--config <path>`
- `OPENROUTER_CLI_CONFIG`

Config format:

```toml
default_profile = "default"

[profiles.default]
api_key = "sk-or-v1-..."
management_key = "or-mgmt-..."
base_url = "https://openrouter.ai/api/v1"
```

## Resolution Priority

For `api_key`, `management_key`, and `base_url`:

1. CLI flags (`--api-key`, `--management-key`, `--base-url`)
2. Environment (`OPENROUTER_API_KEY`, `OPENROUTER_MANAGEMENT_KEY`, `OPENROUTER_BASE_URL`)
3. Active profile values from config file
4. Defaults (for `base_url`: `https://openrouter.ai/api/v1`)

For profile selection:

1. `--profile`
2. `OPENROUTER_PROFILE`
3. `default_profile` in config
4. `"default"`

## Discovery Commands (OR-20)

`openrouter-cli` now supports discovery workflows:

- `models list` with optional filters:
  - `--category`
  - `--supported-parameter`
- `models show <model_id>`
- `models endpoints <model_id>`
- `providers list`

Examples:

```bash
# List models in a category
openrouter-cli --api-key "$OPENROUTER_API_KEY" models list --category programming

# List models supporting tool-calling parameter
openrouter-cli --api-key "$OPENROUTER_API_KEY" models list --supported-parameter tools

# Show one model
openrouter-cli --api-key "$OPENROUTER_API_KEY" models show openai/gpt-4.1

# Show endpoints for one model
openrouter-cli --api-key "$OPENROUTER_API_KEY" models endpoints openai/gpt-4.1

# List providers
openrouter-cli --api-key "$OPENROUTER_API_KEY" providers list
```

## Management Commands (OR-21)

`openrouter-cli` supports management workflows:

- `keys list|create|get|update|delete`
- `guardrails list|create|get|update|delete`
- `guardrails assignments keys list|assign|unassign`
- `guardrails assignments members list|assign|unassign`

Examples:

```bash
# List keys
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" keys list --include-disabled

# Create and update key
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" keys create --name "ci-bot" --limit 100
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" keys update sk-or-v1-hash --disable

# Delete key (requires explicit confirmation)
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" keys delete sk-or-v1-hash --yes

# Update guardrail and clear allowlists
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" guardrails update gr_123 --clear-allowed-providers --clear-allowed-models
```

## Usage And Billing Commands (OR-22)

Supported command groups:

- `credits show`
- `credits charge --amount --sender --chain-id`
- `usage activity [--date YYYY-MM-DD]`

Examples:

```bash
# Show total credits and usage
openrouter-cli --api-key "$OPENROUTER_API_KEY" credits show

# Create Coinbase charge
openrouter-cli \
  --api-key "$OPENROUTER_API_KEY" \
  credits charge \
  --amount 25 \
  --sender 0xYourWalletAddress \
  --chain-id 1

# Query usage activity for a specific day (requires management key)
openrouter-cli \
  --management-key "$OPENROUTER_MANAGEMENT_KEY" \
  usage activity \
  --date 2026-02-28
```

## Output Contract

`--output` supports:

- `table` (default)
- `json`

JSON output is wrapped with schema metadata for automation stability:

```json
{
  "schema_version": "0.1",
  "data": { "...": "..." }
}
```

Error output in JSON mode follows:

```json
{
  "schema_version": "0.1",
  "error": {
    "code": "cli_error",
    "message": "..."
  }
}
```
