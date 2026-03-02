# Official Endpoint Test Matrix

Snapshot date: 2026-03-02  
Source of truth: `https://openrouter.ai/openapi.json` (method+path extracted from latest spec)

## Coverage Summary

- Official OpenAPI endpoints: `36` method+path entries.
- SDK implementation coverage (`src/api` + domain client): `36 / 36` (`100%`).
- Live integration coverage (`tests/integration`): `12 / 36` endpoints currently exercised.
  - Covered live now: `POST /chat/completions`, `POST /messages`, `POST /responses`, `POST /embeddings`, `GET /key`, `GET /models`, `GET /models/user`, `GET /models/count`, `GET /models/{author}/{slug}/endpoints`, `GET /providers`, `GET /endpoints/zdr`, `GET /embeddings/models`

Legend:

- `SDK`: endpoint implemented in `openrouter-rs`.
- `Unit`: unit coverage depth.
  - `Path` = test asserts HTTP method/path (often with header/body checks).
  - `Contract` = serde/request-shape/parser coverage only.
  - `None` = no direct unit coverage found.
- `Live`: real OpenRouter API integration coverage.
- `Priority`: recommended order for adding/improving live coverage.

## Endpoint Matrix

| Official endpoint | SDK surface | SDK | Unit | Live | Priority |
| --- | --- | --- | --- | --- | --- |
| `GET /activity` | `client.management().get_activity(...)` | Yes | Path | No | P1 |
| `POST /auth/keys` | `client.management().create_api_key_from_auth_code(...)` | Yes | Path | No | P2 |
| `POST /auth/keys/code` | `client.management().create_auth_code(...)` | Yes | Path | No | P2 |
| `POST /chat/completions` | `client.chat().create(...)` / `client.chat().stream(...)` | Yes | Contract | Yes | Keep |
| `GET /credits` | `client.get_credits()` / `client.management().get_credits()` | Yes | None | No | P2 |
| `POST /credits/coinbase` | `client.create_coinbase_charge(...)` / `client.management().create_coinbase_charge(...)` | Yes | None | No | P2 |
| `POST /embeddings` | `client.create_embedding(...)` / `client.models().create_embedding(...)` | Yes | Contract | Yes | Keep |
| `GET /embeddings/models` | `client.list_embedding_models()` / `client.models().list_embedding_models()` | Yes | None | Yes | Keep |
| `GET /endpoints/zdr` | `client.models().list_zdr_endpoints(...)` | Yes | Contract | Yes | Keep |
| `GET /generation` | `client.get_generation(...)` / `client.management().get_generation(...)` | Yes | None | No | P2 |
| `GET /guardrails` | `client.management().list_guardrails(...)` | Yes | Path | No | P1 |
| `POST /guardrails` | `client.management().create_guardrail(...)` | Yes | Contract | No | P1 |
| `GET /guardrails/{id}` | `client.management().get_guardrail(...)` | Yes | Contract | No | P1 |
| `PATCH /guardrails/{id}` | `client.management().update_guardrail(...)` | Yes | Contract | No | P1 |
| `DELETE /guardrails/{id}` | `client.management().delete_guardrail(...)` | Yes | Path | No | P1 |
| `GET /guardrails/{id}/assignments/keys` | `client.management().list_guardrail_key_assignments(...)` | Yes | Contract | No | P1 |
| `POST /guardrails/{id}/assignments/keys` | `client.management().create_guardrail_key_assignments(...)` | Yes | Path | No | P1 |
| `POST /guardrails/{id}/assignments/keys/remove` | `client.management().delete_guardrail_key_assignments(...)` | Yes | None | No | P1 |
| `GET /guardrails/{id}/assignments/members` | `client.management().list_guardrail_member_assignments(...)` | Yes | Contract | No | P1 |
| `POST /guardrails/{id}/assignments/members` | `client.management().create_guardrail_member_assignments(...)` | Yes | None | No | P1 |
| `POST /guardrails/{id}/assignments/members/remove` | `client.management().delete_guardrail_member_assignments(...)` | Yes | None | No | P1 |
| `GET /guardrails/assignments/keys` | `client.management().list_key_assignments(...)` | Yes | None | No | P1 |
| `GET /guardrails/assignments/members` | `client.management().list_member_assignments(...)` | Yes | Path | No | P1 |
| `GET /key` | `client.get_current_api_key_info()` / `client.management().get_current_api_key_info()` | Yes | Contract | Yes | Keep |
| `GET /keys` | `client.list_api_keys(...)` / `client.management().list_api_keys(...)` | Yes | Path | No | P1 |
| `POST /keys` | `client.create_api_key(...)` / `client.management().create_api_key(...)` | Yes | None | No | P1 |
| `GET /keys/{hash}` | `client.get_api_key(...)` / `client.management().get_api_key(...)` | Yes | None | No | P1 |
| `PATCH /keys/{hash}` | `client.update_api_key(...)` / `client.management().update_api_key(...)` | Yes | None | No | P1 |
| `DELETE /keys/{hash}` | `client.delete_api_key(...)` / `client.management().delete_api_key(...)` | Yes | Path | No | P1 |
| `GET /models` | `client.list_models()` / `client.models().list()` | Yes | Contract | Yes | Keep |
| `GET /models/{author}/{slug}/endpoints` | `client.list_model_endpoints(...)` / `client.models().list_endpoints(...)` | Yes | None | Yes | Keep |
| `GET /models/count` | `client.count_models()` / `client.models().get_model_count()` | Yes | Contract | Yes | Keep |
| `GET /models/user` | `client.list_models_for_user()` / `client.models().list_user_models()` | Yes | Path | Yes | Keep |
| `GET /providers` | `client.list_providers()` / `client.models().list_providers()` | Yes | Contract | Yes | Keep |
| `POST /messages` | `client.messages().create(...)` / `client.messages().stream(...)` | Yes | Path | Yes | Keep |
| `POST /responses` | `client.responses().create(...)` / `client.responses().stream(...)` | Yes | Contract | Yes | Keep |

## Supplemental (Legacy)

The endpoint below is intentionally kept as legacy compatibility and is not part of current OpenAPI:

| Endpoint | SDK surface | Notes |
| --- | --- | --- |
| `POST /completions` | `client.legacy().completions().create(...)` (feature `legacy-completions`) | Migration-only surface toward `chat`/`responses` |

## Incremental Test Plan

1. P1: add management-key live suite for guardrails and keys in a dedicated workflow gate (manual + weekly, no PR auto-trigger).
2. P2: keep `/credits`, `/credits/coinbase`, `/generation`, `/auth/keys*` as controlled scenarios (manual or mocked contract-first) due cost/side effects.

## Reproduce Snapshot

```bash
curl -L 'https://openrouter.ai/openapi.json' -o /tmp/openrouter-openapi.json
jq -r '.paths | to_entries[] | .key as $p | (.value | keys[] | select(. != "parameters")) as $m | "\($m|ascii_upcase) \($p)"' /tmp/openrouter-openapi.json | sort
```
