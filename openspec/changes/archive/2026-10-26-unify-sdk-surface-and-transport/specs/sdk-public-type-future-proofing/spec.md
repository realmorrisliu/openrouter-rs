## MODIFIED Requirements

### Requirement: High-churn public model types are non-exhaustive
The SDK SHALL mark public API model structs and enums as `#[non_exhaustive]` when they represent upstream OpenRouter request, response, metadata, usage, pricing, discovery, streaming, taxonomy, or error shapes that can gain fields or variants over time.

#### Scenario: Upstream adds optional response metadata
- **WHEN** OpenRouter adds an optional response metadata field to a non-exhaustive public response model
- **THEN** the SDK can add the typed field in a future minor or patch release without requiring external callers to update exhaustive struct literals

#### Scenario: Upstream adds taxonomy variant
- **WHEN** OpenRouter adds a new value to an SDK enum that models an upstream taxonomy
- **THEN** external callers must already use a wildcard arm when matching the enum outside the crate

#### Scenario: The SDK adds an error variant
- **WHEN** a future release adds a variant to the public error enum
- **THEN** callers that match on errors are already required to handle an unknown arm

## ADDED Requirements

### Requirement: Upstream taxonomy enums tolerate unknown values

A public enum that models an upstream string taxonomy SHALL deserialize an unrecognized value into an explicit unknown value rather than failing the containing message.

#### Scenario: Upstream introduces a new message role
- **WHEN** a response contains a role value the SDK does not model as a named variant
- **THEN** the message deserializes successfully and the role is preserved as an unknown value carrying the original string

#### Scenario: Unknown value round-trips
- **WHEN** a value was deserialized as an unknown taxonomy member and is serialized again
- **THEN** the original upstream string is emitted

#### Scenario: Caller-supplied taxonomy values
- **WHEN** an enum is only ever supplied by the caller rather than received from OpenRouter
- **THEN** it may reject unknown values, and the change records why an unrecognized upstream value cannot reach it
