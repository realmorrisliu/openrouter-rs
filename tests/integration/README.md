# Integration Test Model Pools

The integration suite loads model selection in this order:

1. Environment variable overrides
2. `tests/integration/hot_models.json`
3. Built-in defaults

## Required environment variables

- `OPENROUTER_API_KEY`: required for any live integration API call.

## Optional environment variables

- `OPENROUTER_INTEGRATION_TIER`: `stable` (default) or `hot`.
- `OPENROUTER_TEST_MODEL_POOL_FILE`: path to a model-pool JSON file.
- `OPENROUTER_TEST_CHAT_MODEL`: force the primary chat model.
- `OPENROUTER_TEST_REASONING_MODEL`: force the primary reasoning model.
- `OPENROUTER_TEST_STABLE_MODELS`: comma-separated stable regression model list.
- `OPENROUTER_TEST_HOT_MODELS`: comma-separated hot-model sweep list.
- `OPENROUTER_TEST_HOT_MODELS_LIMIT`: max models to run in hot sweep.

## Pool refresh

`scripts/sync_hot_models.sh` updates `tests/integration/hot_models.json` by:

1. Trying to derive top models from `https://openrouter.ai/rankings`
2. Falling back to ordered IDs from `https://openrouter.ai/api/v1/models`
3. Keeping the last known file when remote fetch/parsing fails

Example:

```bash
./scripts/sync_hot_models.sh
```
