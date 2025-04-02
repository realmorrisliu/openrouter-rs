use futures_util::stream::BoxStream;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::{
    api::{api_keys, chat, completion, credits, generation, models},
    config,
    error::OpenRouterError,
};

// Lazy static variables to cache the API key and config
static DEFAULT_API_KEY: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static PROVISIONING_API_KEY: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static CONFIG: Lazy<Mutex<Option<config::OpenRouterConfig>>> = Lazy::new(|| Mutex::new(None));

async fn get_default_api_key() -> Result<String, OpenRouterError> {
    let mut api_key = DEFAULT_API_KEY.lock().await;
    if api_key.is_none() {
        *api_key = Some(config::api_key::get()?.to_string());
    }
    Ok(api_key.clone().unwrap())
}

async fn get_provisioning_api_key() -> Result<String, OpenRouterError> {
    let mut api_key = PROVISIONING_API_KEY.lock().await;
    if api_key.is_none() {
        *api_key = Some(config::api_key::get()?.to_string());
    }
    Ok(api_key.clone().unwrap())
}

async fn get_config() -> Result<config::OpenRouterConfig, OpenRouterError> {
    let mut config = CONFIG.lock().await;
    if config.is_none() {
        *config = Some(config::load_config()?);
    }
    Ok(config.clone().unwrap())
}

/// Sets the API key for the OpenRouter client.
///
/// # Arguments
///
/// * `api_key` - The API key to set.
///
/// # Returns
///
/// * `Result<(), OpenRouterError>` - An empty result indicating success or an error.
///
/// # Example
///
/// ```
/// set_api_key("your_api_key").await?;
/// ```
pub async fn set_api_key(api_key: &str) -> Result<(), OpenRouterError> {
    config::api_key::store(api_key)?;
    let mut cached_api_key = DEFAULT_API_KEY.lock().await;
    *cached_api_key = Some(api_key.to_string());
    Ok(())
}

/// Sends a chat completion request.
///
/// # Arguments
///
/// * `request` - The chat completion request.
///
/// # Returns
///
/// * `Result<chat::ChatCompletionResponse, OpenRouterError>` - The response from the chat completion request.
///
/// # Example
///
/// ```
/// let request = chat::ChatCompletionRequest::builder()
///     .model("deepseek/deepseek-chat:free")
///     .messages(vec![chat::Message::new(chat::Role::User, "What is the meaning of life?")])
///     .max_tokens(100)
///     .temperature(0.7)
///     .build()?;
/// let response = send_chat_completion(request).await?;
/// println!("{:?}", response);
/// ```
pub async fn send_chat_completion(
    request: chat::ChatCompletionRequest,
) -> Result<chat::ChatCompletionResponse, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_default_api_key().await?;
    chat::send_chat_completion(&config.base_url, &api_key, &request).await
}

/// Streams chat completion events.
///
/// # Arguments
///
/// * `request` - The chat completion request.
///
/// # Returns
///
/// * `Result<BoxStream<'static, Result<chat::ChatCompletionStreamEvent, OpenRouterError>>, OpenRouterError>` - A stream of chat completion events or an error.
///
/// # Example
///
/// ```
/// let request = chat::ChatCompletionRequest::builder()
///     .model("deepseek/deepseek-chat:free")
///     .messages(vec![chat::Message::new(chat::Role::User, "Tell me a joke.")])
///     .max_tokens(50)
///     .temperature(0.5)
///     .build()?;
/// let mut stream = stream_chat_completion(request).await?;
/// while let Some(event) = stream.next().await {
///     match event {
///         Ok(event) => println!("{:?}", event),
///         Err(e) => eprintln!("Error: {:?}", e),
///     }
/// }
/// ```
pub async fn stream_chat_completion(
    request: chat::ChatCompletionRequest,
) -> Result<
    BoxStream<'static, Result<chat::ChatCompletionStreamEvent, OpenRouterError>>,
    OpenRouterError,
