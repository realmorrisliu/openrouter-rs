use derive_builder::Builder;
use futures_util::stream::BoxStream;

use crate::{
    api::{
        api_keys, auth, chat, completion, credits, embeddings, generation, messages, models,
        responses,
    },
    config::OpenRouterConfig,
    error::OpenRouterError,
    types::{
        ModelCategory, SupportedParameters, completion::CompletionsResponse,
        stream::ToolAwareStream,
    },
};

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct OpenRouterClient {
    #[builder(
        setter(into),
        default = "String::from(\"https://openrouter.ai/api/v1\")"
    )]
    base_url: String,
    #[builder(setter(into, strip_option), default)]
    api_key: Option<String>,
    #[builder(setter(into, strip_option), default)]
    provisioning_key: Option<String>,
    #[builder(setter(into, strip_option), default)]
    http_referer: Option<String>,
    #[builder(setter(into, strip_option), default)]
    x_title: Option<String>,
    #[builder(setter(into, strip_option), default)]
    config: Option<OpenRouterConfig>,
}

impl OpenRouterClient {
    pub fn builder() -> OpenRouterClientBuilder {
        OpenRouterClientBuilder::default()
    }

    pub fn get_config(&self) -> Option<OpenRouterConfig> {
        self.config.clone()
    }

    /// Sets the API key after client construction.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key to set
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = OpenRouterClient::builder().build();
    /// client.set_api_key("your_api_key");
    /// ```
    pub fn set_api_key(&mut self, api_key: impl Into<String>) {
        self.api_key = Some(api_key.into());
    }

    /// Clears the currently set API key.
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = OpenRouterClient::builder().api_key("your_api_key").build();
    /// client.clear_api_key();
    /// ```
    pub fn clear_api_key(&mut self) {
        self.api_key = None;
    }

    /// Sets the provisioning key after client construction.
    ///
    /// # Arguments
    ///
    /// * `provisioning_key` - The provisioning key to set
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = OpenRouterClient::builder().build();
    /// client.set_provisioning_key("your_provisioning_key");
    /// ```
    pub fn set_provisioning_key(&mut self, provisioning_key: impl Into<String>) {
        self.provisioning_key = Some(provisioning_key.into());
    }

