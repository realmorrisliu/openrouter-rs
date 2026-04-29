## Public Type Inventory

Audit command:

```bash
rg '^pub (struct|enum) ' src/api src/types -n
```

## Marked Non-Exhaustive

These public types mirror upstream OpenRouter/OpenAI/Anthropic request, response, metadata, usage, pricing, discovery, streaming, or taxonomy shapes. They are intentionally `#[non_exhaustive]`.

| Module | Types |
| --- | --- |
| `src/api/audio.rs` | `SpeechResponseFormat`, `SpeechProviderOptions`, `SpeechRequest` |
| `src/api/auth.rs` | `AuthRequest`, `CodeChallengeMethod`, `AuthResponse`, `UsageLimitType`, `CreateAuthCodeRequest`, `AuthCodeData` |
| `src/api/api_keys.rs` | `ApiKey`, `ApiKeyDetails`, `RateLimit` |
| `src/api/chat.rs` | `ImageUrl`, `InputAudio`, `VideoUrl`, `FileInput`, `CacheControlType`, `CacheControl`, `ContentPart`, `Content`, `Message`, `Modality`, `DebugOptions`, `StreamOptions`, `TraceOptions`, `Plugin`, `StopSequence`, `ChatCompletionRequest` |
| `src/api/credits.rs` | `CoinbaseChargeRequest`, `CoinbaseChargeData`, `CreditsData` |
| `src/api/discovery.rs` | `BigNumber`, `Provider`, `PublicPricing`, `ModelArchitecture`, `TopProviderInfo`, `PerRequestLimits`, `UserModel`, `ModelsCountData`, `PercentileStats`, `PublicEndpoint`, `ActivityItem` |
| `src/api/embeddings.rs` | `EmbeddingEncodingFormat`, `EmbeddingImageUrl`, `EmbeddingContentPart`, `EmbeddingMultimodalInput`, `EmbeddingInput`, `EmbeddingRequest`, `EmbeddingVector`, `EmbeddingData`, `EmbeddingPromptTokensDetails`, `EmbeddingUsage`, `EmbeddingResponse` |
| `src/api/generation.rs` | `GenerationData`, `GenerationContentData` |
| `src/api/guardrails.rs` | `Guardrail`, `GuardrailListResponse`, `CreateGuardrailRequest`, `UpdateGuardrailRequest`, `GuardrailKeyAssignment`, `GuardrailMemberAssignment`, `GuardrailKeyAssignmentsResponse`, `GuardrailMemberAssignmentsResponse`, `BulkKeyAssignmentRequest`, `BulkMemberAssignmentRequest`, `AssignedCountResponse`, `UnassignedCountResponse` |
| `src/api/legacy/completion.rs` | `CompletionRequest` |
| `src/api/messages.rs` | `AnthropicRole`, `AnthropicSystemTextBlock`, `AnthropicSystemTextBlockType`, `AnthropicSystemPrompt`, `AnthropicMessageContent`, `AnthropicContentPart`, `AnthropicMessage`, `AnthropicMessagesMetadata`, `AnthropicTool`, `AnthropicToolChoice`, `AnthropicThinking`, `AnthropicOutputEffort`, `AnthropicOutputConfig`, `AnthropicMessagesRequest`, `AnthropicMessagesUsage`, `AnthropicMessagesResponse`, `AnthropicMessagesStreamEvent`, `AnthropicMessagesSseEvent` |
| `src/api/models.rs` | `Model`, `Architecture`, `TopProvider`, `Pricing`, `Endpoint`, `EndpointPricing`, `EndpointData`, `EndpointArchitecture` |
| `src/api/organization.rs` | `OrganizationMember`, `OrganizationMembersResponse` |
| `src/api/rerank.rs` | `RerankRequest`, `RerankDocument`, `RerankResult`, `RerankUsage`, `RerankResponse` |
| `src/api/responses.rs` | `ResponsesRequest`, `ResponsesResponse`, `ResponsesStreamEvent` |
| `src/api/videos.rs` | `VideoImageUrl`, `VideoInputReference`, `VideoFrameImage`, `VideoProviderOptions`, `VideoGenerationRequest`, `VideoGenerationUsage`, `VideoGenerationResponse`, `VideoModel` |
| `src/api/workspaces.rs` | `Workspace`, `WorkspaceListResponse`, `CreateWorkspaceRequest`, `UpdateWorkspaceRequest`, `WorkspaceMember`, `WorkspaceMembersRequest`, `WorkspaceMembersAddResponse`, `WorkspaceMembersRemoveResponse` |
| `src/types/completion.rs` | `ReasoningDetail`, `ResponseCostDetails`, `ResponseUsage`, `FunctionCall`, `ToolCall`, `PartialFunctionCall`, `PartialToolCall`, `ErrorResponse`, `Choice`, `FinishReason`, `NonChatChoice`, `NonStreamingChoice`, `StreamingChoice`, `Message`, `Delta`, `ObjectType`, `CompletionsResponse` |
| `src/types/mod.rs` | `ApiResponse<T>`, `Role`, `Effort`, `ReasoningConfig`, `ModelCategory`, `SupportedParameters` |
| `src/types/pagination.rs` | `PaginationOptions` |
| `src/types/provider.rs` | `ProviderSortBy`, `DataCollectionPolicy`, `Quantization`, `PerformancePreference`, `PercentileCutoffs`, `PriceLimit`, `MaxPrice`, `ProviderPreferences` |
| `src/types/response_format.rs` | `ResponseFormatType`, `JsonSchemaConfig`, `ResponseFormat` |
| `src/types/stream.rs` | `StreamEvent`, `UnifiedStreamSource`, `UnifiedStreamEvent` |
| `src/types/tool.rs` | `Tool`, `FunctionDefinition`, `ToolChoice`, `SpecificToolChoice`, `SpecificToolFunction` |

## Kept Exhaustive

These public types are deliberately outside the upstream-schema rule.

| Module | Types | Reason |
| --- | --- | --- |
| `src/api/workspaces.rs` | `UpdateWorkspaceRequestWithClearedIoLoggingApiKeyIds<'a>` | Serialization wrapper with private fields, created only by `UpdateWorkspaceRequest::with_cleared_io_logging_api_key_ids()`. External callers cannot construct it with a public literal. |
| `src/types/tool.rs` | `ToolBuilder` | SDK-owned builder helper with private fields; callers use builder methods and `build()`. |
| `src/types/stream.rs` | `ToolAwareStream` | SDK-owned stream wrapper with private fields; callers receive it from stream APIs. |

Public client and error types live outside the audited `src/api` and `src/types` OpenAPI model surface and remain governed by their existing SDK-owned API contracts.