> {
    let config = get_config().await?;
    let api_key = get_default_api_key().await?;
    chat::stream_chat_completion(&config.base_url, &api_key, &request).await
}

/// Sends a completion request.
///
/// # Arguments
///
/// * `request` - The completion request.
///
/// # Returns
///
/// * `Result<completion::CompletionResponse, OpenRouterError>` - The response from the completion request.
///
/// # Example
///
/// ```
/// let request = completion::CompletionRequest::builder()
///     .model("deepseek/deepseek-chat:free")
///     .prompt("Once upon a time")
///     .max_tokens(100)
///     .temperature(0.7)
///     .build()?;
/// let response = send_completion(request).await?;
/// println!("{:?}", response);
/// ```
pub async fn send_completion(
    request: completion::CompletionRequest,
) -> Result<completion::CompletionResponse, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_default_api_key().await?;
    completion::send_completion_request(&config.base_url, &api_key, &request).await
}

/// Retrieves the total credits purchased and used for the authenticated user.
///
/// # Returns
///
/// * `Result<credits::CreditsData, OpenRouterError>` - The response data containing the total credits and usage.
///
/// # Example
///
/// ```
/// let credits_data = get_credits().await?;
/// println!("{:?}", credits_data);
/// ```
pub async fn get_credits() -> Result<credits::CreditsData, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_default_api_key().await?;
    credits::get_credits(&config.base_url, &api_key).await
}

/// Retrieves metadata about a specific generation request.
///
/// # Arguments
///
/// * `generation_id` - The ID of the generation request.
///
/// # Returns
///
/// * `Result<generation::GenerationData, OpenRouterError>` - The metadata of the generation request or an error.
///
/// # Example
///
/// ```
/// let generation_data = get_generation("generation_id").await?;
/// println!("{:?}", generation_data);
/// ```
pub async fn get_generation(
    generation_id: &str,
) -> Result<generation::GenerationData, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_default_api_key().await?;
    generation::get_generation(&config.base_url, &api_key, generation_id).await
}

/// Returns a list of models available through the API.
///
/// # Returns
///
/// * `Result<Vec<models::Model>, OpenRouterError>` - A list of models or an error.
///
/// # Example
///
/// ```
/// let models = list_models().await?;
/// println!("{:?}", models);
/// ```
pub async fn list_models() -> Result<Vec<models::Model>, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_default_api_key().await?;
    models::list_models(&config.base_url, &api_key).await
}

/// Returns details about the endpoints for a specific model.
///
/// # Arguments
///
/// * `author` - The author of the model.
/// * `slug` - The slug identifier for the model.
///
/// # Returns
///
/// * `Result<models::EndpointData, OpenRouterError>` - The endpoint data or an error.
///
/// # Example
///
/// ```
/// let endpoint_data = list_model_endpoints("author_name", "model_slug").await?;
/// println!("{:?}", endpoint_data);
/// ```
pub async fn list_model_endpoints(
    author: &str,
    slug: &str,
) -> Result<models::EndpointData, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_default_api_key().await?;
    models::list_model_endpoints(&config.base_url, &api_key, author, slug).await
}

/// Checks if a specific model is enabled.
///
/// # Arguments
///
/// * `model_id` - The ID of the model.
///
/// # Returns
///
/// * `Result<bool, OpenRouterError>` - A boolean indicating whether the model is enabled.
///
/// # Example
///
/// ```
/// let is_enabled = is_model_enabled("model_id").await?;
/// println!("Model enabled: {}", is_enabled);
/// ```
pub async fn is_model_enabled(model_id: &str) -> Result<bool, OpenRouterError> {
    let config = get_config().await?;
    Ok(config.models.is_enabled(model_id))
}

/// Retrieves information on the API key associated with the current authentication session.
///
/// # Returns
///
/// * `Result<api_keys::ApiKeyDetails, OpenRouterError>` - The details of the current API key.
///
/// # Example
///
/// ```
/// let api_key_details = get_current_api_key().await?;
/// println!("{:?}", api_key_details);
/// ```
pub async fn get_current_api_key() -> Result<api_keys::ApiKeyDetails, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_default_api_key().await?;
    api_keys::get_current_api_key(&config.base_url, &api_key).await
}