    /// Clears the currently set provisioning key.
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = OpenRouterClient::builder().build();
    /// client.set_provisioning_key("your_provisioning_key");
    /// client.clear_provisioning_key();
    /// ```
    pub fn clear_provisioning_key(&mut self) {
        self.provisioning_key = None;
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
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
    /// let api_key = client.create_api_key("New API Key", Some(100.0)).await?;
    /// println!("{:?}", api_key);
    /// ```
    pub async fn create_api_key(
        &self,
        name: &str,
        limit: Option<f64>,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            api_keys::create_api_key(&self.base_url, api_key, name, limit).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
    /// let api_key_details = client.get_current_api_key_info().await?;
    /// println!("{:?}", api_key_details);
    /// ```
    pub async fn get_current_api_key_info(
        &self,
    ) -> Result<api_keys::ApiKeyDetails, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            api_keys::get_current_api_key(&self.base_url, api_key).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().provisioning_key("your_provisioning_key").build();
    /// let success = client.delete_api_key("api_key_hash").await?;
    /// println!("Deletion successful: {}", success);
    /// ```
    pub async fn delete_api_key(&self, hash: &str) -> Result<bool, OpenRouterError> {
        if let Some(provisioning_key) = &self.provisioning_key {
            api_keys::delete_api_key(&self.base_url, provisioning_key, hash).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().provisioning_key("your_provisioning_key").build();
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
        if let Some(provisioning_key) = &self.provisioning_key {
            api_keys::update_api_key(
                &self.base_url,
                provisioning_key,
                hash,
                name,
                disabled,
                limit,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().provisioning_key("your_provisioning_key").build();
    /// let api_keys = client.list_api_keys(Some(0.0), Some(true)).await?;
    /// println!("{:?}", api_keys);
    /// ```
    pub async fn list_api_keys(
        &self,
        offset: Option<f64>,
        include_disabled: Option<bool>,
    ) -> Result<Vec<api_keys::ApiKey>, OpenRouterError> {
        if let Some(provisioning_key) = &self.provisioning_key {
            api_keys::list_api_keys(&self.base_url, provisioning_key, offset, include_disabled)
                .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().provisioning_key("your_provisioning_key").build();
    /// let api_key = client.get_api_key("api_key_hash").await?;
    /// println!("{:?}", api_key);
    /// ```
    pub async fn get_api_key(&self, hash: &str) -> Result<api_keys::ApiKey, OpenRouterError> {
        if let Some(provisioning_key) = &self.provisioning_key {
            api_keys::get_api_key(&self.base_url, provisioning_key, hash).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
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
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
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
        if let Some(api_key) = &self.api_key {
            chat::send_chat_completion(
                &self.base_url,
                api_key,
                &self.x_title,
                &self.http_referer,
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Streams chat completion events from a selected model.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request built using ChatCompletionRequest::builder().
    ///
    /// # Returns
    ///
    /// * `Result<BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>, OpenRouterError>` - A stream of chat completion events or an error.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
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
    ) -> Result<BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>, OpenRouterError>
    {
        if let Some(api_key) = &self.api_key {
            chat::stream_chat_completion(&self.base_url, api_key, request).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Streams chat completion events with tool-call-aware processing.
    ///
    /// Returns a [`ToolAwareStream`] that yields [`StreamEvent`](crate::types::stream::StreamEvent)
    /// values. Content and reasoning deltas are forwarded immediately, while
    /// tool call fragments are accumulated internally and emitted as complete
    /// [`ToolCall`](crate::types::completion::ToolCall) objects in the final
    /// [`StreamEvent::Done`](crate::types::stream::StreamEvent::Done) event.
    ///
    /// This is the recommended way to stream responses when using tool calling.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request (should include tools).
    ///
    /// # Returns
    ///
    /// * `Result<ToolAwareStream, OpenRouterError>` - A stream of [`StreamEvent`](crate::types::stream::StreamEvent) values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures_util::StreamExt;
    /// use openrouter_rs::types::stream::StreamEvent;
    ///
    /// # async fn example(client: openrouter_rs::OpenRouterClient, request: openrouter_rs::api::chat::ChatCompletionRequest) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut stream = client.stream_chat_completion_tool_aware(&request).await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     match event {
    ///         StreamEvent::ContentDelta(text) => print!("{}", text),
    ///         StreamEvent::Done { tool_calls, .. } => {
    ///             for tc in &tool_calls {
    ///                 println!("Tool call: {}", tc.name());
    ///             }
    ///         },
    ///         _ => {}
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stream_chat_completion_tool_aware(
        &self,
        request: &chat::ChatCompletionRequest,
    ) -> Result<ToolAwareStream, OpenRouterError> {
        let raw_stream = self.stream_chat_completion(request).await?;
        Ok(ToolAwareStream::new(raw_stream))
    }

    /// Create a non-streaming response using the OpenRouter Responses API.
    ///
    /// # Arguments
    ///
    /// * `request` - The responses request built using `ResponsesRequest::builder()`.
    ///
    /// # Returns
    ///
    /// * `Result<responses::ResponsesResponse, OpenRouterError>` - The response payload.
    pub async fn create_response(
        &self,
        request: &responses::ResponsesRequest,
    ) -> Result<responses::ResponsesResponse, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            responses::create_response(
                &self.base_url,
                api_key,
                &self.x_title,
                &self.http_referer,
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Stream response events from the OpenRouter Responses API.
    ///
    /// # Arguments
    ///
    /// * `request` - The responses request built using `ResponsesRequest::builder()`.
    ///
    /// # Returns
    ///
    /// * `Result<BoxStream<'static, Result<responses::ResponsesStreamEvent, OpenRouterError>>, OpenRouterError>` - A stream of response events.
    pub async fn stream_response(
        &self,
        request: &responses::ResponsesRequest,
    ) -> Result<
        BoxStream<'static, Result<responses::ResponsesStreamEvent, OpenRouterError>>,
        OpenRouterError,
    > {
        if let Some(api_key) = &self.api_key {
            responses::stream_response(
                &self.base_url,
                api_key,
                &self.x_title,
                &self.http_referer,
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Create a non-streaming message using the Anthropic-compatible `/messages` API.
    pub async fn create_message(
        &self,
        request: &messages::AnthropicMessagesRequest,
    ) -> Result<messages::AnthropicMessagesResponse, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            messages::create_message(
                &self.base_url,
                api_key,
                &self.x_title,
                &self.http_referer,
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Stream SSE events from the Anthropic-compatible `/messages` API.
    pub async fn stream_messages(
        &self,
        request: &messages::AnthropicMessagesRequest,
    ) -> Result<
        BoxStream<'static, Result<messages::AnthropicMessagesSseEvent, OpenRouterError>>,
        OpenRouterError,
    > {
        if let Some(api_key) = &self.api_key {
            messages::stream_messages(
                &self.base_url,
                api_key,
                &self.x_title,
                &self.http_referer,
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
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
        if let Some(api_key) = &self.api_key {
            completion::send_completion_request(
                &self.base_url,
                api_key,
                &self.x_title,
                &self.http_referer,
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Submit an embeddings request.
    ///
    /// # Arguments
    ///
    /// * `request` - The embeddings request built using `EmbeddingRequest::builder()`.
    ///
    /// # Returns
    ///
    /// * `Result<embeddings::EmbeddingResponse, OpenRouterError>` - The embeddings response.
    pub async fn create_embedding(
        &self,
        request: &embeddings::EmbeddingRequest,
    ) -> Result<embeddings::EmbeddingResponse, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            embeddings::create_embedding(
                &self.base_url,
                api_key,
                &self.x_title,
                &self.http_referer,
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// List all available embeddings models.
    pub async fn list_embedding_models(&self) -> Result<Vec<models::Model>, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            embeddings::list_embedding_models(&self.base_url, api_key).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
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
        if let Some(api_key) = &self.api_key {
            credits::create_coinbase_charge(&self.base_url, api_key, request).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
    /// let credits_data = client.get_credits().await?;
    /// println!("{:?}", credits_data);
    /// ```
    pub async fn get_credits(&self) -> Result<credits::CreditsData, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            credits::get_credits(&self.base_url, api_key).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
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
        if let Some(api_key) = &self.api_key {
            generation::get_generation(&self.base_url, api_key, id).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
    /// let models = client.list_models().await?;
    /// println!("{:?}", models);
    /// ```
    pub async fn list_models(&self) -> Result<Vec<models::Model>, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            models::list_models(&self.base_url, api_key, None, None).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Returns a list of models available through the API by category.
    ///
    /// # Arguments
    ///
    /// * `category` - The category of the models.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<models::Model>, OpenRouterError>` - A list of models or an error.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
    /// let models = client.list_models_by_category(ModelCategory::TextCompletion).await?;
    /// println!("{:?}", models);
    /// ```
    pub async fn list_models_by_category(
        &self,
        category: ModelCategory,
    ) -> Result<Vec<models::Model>, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            models::list_models(&self.base_url, api_key, Some(category), None).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Returns a list of models available for the specified supported parameters.
    ///
    /// # Arguments
    ///
    /// * `supported_parameters` - The supported parameters for the models.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<models::Model>, OpenRouterError>` - A list of models or an error.
    ///
    /// # Example
    ///
    /// ```
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
    /// let models = client.list_models_by_parameters(SupportedParameters::Tools).await?;
    /// println!("{:?}", models);
    /// ```
    pub async fn list_models_by_parameters(
        &self,
        supported_parameters: SupportedParameters,
    ) -> Result<Vec<models::Model>, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            models::list_models(&self.base_url, api_key, None, Some(supported_parameters)).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
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
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build();
    /// let endpoint_data = client.list_model_endpoints("author_name", "model_slug").await?;
    /// println!("{:?}", endpoint_data);
    /// ```
    pub async fn list_model_endpoints(
        &self,
        author: &str,
        slug: &str,
    ) -> Result<models::EndpointData, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            models::list_model_endpoints(&self.base_url, api_key, author, slug).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }
}
