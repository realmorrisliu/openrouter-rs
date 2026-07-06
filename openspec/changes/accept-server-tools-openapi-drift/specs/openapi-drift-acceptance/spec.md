## ADDED Requirements

### Requirement: OpenAPI drift acceptance is implemented before baseline refresh
The SDK SHALL refresh the tracked OpenAPI baseline only after all accepted actionable drift in the current upstream snapshot is implemented or explicitly documented as deferred.

#### Scenario: Current drift is accepted
- **WHEN** the upstream drift report includes server-tool schema changes and added operations
- **THEN** the implementation covers every accepted changed or added operation before running `just openapi-refresh-baseline`

#### Scenario: Drift is deferred
- **WHEN** an actionable drift item is not implemented in this change
- **THEN** the change documents the deferral and does not classify that drift item as already supported

### Requirement: Chat completions support OpenAPI-defined server tools
Chat completion requests SHALL support OpenAPI-defined server-tool entries in the wire `tools` array while preserving existing function-tool construction.

#### Scenario: Function and server tools are mixed
- **WHEN** a chat completion request includes both caller-defined function tools and OpenRouter server tools
- **THEN** the serialized request contains both entries in the same `tools` array using their OpenAPI wire shapes

#### Scenario: Server tool choice is forced
- **WHEN** a chat completion request forces an OpenRouter server tool through `tool_choice`
- **THEN** the serialized `tool_choice` matches the OpenAPI `ChatServerToolChoice` shape

### Requirement: Responses API supports server-tool request and output shapes
Responses API requests and responses SHALL support the server-tool request variants and output items defined by the accepted OpenAPI schema.

#### Scenario: Responses request uses web search
- **WHEN** a Responses request includes an OpenRouter web-search server tool
- **THEN** the serialized `tools` entry matches the accepted OpenAPI server-tool schema

#### Scenario: Responses output contains server-tool item
- **WHEN** OpenRouter returns server-tool output items such as web-search or datetime output
- **THEN** the SDK deserializes them without losing status, type, identifier, and tool-specific payload fields

### Requirement: Messages API supports accepted server-tool shapes
Anthropic-compatible Messages requests SHALL support the server-tool variants included in the accepted OpenAPI schema without regressing existing Anthropic custom and hosted tool support.

#### Scenario: Existing Messages tools still serialize
- **WHEN** a caller builds an Anthropic custom or hosted tool request
- **THEN** the serialized request remains compatible with the existing SDK behavior

#### Scenario: OpenRouter server tool is included
- **WHEN** a Messages request includes an OpenRouter server tool from the accepted schema
- **THEN** the serialized `tools` entry matches the OpenAPI shape for that tool

### Requirement: Workspace member listing is covered or explicitly deferred
The implementation SHALL handle the new `GET /workspaces/{id}/members` operation from the same drift batch, unless the change explicitly documents deferral before refreshing the baseline.

#### Scenario: Workspace member listing is accepted
- **WHEN** the drift batch is implemented without deferral
- **THEN** the SDK exposes a management-key workspace member listing method and tests its path, authentication, and response shape

### Requirement: Drift classifier remains conservative
The OpenAPI drift classifier SHALL mark future tool item-shape additions as already supported only for operations whose SDK request surfaces can pass those tool payloads through correctly.

#### Scenario: Supported tool item changes
- **WHEN** upstream adds another item variant inside a supported `tools` array
- **THEN** the drift report classifies that item-level schema change as already supported

#### Scenario: Unsupported structural changes
- **WHEN** upstream changes the `tools` container type, adds a new operation, or introduces unsupported required top-level fields
- **THEN** the drift report keeps the change actionable