/// Returns a list of all API keys associated with the account. Requires a Provisioning API key.
///
/// # Arguments
///
/// * `offset` - Optional offset for the API keys.
/// * `include_disabled` - Optional flag to include disabled API keys.
///
/// # Returns
///
/// * `Result<Vec<api_keys::ApiKey>, OpenRouterError>` - A list of API keys.
///
/// # Example
///
/// ```
/// let api_keys = list_api_keys(Some(0.0), Some(true)).await?;
/// println!("{:?}", api_keys);
/// ```
pub async fn list_api_keys(
    offset: Option<f64>,
    include_disabled: Option<bool>,
) -> Result<Vec<api_keys::ApiKey>, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_provisioning_api_key().await?;
    api_keys::list_api_keys(&config.base_url, &api_key, offset, include_disabled).await
}

/// Creates a new API key. Requires a Provisioning API key.
///
/// # Arguments
///
/// * `name` - The display name for the new API key.
/// * `limit` - Optional credit limit for the new API key.
///
/// # Returns
///
/// * `Result<api_keys::ApiKey, OpenRouterError>` - The created API key.
///
/// # Example
///
/// ```
/// let api_key = create_api_key("New API Key", Some(100.0)).await?;
/// println!("{:?}", api_key);
/// ```
pub async fn create_api_key(
    name: &str,
    limit: Option<f64>,
) -> Result<api_keys::ApiKey, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_provisioning_api_key().await?;
    api_keys::create_api_key(&config.base_url, &api_key, name, limit).await
}

/// Returns details about a specific API key. Requires a Provisioning API key.
///
/// # Arguments
///
/// * `hash` - The hash of the API key to retrieve.
///
/// # Returns
///
/// * `Result<api_keys::ApiKey, OpenRouterError>` - The details of the specified API key.
///
/// # Example
///
/// ```
/// let api_key = get_api_key("api_key_hash").await?;
/// println!("{:?}", api_key);
/// ```
pub async fn get_api_key(hash: &str) -> Result<api_keys::ApiKey, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_provisioning_api_key().await?;
    api_keys::get_api_key(&config.base_url, &api_key, hash).await
}

/// Deletes an API key. Requires a Provisioning API key.
///
/// # Arguments
///
/// * `hash` - The hash of the API key to delete.
///
/// # Returns
///
/// * `Result<bool, OpenRouterError>` - A boolean indicating whether the deletion was successful.
///
/// # Example
///
/// ```
/// let success = delete_api_key("api_key_hash").await?;
/// println!("Deletion successful: {}", success);
/// ```
pub async fn delete_api_key(hash: &str) -> Result<bool, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_provisioning_api_key().await?;
    api_keys::delete_api_key(&config.base_url, &api_key, hash).await
}

/// Updates an existing API key. Requires a Provisioning API key.
///
/// # Arguments
///
/// * `hash` - The hash of the API key to update.
/// * `name` - Optional new display name for the API key.
/// * `disabled` - Optional flag to disable the API key.
/// * `limit` - Optional new credit limit for the API key.
///
/// # Returns
///
/// * `Result<api_keys::ApiKey, OpenRouterError>` - The updated API key.
///
/// # Example
///
/// ```
/// let updated_api_key = update_api_key("api_key_hash", Some("Updated Name".to_string()), Some(false), Some(200.0)).await?;
/// println!("{:?}", updated_api_key);
/// ```
pub async fn update_api_key(
    hash: &str,
    name: Option<String>,
    disabled: Option<bool>,
    limit: Option<f64>,
) -> Result<api_keys::ApiKey, OpenRouterError> {
    let config = get_config().await?;
    let api_key = get_provisioning_api_key().await?;
    api_keys::update_api_key(&config.base_url, &api_key, hash, name, disabled, limit).await
}
