# OpenRouter Rust SDK

This context defines project-specific language for compatibility work in the OpenRouter Rust SDK.

## Language

**OpenAPI drift acceptance**:
A reviewed upstream OpenAPI change that the SDK agrees to implement and then records in the tracked baseline. Acceptance includes SDK surface work, tests, documentation, and a refreshed drift baseline.
_Avoid_: Baseline refresh, spec sync

**OpenAPI baseline**:
The tracked reviewed OpenAPI snapshot used by the repository's drift checker. It is evidence of accepted upstream behavior, not a substitute for SDK implementation.
_Avoid_: Generated source, upstream truth

**Server tool**:
An OpenRouter-operated tool provided in a request `tools` array and executed by OpenRouter on behalf of the model, typically identified by an `openrouter:*` tool type.
_Avoid_: Plugin, function tool

**Function tool**:
A caller-defined tool whose invocation is suggested by the model but executed by the caller's application. In chat-completions payloads it is serialized with `type: "function"` and a `function` definition.
_Avoid_: Server tool, plugin

**Plugin**:
An OpenRouter request extension configured through the `plugins` array. Plugins are distinct from tools and must not be used as the canonical model for server-tool support.
_Avoid_: Server tool, function tool
