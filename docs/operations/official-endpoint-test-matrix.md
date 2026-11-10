# Official Endpoint Test Matrix

Snapshot date: 2026-11-10
Source of truth: `https://openrouter.ai/openapi.json` (method+path extracted from latest spec)  
Tracked baseline: `specs/openrouter/openapi-baseline.json`  
Weekly drift workflow: `.github/workflows/openapi-drift.yml`

## Public Access Path

Every endpoint in this matrix is reached through a domain client
(`client.chat()`, `client.responses()`, `client.messages()`, `client.audio()`,
`client.images()`, `client.videos()`, `client.files()`, `client.models()`,
`client.tools()`,
`client.rerank()`, `client.batches()`, `client.vault()`,
`client.decisions()`, `client.management()`). Since 0.23.0 the `api::*` endpoint
free functions and the flat `OpenRouterClient` methods no longer exist, so the
implementation column counts domain-client coverage only. Endpoint paths did not
change.

## Coverage Summary

- Official OpenAPI endpoints: `169` method+path entries.
- SDK implementation coverage (`src/api` + domain client): `158 / 169` (`93.5%`).
- Live integration coverage (`tests/integration`): `27 / 169` endpoints currently exercised.
  - Covered live now: `GET /oauth/jwks`, `POST /chat/completions`, `POST /messages`, `POST /responses`, `POST /embeddings`, `POST /rerank`, `GET /key`, `GET /models`, `GET /models/user`, `GET /models/count`, `GET /models/{author}/{slug}/endpoints`, `GET /providers`, `GET /endpoints/zdr`, `GET /embeddings/models`, `GET /keys`, `POST /keys`, `GET /keys/{hash}`, `PATCH /keys/{hash}`, `DELETE /keys/{hash}`, `GET /guardrails`, `POST /guardrails`, `GET /guardrails/{id}`, `PATCH /guardrails/{id}`, `DELETE /guardrails/{id}`, `GET /organization/members`, `GET /tools`, `GET /tools/{name}`

Chat response regression coverage:

- PR #280: local tests cover `reasoning_content`/`reasoning_text` response and raw-delta decoding, request replay, and optional completion-token details. Canonical reasoning getters and stream adapters keep their existing behavior. No new live qualification is claimed.

- PR #279: local serde and mock HTTP tests cover shape-directed choice decoding, string error codes, omitted tool-call types, and nullable upstream chat costs. No endpoint or live coverage count changes.

- Issues #259/#260: local serde/request tests cover non-streaming assistant-message replay (including null content, signed/encrypted reasoning and modeled metadata), getter behavior for streaming/legacy choices, and optional prompt-cache usage in both response envelopes. Regression coverage also verifies all four official reasoning variants across message replay and both chat stream adapters. The streaming tool example has a local test for ordered multi-chunk replay, including repeated reasoning indices. No new endpoint or live qualification is claimed. The CLI has no chat command consuming these types.

Drift review note:

