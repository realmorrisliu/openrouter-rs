## ADDED Requirements

### Requirement: JSON endpoints share one decode path

Every endpoint that returns a body SHALL decode success and failure through one shared helper.

#### Scenario: Endpoint returns a non-2xx response
- **WHEN** any JSON endpoint receives a non-success status
- **THEN** the caller receives a normalized `OpenRouterError::Api` produced by the shared helper

#### Scenario: Endpoint returns a success response that fails to deserialize
- **WHEN** a success response body does not match the expected type
- **THEN** the error is produced by the shared helper with the endpoint context string, and an embedded API error payload is still preferred over the deserialization error

#### Scenario: Error handling is exhausted
- **WHEN** an endpoint migrates to the shared helper
- **THEN** no `unreachable!()` remains in the endpoint modules and no endpoint repeats the success/error branch

#### Scenario: Success response carries no body
- **WHEN** an endpoint's success response has no body
- **THEN** it uses the shared bodyless helper rather than an ad-hoc branch

### Requirement: Client configuration is passed as one value

Per-request configuration SHALL be carried by a single context value rather than repeated as separate parameters on every transport function.

#### Scenario: A transport function needs client configuration
- **WHEN** an endpoint function requires the HTTP client, base URL, credential, or attribution headers
- **THEN** it receives them as one context value

#### Scenario: Configuration gains a field
- **WHEN** a new per-request configuration field is introduced
- **THEN** existing transport call sites compile without a signature change

### Requirement: The internal refactor preserves wire behavior

The refactor SHALL NOT change request field presence, request field values, or response decoding outcomes.

#### Scenario: Wire compatibility
- **WHEN** the same request is serialized before and after the refactor
- **THEN** the resulting JSON is equal as a value, ignoring key order

#### Scenario: Response decoding
- **WHEN** an endpoint receives the same response body before and after the refactor
- **THEN** the decoded value and the raised error are the same
