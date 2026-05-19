# Official Endpoint Test Matrix

Snapshot date: 2026-05-19
Source of truth: `https://openrouter.ai/openapi.json` (method+path extracted from latest spec)  
Tracked baseline: `specs/openrouter/openapi-baseline.json`  
Weekly drift workflow: `.github/workflows/openapi-drift.yml`

## Coverage Summary

- Official OpenAPI endpoints: `62` method+path entries.
- SDK implementation coverage (`src/api` + domain client): `62 / 62` (`100.0%`).
- Live integration coverage (`tests/integration`): `24 / 62` endpoints currently exercised.
  - Covered live now: `POST /chat/completions`, `POST /messages`, `POST /responses`, `POST /embeddings`, `POST /rerank`, `GET /key`, `GET /models`, `GET /models/user`, `GET /models/count`, `GET /models/{author}/{slug}/endpoints`, `GET /providers`, `GET /endpoints/zdr`, `GET /embeddings/models`, `GET /keys`, `POST /keys`, `GET /keys/{hash}`, `PATCH /keys/{hash}`, `DELETE /keys/{hash}`, `GET /guardrails`, `POST /guardrails`, `GET /guardrails/{id}`, `PATCH /guardrails/{id}`, `DELETE /guardrails/{id}`, `GET /organization/members`

Drift review note:

- Official audio speech routing is now `POST /audio/speech`. The SDK keeps the canonical `client.audio().speech().create(...)` surface and retries legacy `POST /tts` only as a compatibility fallback.
- Upstream added `GET /generation/content`, now exposed as `client.get_generation_content(...)` / `client.management().get_generation_content(...)`.
- Upstream added official workspace-management endpoints and workspace-aware management fields. The SDK and CLI now expose them; live management-key validation remains pending.
- Upstream now exposes request metadata through generator globals (`HTTP-Referer`, `X-Title`) instead of repeating path-level metadata parameters. The SDK already applies `HTTP-Referer`, `X-Title`, `X-OpenRouter-Title`, and optional `X-OpenRouter-Categories` through the client builder.
- Upstream added `num_fetches` to generation metadata responses. The typed `GenerationData` surface now deserializes it.
- Upstream refreshed dynamic taxonomy details (`OutputModality` now uses `speech` instead of `tts`, provider lists now include `Nex AGI`, and Responses result nullable annotations were narrowed). The SDK already carries those surfaces through flexible `String`, `Value`, `HashMap`, and `Option` fields, so no public API migration is required.
- Upstream added `response_cache_source_id` to generation metadata, workspace I/O logging key filters and sampling rate fields, and `callback_url` for video generation requests. The SDK now exposes those typed fields, and the 2026-04-28 baseline refresh keeps the accepted endpoint snapshot at `51 / 51`.
- Upstream refreshed web-search, provider, and Responses output schemas. Existing OpenRouter plugin passthrough, Anthropic hosted-tool extras, provider option maps, and Responses `Value` payloads carry those schema details without a public API migration.
- Upstream added `stt` as a generation origin and chat-completion usage cost metadata. `GenerationData::origin` remains a flexible `String`, and `ResponseUsage` now exposes typed `cost`, `cost_details`, and `is_byok` fields. The 2026-04-29 baseline refresh keeps the accepted endpoint snapshot at `51 / 51`.
- Upstream added `POST /audio/transcriptions`. The SDK now exposes a typed transcription surface, and the 2026-05-03 baseline refresh keeps the accepted endpoint snapshot at `52 / 52`.
- Upstream added experimental response metadata headers for chat completions, Responses API, and Anthropic-compatible Messages. The SDK exposes this through `OpenRouterExperimentalMetadata` on the request builders and parses chat `openrouter_metadata` / `service_tier` response fields.
- Upstream expanded guardrails with built-in content filters, custom regex filters, and provider-specific ZDR flags. The SDK now exposes those typed fields on guardrail create/update requests and guardrail responses.
- Upstream added model `supported_voices` and generation `service_tier` metadata. The SDK now exposes those typed fields, and the 2026-05-15 baseline refresh keeps the accepted endpoint snapshot at `52 / 52`.
- Upstream added BYOK provider credential management and observability destination management. The SDK now exposes those typed management-key surfaces, and the 2026-05-19 baseline refresh restores the accepted endpoint snapshot to `62 / 62`.
- Upstream expanded embeddings multimodal content with audio, video, and file content parts. The SDK now exposes typed `EmbeddingContentPart` constructors for those media inputs.
- Upstream refreshed Responses schemas including image detail `original` and response status shapes. The SDK already carries these through flexible `Value` and `String` fields, so no public API migration is required.

Legend:

