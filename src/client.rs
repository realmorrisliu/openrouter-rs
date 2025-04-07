use futures_util::stream::BoxStream;

use crate::{
    api::{
        api_keys, auth,
        chat::{self, ChatCompletionStreamEvent},
        completion, credits, generation, models,
    },
    error::OpenRouterError,
    types::completion::CompletionsResponse,
};

pub struct OpenRouterClient {
    base_url: String,
    api_key: String,
    http_referer: Option<String>,
    x_title: Option<String>,
}

pub struct OpenRouterClientBuilder {
    base_url: Option<String>,
    api_key: String,
    http_referer: Option<String>,
    x_title: Option<String>,
}

impl OpenRouterClientBuilder {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            base_url: None,
            api_key: api_key.into(),
            http_referer: None,
            x_title: None,
        }
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn http_referer(mut self, http_referer: impl Into<String>) -> Self {
        self.http_referer = Some(http_referer.into());
        self
    }

    pub fn x_title(mut self, x_title: impl Into<String>) -> Self {
        self.x_title = Some(x_title.into());
        self
    }

    pub fn build(self) -> OpenRouterClient {
        OpenRouterClient {
            base_url: self
                .base_url
                .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
            api_key: self.api_key,
            http_referer: self.http_referer,
            x_title: self.x_title,
        }
    }
}