- Issue #297 (2026-11-09): accepted the `Guardrail Policies` family (23 operations) and the agent credit purchase pair. `management()` now exposes account policy CRUD (`list_account_guardrail_policies`/`create_account_guardrail_policy`/`update_account_guardrail_policy`/`delete_account_guardrail_policy`), workspace budget/model-availability/moderation policy CRUD (`list_*_policies`, `create_*_policy`, `get_*_policy`, `update_*_policy`, `delete_*_policy`), API key and member policy assignments (`get_*_policy_assignment`, `change_*_policy_assignment`), and `create_credit_purchase`/`get_credit_purchase`. Budget authoring keeps an explicit-null config form because upstream requires `limit_usd` and `reset_interval` keys; moderation reuses the guardrail content-filter types; Model Armor config, PII filter entries, image `markup` and the policy detail `reference` stay on `Value`. Also accepted: `GET /models?endpoints=`, image `frame_count`/`frame_duration_ms`/`markup` plus `gif` output, `openrouter:mcp` server-tool constructor, and the auth-code `spawn_agent`/`spawn_cloud` fields removed upstream. Reviewed as already covered: provider taxonomy additions (Ant Digital/Parallel/Pearl Research/River and the new BYOK provider slugs), `tools_invoke` `api_type`, `org:viewer`/`viewer`/`restricted_viewer` roles, `openrouter:mcp` in `disabled_server_tools`, `deferred_tools` controls, Responses `compaction`/`mcp_*` output items, and the `openrouter-stealth-v1` reasoning format - all string or `Value` surfaces. Deferred as out of scope: the preset inference `workspace_id` query parameter (adding it would break existing method signatures) and upstream's removal of `PUT /private-endpoints/{id}/parameters` and `PUT /private-endpoints/{id}/pricing` (never covered).
- Issue #291 (2026-11-02): accepted `Model`/`UserModel` `endpoints` (the request API paths that serve the model), `PublicEndpoint` `safety_retention` on `GET /endpoints/zdr`, and Responses `access_programs.cyber`. Upstream also removed private-provider `data_policy` from the create/update requests and every response, so `PrivateProviderDataPolicy` and its fields are gone (breaking). Reviewed as already covered: generation `api_type` `extractions` and provider `provider_sort_order_mode`/`provider_sort_order_rank` stay flexible strings and `extra` maps, and Batch result bodies keep echoing through `Value`. Deferred as out of scope: `POST /api/alpha/extractions`, whose alpha request/response shape (nine typed field specs and nine answer shapes) is documented as unstable upstream.
- Issue #285 (2026-10-30): accepted the drift that renames `/end-users*` to `/external-users*`, removes the `/interns*` family, and adds `/private-providers*`. `management()` now exposes `list_external_users`/`create_external_user`/`get_external_user`/`update_external_user`/`delete_external_user` and `list_private_providers`/`create_private_provider`/`get_private_provider`/`update_private_provider`/`delete_private_provider`; `client.interns()` and its types are gone. Also accepted: guardrail `enforce_first_party_harness_models`, prompt-cache breakpoint `ttl`, user-model `client` filter, and vault connection `routing`/`routing_text`. Deferred as out of scope: the never-covered `/private-endpoints*` family, including the newly added but deprecated `PUT /private-endpoints/{id}/parameters`, plus provider-UI metadata inside flexible objects (`authMode`, `visibleWhen`) and provider option maps that already round-trip through `Value`.
- Issue #273 (2026-10-20): accepted the two added operations `GET /interns/{internId}/icon` and `GET /vault/interns/{internId}/connections` through `client.interns().get_icon(...)` and `client.vault().list_connections_for_intern(...)` (the `/interns*` family and `client.interns()` were later removed upstream; see Issue #285 - only the vault read remains). Added Batch `callback_url`/`expires_at`/`provider`, endpoint `service_tier`/`supports_diarization`, guardrail `allow_safety_retention_*`, observability `broadcast_generation_cost_allocation`, video signed `urls`, intern `created_by_you`/`icon_sha256`, route-bound daemon signing (`route`, nonce and v2 signature), user-model `sort`, and Responses `stop`. Chat `alignment`/`safeguard_results` and Responses `openrouter:armor_blocked` items stay flexible (`Value`); Armor/alignment plugin configuration stays on `Plugin` maps. New 408/409/499 statuses use the shared error parser. No CLI flags change.
- Issue #264 (2026-10-13): accepted canonical `POST /interns/{internId}/daemon/sign` and the deprecated `daemon-access/sign` alias. Local HTTP tests cover encoded paths, digest request casing/validation, both routes, OAuth session bearer forwarding, attribution headers, 403 responses for ordinary API keys and insufficient grants, malformed credential errors and absent auth. Signature values have no Debug implementation. These routes require an `ori login --oidc` session with `vault:read`, including the deprecated alias; ordinary API keys and `interns`-only grants are rejected with 403. The existing client `api_key` slot forwards the session token. Mock tests verify forwarding and error handling; session/scope validation remains server-owned and has no live qualification.
- Added BYOK declared regions with omit/set/clear/reset semantics, key budgets/effective budget and last-used timestamps, expired-key filtering, key workspace moves, video expiration, speech turn input/instructions, portable deferred-tool controls, chat function deferral flags, and Messages tool-search result blocks (including streaming). Older responses and text-input builder calls remain compatible; direct speech input field access needs the documented enum migration.
- Reviewed all 27 changed operations: Responses input/output/tool-search results, allowed callers/output schemas, failable server tools and Batch result bodies use existing flexible JSON; plugins preserve `cost_tier`/`judge_model`; provider names remain strings. End-user ID constraints and the 500-key bulk assignment limit remain server-validated; new conflict/forbidden statuses use existing typed error handling. Existing 13 deferred operations remain deferred.
- API-key update supports `clear_limit()` with explicit null, including combined workspace moves. Shared PATCH deserialization now preserves existing explicit-null clear operations for API-key, BYOK, guardrail and observability requests, including the new BYOK region. Regression tests cover omit/set/clear/reset, JSON round trips, empty arrays, false values, invalid types and an HTTP body containing `"limit": null`.
- CLI audit: key list/update retain existing positional methods and snapshots; optional new response fields are omitted when absent. CLI has no speech/chat/Messages/Responses/Intern signing commands. No CLI flags change. The legacy live reasoning assertion now follows upstream `include_reasoning: true` semantics (`reasoning: {}`); explicit enablement is separately tested.

