# Integration Test Model Pools

The live integration suite resolves models in this order:

1. explicit environment overrides
2. `tests/integration/hot_models.json`
3. built-in defaults from the test helpers

## Required Environment Variables

- `OPENROUTER_API_KEY`: required for any live integration call
- `OPENROUTER_MANAGEMENT_KEY`: required for management smoke and management contract checks

## Optional Environment Variables

- `OPENROUTER_INTEGRATION_TIER`: `stable` (default) or `hot`
- `OPENROUTER_TEST_MODEL_POOL_FILE`: custom model-pool JSON file
- `OPENROUTER_TEST_CHAT_MODEL`: override the primary chat model
- `OPENROUTER_TEST_EMBEDDINGS_MODEL`: override the primary embeddings model
- `OPENROUTER_TEST_MESSAGES_MODEL`: override the primary Messages API model
- `OPENROUTER_TEST_RESPONSES_MODEL`: override the primary Responses API model
- `OPENROUTER_TEST_REASONING_MODEL`: override the primary reasoning model
- `OPENROUTER_TEST_STABLE_MODELS`: comma-separated stable regression models
- `OPENROUTER_TEST_HOT_MODELS`: comma-separated hot-sweep models
- `OPENROUTER_TEST_HOT_MODELS_LIMIT`: max models to run in the hot sweep
- `OPENROUTER_RUN_MANAGEMENT_TESTS`: set to `1`/`true` to enable write-path management smoke

For local development, start from [`.env.example`](../../.env.example).

## Preferred Commands

Use the repo `just` recipes where possible:

```bash
just test-integration
just test-integration-subsets
just test-live-contract
OPENROUTER_MANAGEMENT_KEY=... just test-live-contract-management
```

Direct cargo equivalents:

```bash
OPENROUTER_API_KEY=... cargo test --test integration -- --nocapture
OPENROUTER_API_KEY=... cargo test --test integration contract:: -- --nocapture
OPENROUTER_MANAGEMENT_KEY=... OPENROUTER_RUN_MANAGEMENT_TESTS=1 cargo test --test integration management:: -- --nocapture
```

## Stable vs Hot Tiers

- `stable`: the default regression tier used for predictable local and CI coverage
- `hot`: a broader Responses API sweep for recently popular or recently changed upstream models

Examples:

```bash
# Default stable tier
OPENROUTER_API_KEY=... cargo test --test integration -- --nocapture

# Hot-model sweep
OPENROUTER_API_KEY=... OPENROUTER_INTEGRATION_TIER=hot cargo test --test integration -- --nocapture
```

## Management Smoke Tests

Management smoke is opt-in and performs cleanup-protected lifecycle checks for:

- `keys`: create -> list/get -> update -> delete
- `guardrails`: create -> list/get -> update -> delete

Example:

```bash
OPENROUTER_MANAGEMENT_KEY=... \
OPENROUTER_RUN_MANAGEMENT_TESTS=1 \
cargo test --test integration management:: -- --nocapture
```

## Contract Checks

Use the narrower contract suite for release validation or upstream regression checks:

```bash
just test-live-contract
```

For management-key contract smoke:

```bash
OPENROUTER_MANAGEMENT_KEY=... just test-live-contract-management
```

## Pool Refresh

`scripts/sync_hot_models.sh` updates `tests/integration/hot_models.json` by:

1. trying to derive top models from `https://openrouter.ai/rankings`
2. falling back to ordered IDs from `https://openrouter.ai/api/v1/models`
3. validating candidate hot models with a minimal Responses API request when `OPENROUTER_API_KEY` is available
4. keeping the last known file if remote fetch/parsing fails

Example:

```bash
./scripts/sync_hot_models.sh
```
