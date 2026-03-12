# openrouter-cli

`openrouter-cli` is the workspace CLI companion for `openrouter-rs`.

It currently focuses on four areas:

- profile/config resolution
- model and provider discovery
- API-key and guardrail management
- credits, billing, and usage activity

The implementation lives in [`crates/openrouter-cli/src`](./src), and the crate currently publishes as `0.1.1`.

## Install

From crates.io:

```bash
cargo install openrouter-cli
```

From this workspace:

```bash
cargo install --path crates/openrouter-cli --locked
```

Prebuilt GitHub release archives follow the `openrouter-cli-v<version>` tag naming:

```bash
VERSION=0.1.1
curl -L -o openrouter-cli.tar.gz \
  "https://github.com/realmorrisliu/openrouter-rs/releases/download/openrouter-cli-v${VERSION}/openrouter-cli-${VERSION}-x86_64-unknown-linux-gnu.tar.gz"
curl -L -o SHA256SUMS \
  "https://github.com/realmorrisliu/openrouter-rs/releases/download/openrouter-cli-v${VERSION}/SHA256SUMS"
grep "openrouter-cli-${VERSION}-x86_64-unknown-linux-gnu.tar.gz" SHA256SUMS | sha256sum -c -
tar -xzf openrouter-cli.tar.gz
./openrouter-cli --help
```

For macOS and Windows, use the matching artifact from the same release tag.

## Command Surface

```text
profile show
config show|path
models list|show|endpoints
providers list
credits show|charge
keys list|create|get|update|delete
guardrails list|create|get|update|delete
guardrails assignments keys list|assign|unassign
guardrails assignments members list|assign|unassign
usage activity
```

Auth expectations by command group:

- `models`, `providers`, `credits show`, `credits charge`: API key
- `keys`, `guardrails`, `usage activity`: management key
- `profile`, `config`: no API call required

## Config And Resolution Order

Default config path:

- `$XDG_CONFIG_HOME/openrouter/profiles.toml`, or
- `$HOME/.config/openrouter/profiles.toml`

Overrides:

- `--config <path>`
- `OPENROUTER_CLI_CONFIG`

Profile config format:

```toml
default_profile = "default"

[profiles.default]
api_key = "sk-or-v1-..."
management_key = "or-mgmt-..."
base_url = "https://openrouter.ai/api/v1"
```

Resolution order:

1. Flags: `--api-key`, `--management-key`, `--base-url`
2. Environment: `OPENROUTER_API_KEY`, `OPENROUTER_MANAGEMENT_KEY`, `OPENROUTER_BASE_URL`
3. Profile values from the selected config profile
4. Default base URL: `https://openrouter.ai/api/v1`

Profile selection order:

1. `--profile`
2. `OPENROUTER_PROFILE`
3. `default_profile` in config
4. `"default"`

Useful inspection commands:

```bash
openrouter-cli profile show
openrouter-cli config show
openrouter-cli config path
openrouter-cli --output json profile show
```

## Discovery Workflows

Examples:

```bash
# List models
openrouter-cli --api-key "$OPENROUTER_API_KEY" models list

# Filter by category
openrouter-cli --api-key "$OPENROUTER_API_KEY" models list --category programming

# Filter by supported parameter
openrouter-cli --api-key "$OPENROUTER_API_KEY" models list --supported-parameter tools

# Show one model
openrouter-cli --api-key "$OPENROUTER_API_KEY" models show openai/gpt-4.1

# Show endpoints for one model
openrouter-cli --api-key "$OPENROUTER_API_KEY" models endpoints openai/gpt-4.1

# List providers
openrouter-cli --api-key "$OPENROUTER_API_KEY" providers list
```

`models list` accepts either `--category` or `--supported-parameter`, not both.

## Management Workflows

### API keys

```bash
# List keys
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" keys list --include-disabled

# Create one
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" keys create --name "ci-bot" --limit 100

# Inspect one
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" keys get sk-or-v1-hash

# Update one
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" keys update sk-or-v1-hash --disable

# Delete one
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" keys delete sk-or-v1-hash --yes
```

### Guardrails

```bash
# List guardrails
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" guardrails list --limit 20

# Create one
openrouter-cli \
  --management-key "$OPENROUTER_MANAGEMENT_KEY" \
  guardrails create \
  --name "ci-budget-cap" \
  --limit-usd 25 \
  --enforce-zdr

# Update and clear allowlists
openrouter-cli \
  --management-key "$OPENROUTER_MANAGEMENT_KEY" \
  guardrails update gr_123 \
  --clear-allowed-providers \
  --clear-allowed-models

# Delete one
openrouter-cli --management-key "$OPENROUTER_MANAGEMENT_KEY" guardrails delete gr_123 --yes
```

### Guardrail assignments

```bash
# List global key assignments
openrouter-cli \
  --management-key "$OPENROUTER_MANAGEMENT_KEY" \
  guardrails assignments keys list

# List assignments for one guardrail
openrouter-cli \
  --management-key "$OPENROUTER_MANAGEMENT_KEY" \
  guardrails assignments keys list --guardrail-id gr_123

# Assign keys
openrouter-cli \
  --management-key "$OPENROUTER_MANAGEMENT_KEY" \
  guardrails assignments keys assign gr_123 key_a key_b

# Unassign keys
openrouter-cli \
  --management-key "$OPENROUTER_MANAGEMENT_KEY" \
  guardrails assignments keys unassign gr_123 key_a key_b --yes
```

Member assignment commands mirror the same shape under `guardrails assignments members ...`.

## Credits And Usage

```bash
# Show purchased/used credits
openrouter-cli --api-key "$OPENROUTER_API_KEY" credits show

# Create a Coinbase charge
openrouter-cli \
  --api-key "$OPENROUTER_API_KEY" \
  credits charge \
  --amount 25 \
  --sender 0xYourWalletAddress \
  --chain-id 1

# Show activity for a specific day
openrouter-cli \
  --management-key "$OPENROUTER_MANAGEMENT_KEY" \
  usage activity \
  --date 2026-03-01
```

## Output Contract

`--output` supports:

- `table` (default)
- `json`

JSON output is wrapped in a versioned envelope:

```json
{
  "schema_version": "0.1",
  "data": { "...": "..." }
}
```

Errors in JSON mode use:

```json
{
  "schema_version": "0.1",
  "error": {
    "code": "cli_error",
    "message": "..."
  }
}
```

## Live Smoke Tests

The workspace includes an opt-in live smoke suite at `tests/live_smoke.rs`.

Environment switches:

- `OPENROUTER_CLI_RUN_LIVE=1`: enable live smoke
- `OPENROUTER_CLI_RUN_LIVE_WRITE=1`: also enable create/delete write-path checks

Required secrets:

- `OPENROUTER_API_KEY` for read smoke
- `OPENROUTER_MANAGEMENT_KEY` for usage and write smoke

Examples:

```bash
# Read-only live smoke
OPENROUTER_CLI_RUN_LIVE=1 \
OPENROUTER_API_KEY=... \
cargo test -p openrouter-cli --test live_smoke -- --nocapture --test-threads=1

# Include write-path smoke
OPENROUTER_CLI_RUN_LIVE=1 \
OPENROUTER_CLI_RUN_LIVE_WRITE=1 \
OPENROUTER_API_KEY=... \
OPENROUTER_MANAGEMENT_KEY=... \
cargo test -p openrouter-cli --test live_smoke -- --nocapture --test-threads=1
```