- Issue #257 (2026-10-06): accepted all seven new end-user registration and organization settings operations. Local HTTP regression tests in `tests/unit/openapi_issue_251.rs` cover every route, encoded tracking IDs, list filters, management-key selection, attribution headers, false-valued PATCH bodies, 204 deletion, API errors and missing-key rejection. Read-only live smoke tests are opt-in and have not been qualified live; registration writes and organization policy changes are not run unattended.
- Typed Batch provider `allow_fallbacks`, audio provider `data_collection`/`zdr` and activity `cached_tokens`; omitted audio fields and older activity responses retain their serialization. CLI activity JSON reuses SDK serde and the activity table retains its columns; the CLI has no audio or Batch command surface. CLI checks cover compatibility.
- Reviewed all 18 changed operations: Batch request/result JSON preserves assistant audio; observability `config` preserves `shouldIncludeCacheWriteTokens`; image resolution strings accept `1.5K`; Chat/preset request deserialization now accepts generic server tools. Tool domain constraints and generation ID format/length constraints remain server-validated; existing API error handling accepts the new generation 400 responses. Messages/Responses, provider and plugin changes continue through existing flexible fields. No migration is required.

- Issue #254 (2026-09-30): accepted all six reported additions (four Batch operations, Intern invoke and daemon-access), plus the newer canonical Intern daemon route. Local HTTP tests reuse `tests/unit/openapi_issue_251.rs` and cover path encoding, API-key/management-key auth, repeated Batch `status` parameters, 202 responses, deletion outcomes and payment-required errors. New routes have no live integration coverage.
- Accepted stable schema fields: shared provider-specific options, speech image/URL references, endpoint image/multiple-audio capabilities, SCIM display-name/external-ID filters, and transcription URL input, diarization, keyterms, language confidence, entities, channel/speaker labels and word kinds. Existing base64 constructors and pagination methods retain their shapes. Audio builders reject missing/mixed sources and base64 transcription without a format; URL, format and provider constraints remain server-validated.
- Reviewed remaining drift: dynamic provider/aspect-ratio taxonomies, plugins, Anthropic tool `strict`/thinking controls and Responses input/output/error/alignment remain covered by existing strings, extra maps and flexible values; ZDR native-tool metadata uses its existing `extra` map. Standard error statuses use the shared parser. CLI endpoint table output does not consume the new capability fields; JSON output can expose them. No CLI flags or migration steps change.
- Thirteen operations added after the issue snapshot are reviewed but deferred: Private Endpoints management (10), Tools discovery (2), and effective intern vault secrets (1). These require separate public-surface/auth review; baseline refresh records the reviewed upstream schema and does not claim SDK support for deferred routes.

- Issue #248 (2026-09-08): accepted all eight operations still present upstream. `POST /organization`, present in the September 7 issue report, was withdrawn before this review and is not implemented. Added container files, OAuth and SCIM sync jobs, and the stable request/response fields summarized in the changelog. OAuth uses public signing-key discovery and JWT-authenticated form token exchange; it does not require an existing API/management key.
- Issue #251 (2026-09-23): added eight intern operations, seven vault operations, and Decisions/System One. Workspace-budget path parameters now compare by position, so upstream `{workspace_ref}` naming does not appear as a removed-and-added route. Updated current BYOK, workspace, audio, embedding, image, rerank, video, API-key, generation, and model-endpoint fields.
- All new intern, vault, Decisions, and System One routes have local HTTP path/auth coverage in `tests/unit/openapi_issue_251.rs`; new endpoints do not have live integration coverage yet.
- Reviewed remaining schema changes: provider/output/pricing-unit/reasoning-format taxonomy remains string-based; Responses, plugins, server-tool options, Anthropic response metadata/usage and generation timing retain existing flexible payloads. New error statuses use the existing error parser. JWT, file-ID, audio-format, and video-upscale constraints remain server-validated. Analytics tag changes have no runtime effect. No CLI flags or default request payloads change; SDK consumers and CLI snapshots are checked by `just quality-ci`.
- New endpoints have local HTTP contract tests in `tests/unit/openapi_september.rs`. Public OAuth signing keys have a live test. Container reads require `OPENROUTER_TEST_CONTAINER_ID`; SCIM job reads require `OPENROUTER_TEST_SCIM_SYNC_JOB_ID` plus the existing management test opt-in. Provisioning JWTs, starting directory syncs, and promoting files are not part of unattended live tests.

