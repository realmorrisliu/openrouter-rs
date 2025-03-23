use crate::api::{api_keys, auth, chat, completion, credits, generation, models};
use crate::error::OpenRouterError;
use reqwest::Client;

pub struct OpenRouterClient {
    client: Client,
    api_key: String,
}

impl OpenRouterClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
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
        api_keys::create_api_key(&self.client, &self.api_key, name, limit).await
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
        api_keys::get_current_api_key(&self.client, &self.api_key).await
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
        api_keys::delete_api_key(&self.client, &self.api_key, hash).await
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
        api_keys::update_api_key(&self.client, &self.api_key, hash, name, disabled, limit).await
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
        api_keys::list_api_keys(&self.client, &self.api_key, offset, include_disabled).await
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
        api_keys::get_api_key(&self.client, &self.api_key, hash).await
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
        auth::exchange_code_for_api_key(&self.client, code, code_verifier, code_challenge_method)
            .await
    }

    /// Send a chat completion request to a selected model.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request containing the model and messages.
    ///
    /// # Returns
    ///
    /// * `Result<chat::ChatCompletionResponse, OpenRouterError>` - The response from the chat completion request.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let messages = vec![chat::Message::new(chat::Role::User, "What is the meaning of life?")];
    /// let request = chat::ChatCompletionRequest::new("deepseek/deepseek-chat:free", messages)
    ///     .max_tokens(100)
    ///     .temperature(0.7);
    /// let response = client.send_chat_completion(&request).await?;
    /// println!("{:?}", response);
    /// ```
    pub async fn send_chat_completion(
        &self,
        request: &chat::ChatCompletionRequest,
    ) -> Result<chat::ChatCompletionResponse, OpenRouterError> {
        chat::send_chat_completion(&self.client, &self.api_key, request).await
    }

    /// Send a completion request to a selected model (text-only format).
    ///
    /// # Arguments
    ///
    /// * `request` - The completion request containing the model, prompt, and other optional parameters.
    ///
    /// # Returns
    ///
    /// * `Result<completion::CompletionResponse, OpenRouterError>` - The response from the completion request, containing the generated text and other details.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let completion_request = completion::CompletionRequest::new("deepseek/deepseek-chat:free", "Once upon a time")
    ///     .max_tokens(100)
    ///     .temperature(0.7);
    ///
    /// let completion_response = client.send_completion_request(&completion_request).await?;
    /// println!("{:?}", completion_response);
    /// ```
    pub async fn send_completion_request(
        &self,
        request: &completion::CompletionRequest,
    ) -> Result<completion::CompletionResponse, OpenRouterError> {
        completion::send_completion_request(&self.client, &self.api_key, request).await
    }

    /// Creates and hydrates a Coinbase Commerce charge for cryptocurrency payments.
    ///
    /// # Arguments
    ///
    /// * `request` - The request data for creating a Coinbase charge.
    ///
    /// # Returns
    ///
    /// * `Result<credits::CoinbaseChargeResponse, OpenRouterError>` - The response data containing the charge details.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let request = credits::CoinbaseChargeRequest::new(100.0, "sender_address", 1);
    /// let response = client.create_coinbase_charge(&request).await?;
    /// println!("{:?}", response);
    /// ```
    pub async fn create_coinbase_charge(
        &self,
        request: &credits::CoinbaseChargeRequest,
    ) -> Result<credits::CoinbaseChargeResponse, OpenRouterError> {
        credits::create_coinbase_charge(&self.client, &self.api_key, request).await
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
        credits::get_credits(&self.client, &self.api_key).await
    }

    /// Returns metadata about a specific generation request.
    ///
    /// # Arguments
    ///
    /// * `request` - The GenerationRequest containing the ID of the generation request.
    ///
    /// # Returns
    ///
    /// * `Result<generation::GenerationData, OpenRouterError>` - The metadata of the generation request or an error.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::new("your_api_key".to_string());
    /// let request = generation::GenerationRequest::new("generation_id");
    /// let generation_data = client.get_generation(&request).await?;
    /// println!("{:?}", generation_data);
    /// ```
    pub async fn get_generation(
        &self,
        request: &generation::GenerationRequest,
    ) -> Result<generation::GenerationData, OpenRouterError> {
        generation::get_generation(&self.client, &self.api_key, request).await
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
        models::list_models(&self.client, &self.api_key).await
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
        models::list_model_endpoints(&self.client, &self.api_key, author, slug).await
    }
}