- `SDK`: endpoint implemented in `openrouter-rs`.
- Canonical surface note: docs and examples prefer domain clients (`chat()`, `responses()`, `messages()`, `rerank()`, `audio().speech()`, `audio().transcriptions()`, `videos()`, `models()`, `management()`). Some rows still mention retained flat `OpenRouterClient::*` wrappers when they exist.
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
| `GET /byok` | `client.management().list_byok_keys(...)` | Yes | Path | No | P1 |
| `POST /byok` | `client.management().create_byok_key(...)` | Yes | Path | No | P1 |
| `GET /byok/{id}` | `client.management().get_byok_key(...)` | Yes | Path | No | P1 |
| `PATCH /byok/{id}` | `client.management().update_byok_key(...)` | Yes | Path | No | P1 |
| `DELETE /byok/{id}` | `client.management().delete_byok_key(...)` | Yes | Path | No | P1 |
| `POST /chat/completions` | `client.chat().create(...)` / `client.chat().stream(...)` | Yes | Contract | Yes | Keep |
| `GET /credits` | `client.get_credits()` / `client.management().get_credits()` | Yes | Path | No | P2 |
| `POST /credits/coinbase` | `client.create_coinbase_charge(...)` / `client.management().create_coinbase_charge(...)` | Yes | Path | No | P2 |
| `POST /embeddings` | `client.create_embedding(...)` / `client.models().create_embedding(...)` | Yes | Contract | Yes | Keep |
| `GET /embeddings/models` | `client.list_embedding_models()` / `client.models().list_embedding_models()` | Yes | Path | Yes | Keep |
| `GET /endpoints/zdr` | `client.models().list_zdr_endpoints(...)` | Yes | Contract | Yes | Keep |
| `GET /generation` | `client.get_generation(...)` / `client.management().get_generation(...)` | Yes | Path | No | P2 |
| `GET /generation/content` | `client.get_generation_content(...)` / `client.management().get_generation_content(...)` | Yes | Path | No | P2 |
| `GET /guardrails` | `client.management().list_guardrails(...)` / `client.management().list_guardrails_in_workspace(...)` | Yes | Path | Yes | Keep |
| `POST /guardrails` | `client.management().create_guardrail(...)` | Yes | Contract | Yes | Keep |
| `GET /guardrails/{id}` | `client.management().get_guardrail(...)` | Yes | Contract | Yes | Keep |
| `PATCH /guardrails/{id}` | `client.management().update_guardrail(...)` | Yes | Contract | Yes | Keep |
| `DELETE /guardrails/{id}` | `client.management().delete_guardrail(...)` | Yes | Path | Yes | Keep |
| `GET /guardrails/{id}/assignments/keys` | `client.management().list_guardrail_key_assignments(...)` | Yes | Contract | No | P1 |
| `POST /guardrails/{id}/assignments/keys` | `client.management().create_guardrail_key_assignments(...)` | Yes | Path | No | P1 |
| `POST /guardrails/{id}/assignments/keys/remove` | `client.management().delete_guardrail_key_assignments(...)` | Yes | Path | No | P1 |
| `GET /guardrails/{id}/assignments/members` | `client.management().list_guardrail_member_assignments(...)` | Yes | Contract | No | P1 |
| `POST /guardrails/{id}/assignments/members` | `client.management().create_guardrail_member_assignments(...)` | Yes | Path | No | P1 |
| `POST /guardrails/{id}/assignments/members/remove` | `client.management().delete_guardrail_member_assignments(...)` | Yes | Path | No | P1 |
| `GET /guardrails/assignments/keys` | `client.management().list_key_assignments(...)` | Yes | Path | No | P1 |
| `GET /guardrails/assignments/members` | `client.management().list_member_assignments(...)` | Yes | Path | No | P1 |
| `GET /key` | `client.get_current_api_key_info()` / `client.management().get_current_api_key_info()` | Yes | Contract | Yes | Keep |
| `GET /keys` | `client.management().list_api_keys(...)` / `client.management().list_api_keys_in_workspace(...)` | Yes | Path | Yes | Keep |
| `POST /keys` | `client.create_api_key(...)` / `client.create_api_key_in_workspace(...)` / `client.management().create_api_key(...)` / `client.management().create_api_key_in_workspace(...)` | Yes | Path | Yes | Keep |
| `GET /keys/{hash}` | `client.get_api_key(...)` / `client.management().get_api_key(...)` | Yes | Path | Yes | Keep |
| `PATCH /keys/{hash}` | `client.update_api_key(...)` / `client.management().update_api_key(...)` | Yes | Path | Yes | Keep |
| `DELETE /keys/{hash}` | `client.delete_api_key(...)` / `client.management().delete_api_key(...)` | Yes | Path | Yes | Keep |
| `GET /models` | `client.list_models()` / `client.models().list()` | Yes | Contract | Yes | Keep |
| `GET /models/{author}/{slug}/endpoints` | `client.list_model_endpoints(...)` / `client.models().list_endpoints(...)` | Yes | Path | Yes | Keep |
| `GET /models/count` | `client.count_models()` / `client.models().get_model_count()` | Yes | Contract | Yes | Keep |
| `GET /models/user` | `client.list_models_for_user()` / `client.models().list_user_models()` | Yes | Path | Yes | Keep |
| `GET /observability/destinations` | `client.management().list_observability_destinations(...)` | Yes | Path | No | P1 |
| `POST /observability/destinations` | `client.management().create_observability_destination(...)` | Yes | Path | No | P1 |
| `GET /observability/destinations/{id}` | `client.management().get_observability_destination(...)` | Yes | Path | No | P1 |
| `PATCH /observability/destinations/{id}` | `client.management().update_observability_destination(...)` | Yes | Path | No | P1 |
| `DELETE /observability/destinations/{id}` | `client.management().delete_observability_destination(...)` | Yes | Path | No | P1 |
| `GET /organization/members` | `client.management().list_organization_members(...)` | Yes | Path | Yes | Keep |
| `GET /providers` | `client.list_providers()` / `client.models().list_providers()` | Yes | Contract | Yes | Keep |
| `GET /workspaces` | `client.management().list_workspaces(...)` | Yes | Path | No | P1 |
| `GET /workspaces/{id}` | `client.management().get_workspace(...)` | Yes | Path | No | P1 |
| `POST /messages` | `client.messages().create(...)` / `client.messages().stream(...)` | Yes | Path | Yes | Keep |
| `POST /rerank` | `client.rerank().create(...)` | Yes | Path | Yes | Keep |
| `POST /responses` | `client.responses().create(...)` / `client.responses().stream(...)` | Yes | Contract | Yes | Keep |
| `POST /audio/speech` | `client.audio().speech().create(...)` | Yes | Path | No | P1 |
| `POST /audio/transcriptions` | `client.audio().transcriptions().create(...)` | Yes | Path | No | P1 |
| `POST /videos` | `client.videos().create(...)` | Yes | Path | No | P2 |
| `POST /workspaces` | `client.management().create_workspace(...)` | Yes | Path | No | P1 |
| `POST /workspaces/{id}/members/add` | `client.management().add_workspace_members(...)` | Yes | Path | No | P1 |
| `POST /workspaces/{id}/members/remove` | `client.management().remove_workspace_members(...)` | Yes | Path | No | P1 |
| `GET /videos/models` | `client.videos().list_models()` | Yes | Path | No | P2 |
| `GET /videos/{jobId}` | `client.videos().get_generation(...)` | Yes | Path | No | P2 |
| `GET /videos/{jobId}/content` | `client.videos().get_content(...)` | Yes | Path | No | P2 |
| `PATCH /workspaces/{id}` | `client.management().update_workspace(...)` | Yes | Path | No | P1 |
| `DELETE /workspaces/{id}` | `client.management().delete_workspace(...)` | Yes | Path | No | P1 |