- Upstream added `GET /datasets/session-cost` and `GET /workspaces/{id}/budgets/{interval}`, activity workspace filters/grouping, benchmark search filters, workspace deletion confirmation, BYOK/guardrail nullable workspace IDs, speech voice-cloning references with endpoint `supports_voice_cloning`, generation `workspace_id` and stored content errors, observability generation broadcast flags, analytics `include_unset`, and the `us` model region. The SDK types the stable fields, makes `ByokKey::workspace_id` optional, and keeps dynamic provider/resolution taxonomy on flexible string surfaces.
- Upstream added six SCIM group and group-role mapping endpoints plus provider-native Files shapes and pagination. The SDK exposes SCIM through `client.management()`, provider-backed storage through `client.files().*_for_provider(...)`, and types the stable benchmark, audio, model, guardrail, and workspace fields. High-churn provider/plugin/Responses additions continue through existing flexible representations.
- Upstream added assistant-message model annotations, BYOK-aware guardrail and workspace budget fields, workspace default guardrail IDs, transcription speaker labels, Anthropic compaction and dynamic-tool blocks, and more flexible plugin/server-tool/Responses fields. The SDK types the stable fields and continues to carry provider taxonomy, plugin options, server-tool options, and Responses payload growth through existing flexible representations.
- Upstream added explicit chat prompt-cache controls, static predicted output, image provider routing preferences, conditional pricing overrides, detailed cache-creation usage, custom guardrail `flag` actions, namespace tools, and image-only video generation. The SDK types the stable fields and reuses its existing flexible server-tool, plugin, provider-taxonomy, and Responses payload handling for the remaining schema additions.
- Upstream added `POST /generation/feedback`, new model/rankings filters, analytics classifier controls, verbose transcription fields, XAI ZDR policy, routed service tiers, server-tool reasoning details, and image text chunks. The SDK exposes stable fields and keeps flexible provider, Responses, and server-tool payloads where the upstream surface remains high-churn.
- Upstream added OpenRouter server-tool variants to chat completions, Responses API, Anthropic-compatible Messages, and preset creation request schemas. The SDK now exposes `ServerTool` helpers and preserves raw `Value` escape hatches for high-churn server-tool payloads.
- Upstream added `GET /workspaces/{id}/members`, now exposed as `client.management().list_workspace_members(...)`.
- Upstream added task classification discovery and image generation endpoints. The SDK now exposes task classifications through `client.models().get_task_classifications(...)` and images through `client.images().create(...)`, `stream(...)`, `list_models()`, and `list_model_endpoints(...)`. The unified benchmarks endpoint now allows an omitted `source`, and nullable benchmark metadata is reflected in the typed response.
- Upstream replaced the two per-source benchmark dataset endpoints with unified `GET /benchmarks` and added workspace budget management. The SDK now exposes `client.models().get_benchmarks(...)` and `client.management().list_workspace_budgets(...)` / `upsert_workspace_budget(...)` / `delete_workspace_budget(...)`. The old per-source benchmark methods remain deprecated compatibility wrappers.
- Upstream added model reasoning metadata, analytics warnings, chat server-tool usage metadata, embedding cost details, and Anthropic file document sources. The SDK now exposes typed fields/helpers for these stable surfaces while preserving flexible `Value` and `HashMap` escape hatches for high-churn payloads.
- Official audio speech routing is now `POST /audio/speech`. The SDK keeps the canonical `client.audio().speech().create(...)` surface and retries legacy `POST /tts` only as a compatibility fallback.
- Upstream added `GET /generation/content`, now exposed as `client.management().get_generation_content(...)`.
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
- Upstream added `GET /datasets/rankings-daily` and preset creation endpoints for chat-completions, Responses, and Anthropic-compatible Messages request bodies. The SDK now exposes rankings through `client.models().get_rankings_daily(...)` and preset creation through `client.management().create_*_preset(...)`.
- Upstream refreshed generation metadata, provider taxonomy values, plugin payload shapes, and Messages role variants. Existing flexible `String`, `Value`, `HashMap`, and `Option` fields carry those schema details without a public API migration.
- Upstream added analytics metadata/query endpoints, the Files API, app rankings and benchmark datasets, singular model lookup, and preset list/read/version endpoints. The SDK now exposes files through `client.files()`, the new datasets and singular lookup through `client.models()`, and analytics plus preset read/version workflows through `client.management()`.
- Upstream expanded model list filters, nullable model metadata, generation metadata fields, rerank document input shapes, video input references, auth-code workspace IDs, and the request metadata header spelling. The SDK now exposes typed builders/fields for those surfaces and sends `X-OpenRouter-Metadata` for opt-in metadata.

Legend:

