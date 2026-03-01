# openrouter-cli

`openrouter-cli` is a workspace CLI companion for `openrouter-rs`.

## Current Scope

- OR-19: command bootstrap and config/profile resolution
- OR-20: discovery commands for models/providers/endpoints

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
