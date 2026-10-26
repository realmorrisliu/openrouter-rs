# sdk-public-type-future-proofing Specification

## Purpose
TBD - created by archiving change mark-public-response-types-non-exhaustive. Update Purpose after archive.
## Requirements
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

### Requirement: Caller-built types keep ergonomic construction
Every public request or configuration type marked `#[non_exhaustive]` that callers are expected to construct SHALL expose a builder, constructor, or documented helper that supports all required fields and optional fields.

#### Scenario: Request type becomes non-exhaustive
- **WHEN** a public request type is marked `#[non_exhaustive]`
- **THEN** examples and docs construct it through `builder()`, a constructor, or a helper rather than a public struct literal

#### Scenario: Required field remains available
- **WHEN** a caller uses the supported builder or constructor for a non-exhaustive request type
- **THEN** the caller can still set every required request field and every documented optional request field

### Requirement: Stable exhaustive APIs remain intentional
The SDK SHALL leave a public struct or enum exhaustive only when the audit records that exhaustive construction or matching is a deliberate stable API contract rather than an accidental exposure of an upstream schema.

#### Scenario: Type is intentionally exhaustive
- **WHEN** the audit keeps a public type exhaustive
- **THEN** the implementation notes or review checklist identify why future upstream drift is not expected to require additional public fields or variants

### Requirement: Migration guidance documents source breaks
The SDK SHALL document the 0.10.0 source-level migration for affected direct struct literals and exhaustive enum matches.

#### Scenario: Caller used direct struct literals
- **WHEN** a caller reads the 0.10.0 migration guidance
- **THEN** the guidance shows that direct literals for affected non-exhaustive types must be replaced with builders, constructors, helper methods, or deserialization paths

#### Scenario: Caller used exhaustive enum matching
- **WHEN** a caller reads the 0.10.0 migration guidance
- **THEN** the guidance explains that affected enum matches need a wildcard arm outside the crate

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