- `SDK`: endpoint implemented in `openrouter-rs`.
- Canonical surface note: docs and examples prefer domain clients (`chat()`, `responses()`, `messages()`, `rerank()`, `audio().speech()`, `audio().transcriptions()`, `images()`, `videos()`, `files()`, `models()`, `management()`, `vault()`, `decisions()`). Some rows still mention retained flat `OpenRouterClient::*` wrappers when they exist.
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
| `GET /analytics/meta` | `client.management().get_analytics_meta()` | Yes | Path | No | P1 |
| `POST /analytics/query` | `client.management().query_analytics(...)` | Yes | Path | No | P1 |
| `POST /api/alpha/decisions` | `client.decisions().create(...)` | Yes | Path | No | P2 |
| `POST /auth/keys` | `client.management().create_api_key_from_auth_code(...)` | Yes | Path | No | P2 |
| `POST /auth/keys/code` | `client.management().create_auth_code(...)` | Yes | Path | No | P2 |
| `GET /byok` | `client.management().list_byok_keys(...)` | Yes | Path | No | P1 |
| `POST /byok` | `client.management().create_byok_key(...)` | Yes | Path | No | P1 |
| `GET /byok/{id}` | `client.management().get_byok_key(...)` | Yes | Path | No | P1 |
| `PATCH /byok/{id}` | `client.management().update_byok_key(...)` | Yes | Path | No | P1 |
| `DELETE /byok/{id}` | `client.management().delete_byok_key(...)` | Yes | Path | No | P1 |
| `GET /benchmarks` | `client.models().get_benchmarks(...)` | Yes | Path | No | P2 |
| `POST /chat/completions` | `client.chat().create(...)` / `client.chat().stream(...)` | Yes | Contract | Yes | Keep |
| `GET /classifications/task` | `client.models().get_task_classifications(...)` | Yes | Path | No | P2 |
| `GET /containers/{container_id}/files` | `client.files().list_container_files(...)` | Yes | Path | Conditional | P1 |
| `GET /containers/{container_id}/files/{file_id}` | `client.files().get_container_file(...)` | Yes | Path | Conditional | P1 |
| `GET /containers/{container_id}/files/{file_id}/content` | `client.files().download_container_file(...)` | Yes | Path | Conditional | P1 |
| `POST /containers/{container_id}/files/{file_id}/promote` | `client.files().promote_container_file(...)` | Yes | Path | No | P1 |
| `GET /oauth/jwks` | `client.management().get_oauth_jwks(...)` | Yes | Path | Yes | P1 |
| `POST /oauth/token` | `client.management().exchange_oauth_token(...)` | Yes | Path | No | P1 |
| `POST /scim/sync-jobs` | `client.management().create_scim_sync_job(...)` | Yes | Path | No | P1 |
| `GET /scim/sync-jobs/{id}` | `client.management().get_scim_sync_job(...)` | Yes | Path | Conditional | P1 |
| `GET /credits` | `client.management().get_credits()` | Yes | Path | No | P2 |
| `POST /credits/coinbase` | `client.management().create_coinbase_charge(...)` (deprecated upstream; use web credits purchase) | Yes (compatibility) | Path | No | P2 |
| `POST /credits/purchases` | `client.management().create_credit_purchase(...)` | Yes | Contract | No | P2 |
| `GET /credits/purchases/{id}` | `client.management().get_credit_purchase(...)` | Yes | Path | No | P2 |
| `GET /datasets/app-rankings` | `client.models().get_app_rankings(...)` | Yes | Path | No | P2 |
| `GET /datasets/rankings-daily` | `client.models().get_rankings_daily(...)` | Yes | Path | No | P2 |
| `GET /datasets/session-cost` | `client.models().get_session_cost(...)` | Yes | Path | No | P2 |
| `POST /embeddings` | `client.models().create_embedding(...)` | Yes | Contract | Yes | Keep |
| `GET /embeddings/models` | `client.models().list_embedding_models()` | Yes | Path | Yes | Keep |
| `GET /endpoints/zdr` | `client.models().list_zdr_endpoints(...)` | Yes | Contract | Yes | Keep |
| `GET /files` | `client.files().list(...)` / `list_for_provider(...)` | Yes | Path | No | P1 |
| `POST /files` | `client.files().upload(...)` / `upload_for_provider(...)` | Yes | Path | No | P1 |
| `GET /files/{file_id}` | `client.files().get_metadata(...)` / `get_metadata_for_provider(...)` | Yes | Path | No | P1 |
| `DELETE /files/{file_id}` | `client.files().delete(...)` / `delete_for_provider(...)` | Yes | Path | No | P1 |
| `GET /files/{file_id}/content` | `client.files().download_content(...)` / `download_content_for_provider(...)` | Yes | Path | No | P1 |
| `GET /generation` | `client.management().get_generation(...)` | Yes | Path | No | P2 |
| `GET /generation/content` | `client.management().get_generation_content(...)` | Yes | Path | No | P2 |
| `POST /generation/feedback` | `client.management().submit_generation_feedback(...)` | Yes | Path | No | P1 |
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
| `GET /guardrails/policies/account` | `client.management().list_account_guardrail_policies(...)` | Yes | Path | No | P1 |
| `POST /guardrails/policies/account` | `client.management().create_account_guardrail_policy(...)` | Yes | Contract | No | P1 |
| `PUT /guardrails/policies/account/{id}` | `client.management().update_account_guardrail_policy(...)` | Yes | Contract | No | P1 |
| `DELETE /guardrails/policies/account/{id}` | `client.management().delete_account_guardrail_policy(...)` | Yes | Path | No | P1 |
| `GET /guardrails/policies/budget` | `client.management().list_budget_policies(...)` | Yes | Path | No | P1 |
| `POST /guardrails/policies/budget` | `client.management().create_budget_policy(...)` | Yes | Contract | No | P1 |
| `GET /guardrails/policies/budget/{id}` | `client.management().get_budget_policy(...)` | Yes | Contract | No | P1 |
| `PUT /guardrails/policies/budget/{id}` | `client.management().update_budget_policy(...)` | Yes | Contract | No | P1 |
| `DELETE /guardrails/policies/budget/{id}` | `client.management().delete_budget_policy(...)` | Yes | Path | No | P1 |
| `GET /guardrails/policies/model-availability` | `client.management().list_model_availability_policies(...)` | Yes | Path | No | P1 |
| `POST /guardrails/policies/model-availability` | `client.management().create_model_availability_policy(...)` | Yes | Contract | No | P1 |
| `GET /guardrails/policies/model-availability/{id}` | `client.management().get_model_availability_policy(...)` | Yes | Contract | No | P1 |
| `PUT /guardrails/policies/model-availability/{id}` | `client.management().update_model_availability_policy(...)` | Yes | Contract | No | P1 |
| `DELETE /guardrails/policies/model-availability/{id}` | `client.management().delete_model_availability_policy(...)` | Yes | Path | No | P1 |
| `GET /guardrails/policies/moderation` | `client.management().list_moderation_policies(...)` | Yes | Path | No | P1 |
| `POST /guardrails/policies/moderation` | `client.management().create_moderation_policy(...)` | Yes | Contract | No | P1 |
| `GET /guardrails/policies/moderation/{id}` | `client.management().get_moderation_policy(...)` | Yes | Contract | No | P1 |
| `PUT /guardrails/policies/moderation/{id}` | `client.management().update_moderation_policy(...)` | Yes | Contract | No | P1 |
| `DELETE /guardrails/policies/moderation/{id}` | `client.management().delete_moderation_policy(...)` | Yes | Path | No | P1 |
| `GET /guardrails/policies/assignments/api-keys/{hash}` | `client.management().get_api_key_policy_assignment(...)` | Yes | Path | No | P1 |
| `PUT /guardrails/policies/assignments/api-keys/{hash}` | `client.management().change_api_key_policy_assignment(...)` | Yes | Contract | No | P1 |
| `GET /guardrails/policies/assignments/members/{member_id}` | `client.management().get_member_policy_assignment(...)` | Yes | Path | No | P1 |
| `PUT /guardrails/policies/assignments/members/{member_id}` | `client.management().change_member_policy_assignment(...)` | Yes | Contract | No | P1 |
| `POST /images` | `client.images().create(...)` / `client.images().stream(...)` | Yes | Path | No | P2 |
| `GET /images/models` | `client.images().list_models()` | Yes | Path | No | P2 |
| `GET /images/models/{author}/{slug}/endpoints` | `client.images().list_model_endpoints(...)` | Yes | Path | No | P2 |
| `GET /batches` | `client.batches().list(...)` | Yes | Path | No | P1 |
| `POST /batches` | `client.batches().create(...)` | Yes | Contract | No | P1 |
| `GET /batches/{id}` | `client.batches().get(...)` | Yes | Contract | No | P1 |
| `DELETE /batches/{id}` | `client.batches().delete(...)` | Yes | Contract | No | P1 |
| `GET /key` | `client.management().get_current_api_key_info()` | Yes | Contract | Yes | Keep |
| `GET /keys` | `client.management().list_api_keys(...)` / `client.management().list_api_keys_in_workspace(...)` | Yes | Path | Yes | Keep |
| `POST /keys` | `client.management().create_api_key(...)` / `client.management().create_api_key_in_workspace(...)` | Yes | Path | Yes | Keep |
| `GET /keys/{hash}` | `client.management().get_api_key(...)` | Yes | Path | Yes | Keep |
| `PATCH /keys/{hash}` | `client.management().update_api_key(...)` | Yes | Path | Yes | Keep |
| `DELETE /keys/{hash}` | `client.management().delete_api_key(...)` | Yes | Path | Yes | Keep |
| `GET /model/{author}/{slug}` | `client.models().get(...)` | Yes | Path | No | P2 |
| `GET /models` | `client.models().list()` | Yes | Contract | Yes | Keep |
| `GET /models/{author}/{slug}/endpoints` | `client.models().list_endpoints(...)` | Yes | Path | Yes | Keep |
| `GET /models/count` | `client.models().get_model_count()` | Yes | Contract | Yes | Keep |
| `GET /models/user` | `client.models().list_user_models()` | Yes | Path | Yes | Keep |
| `GET /observability/destinations` | `client.management().list_observability_destinations(...)` | Yes | Path | No | P1 |
| `POST /observability/destinations` | `client.management().create_observability_destination(...)` | Yes | Path | No | P1 |
| `GET /observability/destinations/{id}` | `client.management().get_observability_destination(...)` | Yes | Path | No | P1 |
| `PATCH /observability/destinations/{id}` | `client.management().update_observability_destination(...)` | Yes | Path | No | P1 |
| `DELETE /observability/destinations/{id}` | `client.management().delete_observability_destination(...)` | Yes | Path | No | P1 |
| `GET /organization/members` | `client.management().list_organization_members(...)` | Yes | Path | Yes | Keep |
| `GET /external-users` | `client.management().list_external_users(...)` | Yes | Path | No | Local HTTP tested; live qualification pending |
| `POST /external-users` | `client.management().create_external_user(...)` | Yes | Path | No | Local HTTP tested; live qualification pending |
| `GET /external-users/{user}` | `client.management().get_external_user(...)` | Yes | Path | No | Local HTTP tested; live qualification pending |
| `PATCH /external-users/{user}` | `client.management().update_external_user(...)` | Yes | Path | No | Local HTTP tested; live qualification pending |
| `DELETE /external-users/{user}` | `client.management().delete_external_user(...)` | Yes | Path | No | Local HTTP tested; live qualification pending |
| `GET /organization/settings` | `client.management().get_organization_settings(...)` | Yes | Path | No | Local HTTP tested; live qualification pending |
| `PATCH /organization/settings` | `client.management().update_organization_settings(...)` | Yes | Path | No | Local HTTP tested; live qualification pending |
| `GET /private-providers` | `client.management().list_private_providers(...)` | Yes | Path | No | P1 |
| `POST /private-providers` | `client.management().create_private_provider(...)` | Yes | Contract | No | P1 |
| `GET /private-providers/{slug}` | `client.management().get_private_provider(...)` | Yes | Path | No | P1 |
| `PATCH /private-providers/{slug}` | `client.management().update_private_provider(...)` | Yes | Contract | No | P1 |
| `DELETE /private-providers/{slug}` | `client.management().delete_private_provider(...)` | Yes | Path | No | P1 |
| `GET /presets` | `client.management().list_presets(...)` | Yes | Path | No | P2 |
| `GET /presets/{slug}` | `client.management().get_preset(...)` | Yes | Path | No | P2 |
| `POST /presets/{slug}/chat/completions` | `client.management().create_chat_completion_preset(...)` | Yes | Path | No | P2 |
| `POST /presets/{slug}/messages` | `client.management().create_message_preset(...)` | Yes | Path | No | P2 |
| `POST /presets/{slug}/responses` | `client.management().create_response_preset(...)` | Yes | Path | No | P2 |
| `GET /presets/{slug}/versions` | `client.management().list_preset_versions(...)` | Yes | Path | No | P2 |
| `GET /presets/{slug}/versions/{version}` | `client.management().get_preset_version(...)` | Yes | Path | No | P2 |
| `GET /providers` | `client.models().list_providers()` | Yes | Contract | Yes | Keep |
| `GET /scim/groups` | `client.management().list_scim_groups(...)` | Yes | Path | No | P1 |
| `GET /scim/group-mappings` | `client.management().list_scim_group_mappings(...)` | Yes | Path | No | P1 |
| `POST /scim/group-mappings` | `client.management().create_scim_group_mapping(...)` | Yes | Path | No | P1 |
| `GET /scim/group-mappings/{id}` | `client.management().get_scim_group_mapping(...)` | Yes | Path | No | P1 |
| `PATCH /scim/group-mappings/{id}` | `client.management().update_scim_group_mapping(...)` | Yes | Path | No | P1 |
| `DELETE /scim/group-mappings/{id}` | `client.management().delete_scim_group_mapping(...)` | Yes | Path | No | P1 |
| `GET /workspaces` | `client.management().list_workspaces(...)` | Yes | Path | No | P1 |
| `POST /systemone` | `client.decisions().create_system_one(...)` | Yes | Path | No | P2 |
| `GET /tools` | `client.tools().list(...)` | Yes | Contract | Yes | P1 |
| `GET /tools/{name}` | `client.tools().get(...)` | Yes | Path | Yes | P1 |
| `DELETE /vault/interns/{internId}/secrets/{name}` | `client.vault().delete_for_intern(...)` | Yes | Path | No | P1 |
| `GET /vault/interns/{internId}/connections` | `client.vault().list_connections_for_intern(...)` | Yes | Path | No | P1 |
| `GET /vault/interns/{internId}/secrets` | `client.vault().list_for_intern(...)` | Yes | Path | No | P1 |
| `POST /vault/interns/{internId}/secrets/copy` | `client.vault().copy_to_intern(...)` | Yes | Path | No | P1 |
| `PUT /vault/interns/{internId}/secrets/{name}` | `client.vault().store_for_intern(...)` | Yes | Path | No | P1 |
| `DELETE /vault/secrets/{name}` | `client.vault().delete(...)` | Yes | Path | No | P1 |
| `GET /vault/secrets` | `client.vault().list(...)` | Yes | Path | No | P1 |
| `PUT /vault/secrets/{name}` | `client.vault().store(...)` | Yes | Path | No | P1 |
| `GET /workspaces/{id}` | `client.management().get_workspace(...)` | Yes | Path | No | P1 |
| `GET /workspaces/{workspace_ref}/budgets` | `client.management().list_workspace_budgets(...)` | Yes | Path | No | P1 |
| `GET /workspaces/{id}/members` | `client.management().list_workspace_members(...)` | Yes | Path | No | P1 |
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
| `GET /workspaces/{workspace_ref}/budgets/{interval}` | `client.management().get_workspace_budget(...)` | Yes | Path | No | P1 |
| `PUT /workspaces/{workspace_ref}/budgets/{interval}` | `client.management().upsert_workspace_budget(...)` | Yes | Path | No | P1 |
| `DELETE /workspaces/{workspace_ref}/budgets/{interval}` | `client.management().delete_workspace_budget(...)` | Yes | Path | No | P1 |