## Supplemental (Legacy)

The endpoint below is intentionally kept as legacy compatibility and is not part of current OpenAPI:

| Endpoint | SDK surface | Notes |
| --- | --- | --- |
| `POST /completions` | `client.legacy().completions().create(...)` (feature `legacy-completions`) | Migration-only surface toward `chat`/`responses` |
| `POST /tts` | `client.audio().speech().create(...)` | Compatibility fallback while upstream rolls out `POST /audio/speech` everywhere |

## Incremental Test Plan

1. P1: add management-key live coverage for assignment endpoints (`/guardrails/*/assignments/*` and `/guardrails/assignments/*`).
2. P1: add management-key live smoke coverage for `/activity`.
3. P2: keep `/credits`, `/credits/coinbase`, `/generation`, `/generation/content`, and `/auth/keys*` as controlled scenarios (manual or mocked contract-first) due cost/side effects.
4. P1/P2: add low-cost live or smoke coverage for `/audio/speech`, `/audio/transcriptions`, and `/videos*` once stable fixtures and cost controls are defined.
5. P1: add management-key live validation for `/workspaces*`, plus targeted workspace-scoped read/write coverage for `/keys` and `/guardrails`.
6. P1: add management-key live validation for `/byok*` and `/observability/destinations*` once safe test credentials and destination fixtures are defined.

## Reproduce Snapshot

```bash
curl -L 'https://openrouter.ai/openapi.json' -o /tmp/openrouter-openapi.json
jq -r '.paths | to_entries[] | .key as $p | (.value | keys[] | select(. != "parameters")) as $m | "\($m|ascii_upcase) \($p)"' /tmp/openrouter-openapi.json | sort
```

For the repo-level drift report and baseline refresh flow, see [`docs/operations/openapi-drift-reporting.md`](openapi-drift-reporting.md).