impl OpenRouterClient {
    pub fn builder(api_key: impl Into<String>) -> OpenRouterClientBuilder {
        OpenRouterClientBuilder::new(api_key)
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
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let api_key = client.create_api_key("New API Key", Some(100.0)).await?;
    /// println!("{:?}", api_key);
    /// ```
    pub async fn create_api_key(
        &self,
        name: &str,
        limit: Option<f64>,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        api_keys::create_api_key(&self.base_url, &self.api_key, name, limit).await
    }

    /// Get information on the API key associated with the current authentication session.
    ///
    /// # Returns
    ///
    /// * `Result<api_keys::ApiKeyDetails, OpenRouterError>` - The details of the current API key.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let api_key_details = client.get_current_api_key_info().await?;
    /// println!("{:?}", api_key_details);
    /// ```
    pub async fn get_current_api_key_info(
        &self,
    ) -> Result<api_keys::ApiKeyDetails, OpenRouterError> {
        api_keys::get_current_api_key(&self.base_url, &self.api_key).await
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
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let success = client.delete_api_key("api_key_hash").await?;
    /// println!("Deletion successful: {}", success);
    /// ```
    pub async fn delete_api_key(&self, hash: &str) -> Result<bool, OpenRouterError> {
        api_keys::delete_api_key(&self.base_url, &self.api_key, hash).await
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
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let updated_api_key = client.update_api_key("api_key_hash", Some("Updated Name".to_string()), Some(false), Some(200.0)).await?;
    /// println!("{:?}", updated_api_key);
    /// ```
    pub async fn update_api_key(
        &self,
        hash: &str,
        name: Option<String>,
        disabled: Option<bool>,
        limit: Option<f64>,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        api_keys::update_api_key(&self.base_url, &self.api_key, hash, name, disabled, limit).await
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
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let api_keys = client.list_api_keys(Some(0.0), Some(true)).await?;
    /// println!("{:?}", api_keys);
    /// ```
    pub async fn list_api_keys(
        &self,
        offset: Option<f64>,
        include_disabled: Option<bool>,
    ) -> Result<Vec<api_keys::ApiKey>, OpenRouterError> {
        api_keys::list_api_keys(&self.base_url, &self.api_key, offset, include_disabled).await
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
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let api_key = client.get_api_key("api_key_hash").await?;
    /// println!("{:?}", api_key);
    /// ```
    pub async fn get_api_key(&self, hash: &str) -> Result<api_keys::ApiKey, OpenRouterError> {
        api_keys::get_api_key(&self.base_url, &self.api_key, hash).await
    }

    /// Exchange an authorization code from the PKCE flow for a user-controlled API key.
    ///
    /// # Arguments
    ///
    /// * `code` - The authorization code received from the OAuth redirect.
    /// * `code_verifier` - The code verifier if code_challenge was used in the authorization request.
    /// * `code_challenge_method` - The method used to generate the code challenge.
    ///
    /// # Returns
    ///
    /// * `Result<auth::AuthResponse, OpenRouterError>` - The API key and user ID associated with the API key.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let auth_response = client.exchange_code_for_api_key("auth_code", Some("code_verifier"), Some(auth::CodeChallengeMethod::S256)).await?;
    /// println!("{:?}", auth_response);
    /// ```
    pub async fn exchange_code_for_api_key(
        &self,
        code: &str,
        code_verifier: Option<&str>,
        code_challenge_method: Option<auth::CodeChallengeMethod>,
    ) -> Result<auth::AuthResponse, OpenRouterError> {
        auth::exchange_code_for_api_key(&self.base_url, code, code_verifier, code_challenge_method)
            .await
    }

    /// Send a chat completion request to a selected model.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request built using ChatCompletionRequest::builder().
    ///
    /// # Returns
    ///
    /// * `Result<chat::ChatCompletionResponse, OpenRouterError>` - The response from the chat completion request.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let request = chat::ChatCompletionRequest::builder()
    ///     .model("deepseek/deepseek-chat-v3-0324:free")
    ///     .messages(vec![chat::Message::new(chat::Role::User, "What is the meaning of life?")])
    ///     .max_tokens(100)
    ///     .temperature(0.7)
    ///     .build()?;
    /// let response = client.send_chat_completion(&request).await?;
    /// println!("{:?}", response);
    /// ```
    pub async fn send_chat_completion(
        &self,
        request: &chat::ChatCompletionRequest,
    ) -> Result<CompletionsResponse, OpenRouterError> {
        chat::send_chat_completion(
            &self.base_url,
            &self.api_key,
            &self.x_title,
            &self.http_referer,
            request,
        )
        .await
    }

    /// Streams chat completion events from a selected model.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request built using ChatCompletionRequest::builder().
    ///
    /// # Returns
    ///
    /// * `Result<BoxStream<'static, Result<ChatCompletionStreamEvent, OpenRouterError>>, OpenRouterError>` - A stream of chat completion events or an error.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let request = chat::ChatCompletionRequest::builder()
    ///     .model("deepseek/deepseek-chat-v3-0324:free")
    ///     .messages(vec![chat::Message::new(chat::Role::User, "Tell me a joke.")])
    ///     .max_tokens(50)
    ///     .temperature(0.5)
    ///     .build()?;
    /// let mut stream = client.stream_chat_completion(&request).await?;
    /// while let Some(event) = stream.next().await {
    ///     match event {
    ///         Ok(event) => println!("{:?}", event),
    ///         Err(e) => eprintln!("Error: {:?}", e),
    ///     }
    /// }
    /// ```
    pub async fn stream_chat_completion(
        &self,
        request: &chat::ChatCompletionRequest,
    ) -> Result<
        BoxStream<'static, Result<ChatCompletionStreamEvent, OpenRouterError>>,
        OpenRouterError,
    > {
        chat::stream_chat_completion(&self.base_url, &self.api_key, request).await
    }

    /// Send a completion request to a selected model (text-only format).
    ///
    /// # Arguments
    ///
    /// * `request` - The completion request built using CompletionRequest::builder().
    ///
    /// # Returns
    ///
    /// * `Result<completion::CompletionResponse, OpenRouterError>` - The response from the completion request, containing the generated text and other details.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let request = completion::CompletionRequest::builder()
    ///     .model("deepseek/deepseek-chat-v3-0324:free")
    ///     .prompt("Once upon a time")
    ///     .max_tokens(100)
    ///     .temperature(0.7)
    ///     .build()?;
    /// let response = client.send_completion_request(&request).await?;
    /// println!("{:?}", response);
    /// ```
    pub async fn send_completion_request(
        &self,
        request: &completion::CompletionRequest,
    ) -> Result<CompletionsResponse, OpenRouterError> {
        completion::send_completion_request(
            &self.base_url,
            &self.api_key,
            &self.x_title,
            &self.http_referer,
            request,
        )
        .await
    }

    /// Creates and hydrates a Coinbase Commerce charge for cryptocurrency payments.
    ///
    /// # Arguments
    ///
    /// * `request` - The request data built using CoinbaseChargeRequest::builder().
    ///
    /// # Returns
    ///
    /// * `Result<credits::CoinbaseChargeData, OpenRouterError>` - The response data containing the charge details.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let request = credits::CoinbaseChargeRequest::builder()
    ///     .amount(100.0)
    ///     .sender("sender_address")
    ///     .chain_id(1)
    ///     .build()?;
    /// let response = client.create_coinbase_charge(&request).await?;
    /// println!("{:?}", response);
    /// ```
    pub async fn create_coinbase_charge(
        &self,
        request: &credits::CoinbaseChargeRequest,
    ) -> Result<credits::CoinbaseChargeData, OpenRouterError> {
        credits::create_coinbase_charge(&self.base_url, &self.api_key, request).await
    }

    /// Returns the total credits purchased and used for the authenticated user.
    ///
    /// # Returns
    ///
    /// * `Result<credits::CreditsData, OpenRouterError>` - The response data containing the total credits and usage.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let credits_data = client.get_credits().await?;
    /// println!("{:?}", credits_data);
    /// ```
    pub async fn get_credits(&self) -> Result<credits::CreditsData, OpenRouterError> {
        credits::get_credits(&self.base_url, &self.api_key).await
    }

    /// Returns metadata about a specific generation request.
    ///
    /// # Arguments
    ///
    /// * `request` - The GenerationRequest built using GenerationRequest::builder().
    ///
    /// # Returns
    ///
    /// * `Result<generation::GenerationData, OpenRouterError>` - The metadata of the generation request or an error.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let request = generation::GenerationRequest::builder()
    ///     .id("generation_id")
    ///     .build()?;
    /// let generation_data = client.get_generation(&request).await?;
    /// println!("{:?}", generation_data);
    /// ```
    pub async fn get_generation(
        &self,
        id: impl Into<String>,
    ) -> Result<generation::GenerationData, OpenRouterError> {
        generation::get_generation(&self.base_url, &self.api_key, id).await
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
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let models = client.list_models().await?;
    /// println!("{:?}", models);
    /// ```
    pub async fn list_models(&self) -> Result<Vec<models::Model>, OpenRouterError> {
        models::list_models(&self.base_url, &self.api_key).await
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
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let endpoint_data = client.list_model_endpoints("author_name", "model_slug").await?;
    /// println!("{:?}", endpoint_data);
    /// ```
    pub async fn list_model_endpoints(
        &self,
        author: &str,
        slug: &str,
    ) -> Result<models::EndpointData, OpenRouterError> {
        models::list_model_endpoints(&self.base_url, &self.api_key, author, slug).await
    }
}