## Reviewed, Deferred Operations

These are in the tracked upstream baseline and are not implemented by the SDK.

| Endpoint | Decision |
| --- | --- |
| `GET /private-endpoints` | Deferred beyond issue #254 |
| `POST /private-endpoints` | Deferred beyond issue #254 |
| `DELETE /private-endpoints/{id}` | Deferred beyond issue #254 |
| `GET /private-endpoints/{id}` | Deferred beyond issue #254 |
| `PATCH /private-endpoints/{id}` | Deferred beyond issue #254 |
| `POST /private-endpoints/{id}/activate` | Deferred beyond issue #254 |
| `POST /private-endpoints/{id}/disable` | Deferred beyond issue #254 |
| `POST /private-endpoints/{id}/enable` | Deferred beyond issue #254 |
| `POST /private-endpoints/{id}/validate` | Deferred beyond issue #254 |
| `GET /vault/interns/{internId}/effective-secrets` | Deferred beyond issue #254 |
| `POST /api/alpha/extractions` | Deferred beyond issue #291 (alpha; upstream shape is unstable) |

## Supplemental (Legacy)

The endpoints below are intentionally kept as legacy compatibility and are not part of current OpenAPI:

| Endpoint | SDK surface | Notes |
| --- | --- | --- |
| `POST /completions` | `client.legacy().completions().create(...)` (feature `legacy-completions`) | Migration-only surface toward `chat`/`responses` |
| `POST /tts` | `client.audio().speech().create(...)` | Compatibility fallback while upstream rolls out `POST /audio/speech` everywhere |
| `GET /datasets/benchmarks/artificial-analysis` | `client.models().get_benchmarks_artificial_analysis(...)` | Deprecated compatibility surface; prefer `client.models().get_benchmarks(...)` |
| `GET /datasets/benchmarks/design-arena` | `client.models().get_benchmarks_design_arena(...)` | Deprecated compatibility surface; prefer `client.models().get_benchmarks(...)` |

