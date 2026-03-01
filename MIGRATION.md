# Migration Guide: 0.5.x -> 0.6.0

This guide is for users upgrading from the `0.5.x` API surface to the planned `0.6.0` API surface.

## Quick Checklist

1. Replace `provisioning_key` naming with `management_key`.
2. Move legacy completions usage to `api::legacy::completion` and `client.legacy().completions()`.
3. Move from flat client calls to domain clients (`chat()`, `responses()`, `messages()`, `models()`, `management()`).
4. Replace old pagination shapes with `types::PaginationOptions`.
5. Update renamed domain methods (`count`, `list_for_user`, `exchange_code_for_api_key`).

## Breaking-Change Mapping

| Area | Old Usage | New Usage (`0.6.0` target) |
| --- | --- | --- |
| Management key naming | `builder().provisioning_key(...)` | `builder().management_key(...)` |
| Runtime management key naming | `set_provisioning_key`, `clear_provisioning_key` | `set_management_key`, `clear_management_key` |
| Legacy completions module | `api::completion::*` | `api::legacy::completion::*` |
| Legacy completion call path | `client.send_completion_request(...)` | `client.legacy().completions().create(...)` |
| Models user list method | `models().list_for_user()` | `models().list_user_models()` |
| Models count method | `models().count()` | `models().get_model_count()` |
| PKCE exchange in management domain | `management().exchange_code_for_api_key(...)` | `management().create_api_key_from_auth_code(...)` |
| API key management pagination | `client.list_api_keys(Some(offset_f64), include_disabled)` | `client.management().list_api_keys(Some(PaginationOptions::with_offset(offset_u32)), include_disabled)` |
| Guardrails access path | `client.list_guardrails(...)` | `client.management().list_guardrails(...)` |
| Flat endpoint calls | `client.send_chat_completion`, `client.stream_response`, `client.create_api_key`, ... | `client.chat().create`, `client.responses().stream`, `client.management().create_api_key`, ... |

## Top 10 Before/After Recipes

### 1) Builder management key rename

Before:

```rust
let client = OpenRouterClient::builder()
    .provisioning_key("or-mgmt-key")
    .build()?;
```

After:

```rust
let client = OpenRouterClient::builder()
    .management_key("or-mgmt-key")
    .build()?;
```

### 2) Runtime management key rename

Before:

```rust
client.set_provisioning_key("or-mgmt-key");
client.clear_provisioning_key();
```

After:

```rust
client.set_management_key("or-mgmt-key");
client.clear_management_key();
```

### 3) Legacy completions import path

Before:

```rust
use openrouter_rs::api::completion::CompletionRequest;
```

After:

```rust
use openrouter_rs::api::legacy::completion::CompletionRequest;
```

### 4) Legacy completions call path

Before:

```rust
let response = client.send_completion_request(&request).await?;
```

After:

```rust
let response = client.legacy().completions().create(&request).await?;
```

### 5) Flat chat call -> domain chat call

Before:

```rust
let response = client.send_chat_completion(&request).await?;
```

After:

```rust
let response = client.chat().create(&request).await?;
```

### 6) Flat response stream -> domain response stream

Before:

```rust
let stream = client.stream_response(&request).await?;
```

After:

```rust
let stream = client.responses().stream(&request).await?;
```

### 7) Models renamed methods

Before:

```rust
let user_models = client.models().list_for_user().await?;
let count = client.models().count().await?;
```

After:

```rust
let user_models = client.models().list_user_models().await?;
let count = client.models().get_model_count().await?;
```

### 8) PKCE exchange renamed in management domain

Before:

```rust
let auth = client
    .management()
    .exchange_code_for_api_key("code", Some("verifier"), None)
    .await?;
```

After:

```rust
let auth = client
    .management()
    .create_api_key_from_auth_code("code", Some("verifier"), None)
    .await?;
```

### 9) API keys pagination migration

Before:

```rust
let keys = client.list_api_keys(Some(0.0), Some(false)).await?;
```

After:

```rust
use openrouter_rs::types::PaginationOptions;

let keys = client
    .management()
    .list_api_keys(Some(PaginationOptions::with_offset(0)), Some(false))
    .await?;
```

### 10) Guardrails flat call -> management domain

Before:

```rust
use openrouter_rs::types::PaginationOptions;

let guardrails = client
    .list_guardrails(Some(PaginationOptions::with_offset_and_limit(0, 25)))
    .await?;
```

After:

```rust
use openrouter_rs::types::PaginationOptions;

let guardrails = client
    .management()
    .list_guardrails(Some(PaginationOptions::with_offset_and_limit(0, 25)))
    .await?;
```

## Feature Notes

- Legacy text completions require the `legacy-completions` feature.
- On the target `0.6.x` line, enable it explicitly if needed:

```toml
[dependencies]
openrouter-rs = { version = "0.6", features = ["legacy-completions"] }
```

## Validation Tips

- Compile your migration branch with strict checks:
  - `cargo check --all-targets --all-features`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Re-run your integration suite with real keys after API call-path updates.
