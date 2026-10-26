# sdk-public-surface Specification

## Purpose
TBD - created by archiving change unify-sdk-surface-and-transport. Update Purpose after archive.
## Requirements
### Requirement: One canonical entry point per endpoint family

The SDK SHALL expose exactly one documented public path to each endpoint family: the domain client returned by `OpenRouterClient`. No public function may perform an HTTP request without receiving caller-supplied client configuration.

#### Scenario: Caller lists credits
- **WHEN** a caller wants the authenticated account's credits
- **THEN** the only public path is `client.management().get_credits()`

#### Scenario: Endpoint function builds its own transport
- **WHEN** any public API function would need to construct its own HTTP client because its signature has no client parameter
- **THEN** that function is not public

#### Scenario: Rustdoc search
- **WHEN** a caller searches the crate documentation for an endpoint operation
- **THEN** the result is a domain-client method, not a standalone function taking a base URL and API key

### Requirement: Transport failures preserve their classification

The SDK SHALL report a transport failure in a form that lets the caller distinguish a timeout, a connection failure, a request-building failure, a body-read failure, and a decoding failure, without parsing an error message, and without exposing a backend-specific error type.

#### Scenario: Request times out
- **WHEN** an HTTP request exceeds the configured client timeout
- **THEN** the returned error identifies the failure as a timeout and exposes the request URL when one is available

#### Scenario: Connection cannot be established
- **WHEN** the connection cannot be established
- **THEN** the returned error identifies the failure as a connection failure

#### Scenario: Backend is not reqwest
- **WHEN** the SDK is built with the `wasip3` feature and no `reqwest` feature
- **THEN** the same classification type is used and the same accessors are available

#### Scenario: Caller wants to compare errors
- **WHEN** a caller compares two transport errors
- **THEN** the public error type remains comparable and cloneable, because it holds a classification rather than a backend error

### Requirement: API errors expose retry guidance

An API error SHALL carry the server-provided retry delay when the response includes a `Retry-After` header.

#### Scenario: Rate limited with Retry-After
- **WHEN** OpenRouter responds with 429 and a `Retry-After` delay in seconds
- **THEN** the API error exposes both that delay and a retryable classification

#### Scenario: Retry-After uses an HTTP date
- **WHEN** `Retry-After` uses the HTTP-date form
- **THEN** the delay is reported as absent rather than guessed

#### Scenario: Non-retryable error
- **WHEN** the API error is not retryable
- **THEN** the error reports that it is not retryable and carries no retry delay

### Requirement: One streaming event model across sources

The SDK SHALL expose a single streaming event model for chat, Responses, and Messages, and that model SHALL carry assembled tool calls.

#### Scenario: Streaming a chat response with tool calls
- **WHEN** a caller streams a chat request whose model invokes a tool
- **THEN** the terminal event contains the fully assembled tool calls, in addition to content and reasoning deltas delivered earlier

#### Scenario: Streaming a Responses or Messages request
- **WHEN** a caller streams against a non-chat source
- **THEN** the same event model is used, with tool calls absent where the source provides none

#### Scenario: Usage on the terminal event
- **WHEN** the terminal event carries usage
- **THEN** it is exposed as the typed usage model rather than an untyped JSON value

#### Scenario: Raw chunks remain available
- **WHEN** a caller needs individual chunks instead of the unified projection
- **THEN** the raw chat stream remains public as the input the adapters consume

### Requirement: Mid-stream API failures are API errors

The SDK SHALL map an API error payload delivered inside an event stream to an API error, not to a deserialization error.

#### Scenario: Provider fails after headers were sent
- **WHEN** an event stream delivers an API error payload after a successful response status
- **THEN** the stream yields an `OpenRouterError::Api` carrying the upstream code and message

#### Scenario: Malformed event payload
- **WHEN** a stream delivers a payload that is neither a valid event nor an API error
- **THEN** the stream yields a deserialization error

### Requirement: Request types declare each wire field once

A public request type SHALL declare its wire fields once, so that adding a field cannot desynchronize the type from its serializer.

#### Scenario: A new request field is added
- **WHEN** a developer adds an optional field to a public request type
- **THEN** the field reaches the wire after a single declaration, without editing a separate serializer or a duplicate wire struct

#### Scenario: Wire shape differs from the struct shape
- **WHEN** a request type must merge or restructure fields for the wire
- **THEN** that difference is expressed by the field's own type, with tests covering each variant

#### Scenario: Absent and empty tool lists
- **WHEN** a request carries no tools, or an explicitly empty tool list
- **THEN** the serialized payload omits the field or sends an empty array according to the field's documented behavior

