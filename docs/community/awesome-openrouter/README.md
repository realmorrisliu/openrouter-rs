# Get Started with OpenRouter in Rust

Use [`openrouter-rs`](../../../README.md) for a typed Rust SDK, or [`openrouter-cli`](../../../crates/openrouter-cli/README.md) for terminal workflows. This is a community-maintained project, listed in [Works with OpenRouter](https://openrouter.ai/works-with-openrouter/openrouter-rs).

## SDK Setup

1. Create an [OpenRouter account](https://openrouter.ai/) and generate an [API key](https://openrouter.ai/settings/keys). Ensure your account can use the model you select; paid models require credits.
2. Follow the [installation and Quick Start](../../../README.md#installation) for the current published SDK dependency, required Rust version, and complete `src/main.rs` example.
3. Set `OPENROUTER_API_KEY` in the terminal that runs your application, then run `cargo run`. Never put your key into source code or a browser bundle.

```bash
export OPENROUTER_API_KEY="your-openrouter-api-key"
cargo run
```

The Quick Start prints a chat response. Set `OPENROUTER_MODEL` to choose another model available to your account. If authentication fails, verify the key; if the API reports insufficient credits or an unavailable model, resolve that account/model condition before retrying.

Choose your next workflow:

- [Chat](../../../examples/domain_chat_completion.rs) and [streaming](../../../examples/stream_chat_completion.rs)
- [Typed tools](../../../examples/typed_tool_calling.rs) and a [bounded agent loop](../../../examples/typed_tool_agent.rs)
- [Axum HTTP gateway](../../../examples/axum_chat_gateway.rs)
- [Responses](../../../examples/create_response.rs) and [Messages](../../../examples/create_message.rs)
- [API reference](https://docs.rs/openrouter-rs) and [upgrade guidance](../../../MIGRATION.md)

Accepted endpoint support and live-test evidence are tracked in the [endpoint matrix](../../operations/official-endpoint-test-matrix.md).

## CLI Setup

Install the published companion CLI:

```bash
cargo install openrouter-cli --locked
export OPENROUTER_API_KEY="your-openrouter-api-key"
openrouter-cli --help
openrouter-cli models list
openrouter-cli usage credits
```

The SDK and CLI have independent release versions. See the [CLI README](../../../crates/openrouter-cli/README.md) for supported commands and configuration, and the [automation recipes](../../operations/cli-automation-workflows.md) for shell/CI usage. Management-governed commands require `OPENROUTER_MANAGEMENT_KEY`; chat and credits use the ordinary API key.

## Listing Maintenance

This URL is the stable setup destination used by the [Awesome OpenRouter](https://github.com/OpenRouterTeam/awesome-openrouter) listing. Installation versions are maintained in the root README rather than duplicated here.

- Listing metadata: [`app.yaml`](app.yaml)
- Logo assets: [`logo.svg`](logo.svg), [`logo.png`](logo.png)
- Original submission text and dated traction evidence: [`pr-draft.md`](pr-draft.md)

The original submission draft is historical material, not current installation guidance. When updating an upstream listing, verify its existing entry first and update only changed metadata. Keep the SDK and companion CLI in the same project entry.

The logo combines the OpenRouter mark with Ferris's `flat-happy` SVG from rustacean.net (CC0).
