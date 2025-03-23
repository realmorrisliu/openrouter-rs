# OpenRouter Rust SDK

`openrouter-rs` is a third-party Rust SDK that helps you interact with the OpenRouter API. It wraps various endpoints of the OpenRouter API, making it easier to use in Rust projects. By taking advantage of Rust's strengths like type safety, memory safety, and concurrency without data races, `openrouter-rs` ensures a solid and reliable integration with the OpenRouter API.

## Current Status

This SDK is currently being used in my own project and has not been fully tested. If you encounter any issues while using it, please open an issue, and I will address it as soon as possible.

## Features

- Create API keys
- Retrieve current API key information
- Delete exposed API keys
- Send chat completion requests
- Send text completion requests
- Create Coinbase charge requests
- Retrieve total credits
- Retrieve generation request metadata
- List all available models
- List endpoints for a specific model

## Planned Features

- Streaming support for chat and text completion requests

## Installation

To add `openrouter-rs` to your project, include it in your `Cargo.toml` file:

```toml
[dependencies]
openrouter-rs = { path = "path/to/openrouter-rs" }
```

## Usage Example

Here is a simple example demonstrating how to use the `openrouter-rs` library:

```rust
use openrouter_rs::{
    OpenRouterClient,
    api::{
        chat::{ChatCompletionRequest, Message},
        completion::CompletionRequest,
        credits::CoinbaseChargeRequest,
        generation::GenerationRequest,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenRouterClient::new("your_api_key".to_string());

    // Create API key
    let api_key = client.create_api_key("My API Key", Some(100.0)).await?;
    println!("{:?}", api_key);

    // Retrieve current API key information
    let api_key_info = client.get_current_api_key_info().await?;
    println!("{:?}", api_key_info);

    // Delete exposed API key
    client
        .delete_exposed_api_key("your_exposed_key_hash")
        .await?;

    // Send chat completion request
    let chat_request = ChatCompletionRequest::new(
        "deepseek/deepseek-chat:free",
        vec![Message::new("user", "What is the meaning of life?")],
    )
    .max_tokens(100)
    .temperature(0.7);

    let chat_response = client.send_chat_completion(&chat_request).await?;
    println!("{:?}", chat_response);

    // Send text completion request
    let completion_request = CompletionRequest::new("deepseek/deepseek-chat:free", "Once upon a time")
        .max_tokens(100)
        .temperature(0.7);

    let completion_response = client.send_completion_request(&completion_request).await?;
    println!("{:?}", completion_response);

    // Create Coinbase charge request
    let coinbase_request = CoinbaseChargeRequest::new(1.1, "your_ethereum_address", 1);

    let coinbase_response = client.create_coinbase_charge(&coinbase_request).await?;
    println!("{:?}", coinbase_response);

    // Retrieve total credits
    let credits = client.get_credits().await?;
    println!("{:?}", credits);

    // Retrieve generation request metadata
    let generation_request = GenerationRequest::new("your_generation_id");

    let generation_data = client.get_generation(&generation_request).await?;
    println!("{:?}", generation_data);

    // List all available models
    let models = client.list_models().await?;
    println!("{:?}", models);

    // List endpoints for a specific model
    let endpoints = client.list_model_endpoints("author", "slug").await?;
    println!("{:?}", endpoints);

    Ok(())
}
```

## Examples

I put together a bunch of example programs in the `examples` directory to help you get started with `openrouter-rs`. Each example is a standalone executable that shows off a specific feature of the SDK. You can run these examples to see how to use the SDK in different scenarios.

### Running Examples

To run an example, use the following command:

```sh
cargo run --example <example_name>
```

Replace `<example_name>` with the name of the example you want to run. For example, to run the `create_api_key` example, use the following command:

```sh
cargo run --example create_api_key
```

## Risk Disclaimer

Please note that `openrouter-rs` is a third-party SDK and is not officially affiliated with or endorsed by OpenRouter. Use this SDK at your own risk. The maintainers of this SDK are not responsible for any issues or damages that may arise from using this SDK. Ensure that you understand the terms and conditions of the OpenRouter API and comply with them while using this SDK.

## Contributing

Contributions are welcome! Please fork this repository and submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