## Incremental Test Plan

1. P1: add management-key live coverage for assignment endpoints (`/guardrails/*/assignments/*` and `/guardrails/assignments/*`).
2. P1: add management-key live smoke coverage for `/activity` and `/analytics*`.
3. P1: add controlled file lifecycle live coverage for `/files*` once upload/download fixtures and cleanup guarantees are defined.
4. P2: keep `/credits`, `/credits/coinbase`, `/generation`, `/generation/content`, `/datasets*`, `/benchmarks`, `/model/{author}/{slug}`, `/presets*`, and `/auth/keys*` as controlled scenarios (manual or mocked contract-first) due rate limits, cost, or side effects.
5. P1/P2: add low-cost live or smoke coverage for `/audio/speech`, `/audio/transcriptions`, `/images*`, and `/videos*` once stable fixtures and cost controls are defined.
6. P1: add management-key live validation for `/workspaces*`, plus targeted workspace-scoped read/write coverage for `/keys` and `/guardrails`.
7. P1: add management-key live validation for `/byok*`, `/observability/destinations*`, and `/scim*` once safe credentials and fixtures are defined.

## Reproduce Snapshot

```bash
curl -L 'https://openrouter.ai/openapi.json' -o /tmp/openrouter-openapi.json
jq -r '.paths | to_entries[] | .key as $p | (.value | keys[] | select(. != "parameters")) as $m | "\($m|ascii_upcase) \($p)"' /tmp/openrouter-openapi.json | sort
```

For the repo-level drift report and baseline refresh flow, see [`docs/operations/openapi-drift-reporting.md`](openapi-drift-reporting.md).
