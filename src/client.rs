use derive_builder::Builder;
use futures_util::stream::BoxStream;

#[cfg(feature = "legacy-completions")]
use crate::api::legacy::completion;
use crate::{
    api::{
        api_keys, auth, chat, credits, discovery, embeddings, generation, guardrails, messages,
        models, responses,
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
    management_key: Option<String>,
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

    /// Sets the management key after client construction.
    ///
    /// # Arguments
    ///
    /// * `management_key` - The management key to set
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = OpenRouterClient::builder().build();
    /// client.set_management_key("your_management_key");
    /// ```
    pub fn set_management_key(&mut self, management_key: impl Into<String>) {
        self.management_key = Some(management_key.into());
    }

    /// Clears the currently set management key.
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = OpenRouterClient::builder().build();
    /// client.set_management_key("your_management_key");
    /// client.clear_management_key();
    /// ```
    pub fn clear_management_key(&mut self) {
        self.management_key = None;
    }

    /// Domain client for chat completions and chat streaming.
    pub fn chat(&self) -> ChatClient<'_> {
        ChatClient { client: self }
    }

    /// Domain client for Responses API operations.
    pub fn responses(&self) -> ResponsesClient<'_> {
        ResponsesClient { client: self }
    }

    /// Domain client for Anthropic-compatible `/messages` operations.
    pub fn messages(&self) -> MessagesClient<'_> {
        MessagesClient { client: self }
    }

    /// Domain client for model/discovery/embedding operations.
    pub fn models(&self) -> ModelsClient<'_> {
        ModelsClient { client: self }
    }

    /// Domain client for management-governed endpoints.
    pub fn management(&self) -> ManagementClient<'_> {
        ManagementClient { client: self }
    }

    /// Domain client for legacy endpoint access (`legacy-completions` feature).
    #[cfg(feature = "legacy-completions")]
    pub fn legacy(&self) -> LegacyClient<'_> {
        LegacyClient { client: self }
    }
}

#[doc(hidden)]
impl OpenRouterClient {
    /// Creates a new API key. Requires a management API key.
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
    /// let client = OpenRouterClient::builder().management_key("your_management_key").build();
    /// let api_key = client.create_api_key("New API Key", Some(100.0)).await?;
    /// println!("{:?}", api_key);
    /// ```
    pub async fn create_api_key(
        &self,
        name: &str,
        limit: Option<f64>,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            api_keys::create_api_key(&self.base_url, management_key, name, limit).await
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

    /// Deletes an API key. Requires a management API key.
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
    /// let client = OpenRouterClient::builder().management_key("your_management_key").build();
    /// let success = client.delete_api_key("api_key_hash").await?;
    /// println!("Deletion successful: {}", success);
    /// ```
    pub async fn delete_api_key(&self, hash: &str) -> Result<bool, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            api_keys::delete_api_key(&self.base_url, management_key, hash).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Updates an existing API key. Requires a management API key.
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
    /// let client = OpenRouterClient::builder().management_key("your_management_key").build();
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
        if let Some(management_key) = &self.management_key {
            api_keys::update_api_key(&self.base_url, management_key, hash, name, disabled, limit)
                .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Returns a list of all API keys associated with the account. Requires a management API key.
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
    /// let client = OpenRouterClient::builder().management_key("your_management_key").build();
    /// let api_keys = client.list_api_keys(Some(0.0), Some(true)).await?;
    /// println!("{:?}", api_keys);
    /// ```
    pub async fn list_api_keys(
        &self,
        offset: Option<f64>,
        include_disabled: Option<bool>,
    ) -> Result<Vec<api_keys::ApiKey>, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            api_keys::list_api_keys(&self.base_url, management_key, offset, include_disabled).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Returns details about a specific API key. Requires a management API key.
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
    /// let client = OpenRouterClient::builder().management_key("your_management_key").build();
    /// let api_key = client.get_api_key("api_key_hash").await?;
    /// println!("{:?}", api_key);
    /// ```
    pub async fn get_api_key(&self, hash: &str) -> Result<api_keys::ApiKey, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            api_keys::get_api_key(&self.base_url, management_key, hash).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Create an authorization code for PKCE flow (`POST /auth/keys/code`).
    ///
    /// # Arguments
    ///
    /// * `request` - The auth-code creation request built with `CreateAuthCodeRequest::builder()`.
    ///
    /// # Returns
    ///
    /// * `Result<auth::AuthCodeData, OpenRouterError>` - The created authorization code payload.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use openrouter_rs::{OpenRouterClient, api::auth};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build()?;
    ///
    /// let create = auth::CreateAuthCodeRequest::builder()
    ///     .callback_url("https://myapp.com/auth/callback")
    ///     .code_challenge("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM")
    ///     .code_challenge_method(auth::CodeChallengeMethod::S256)
    ///     .build()?;
    ///
    /// let auth_code = client.create_auth_code(&create).await?;
    ///
    /// let exchanged = client
    ///     .exchange_code_for_api_key(
    ///         &auth_code.id,
    ///         Some("your_pkce_code_verifier"),
    ///         Some(auth::CodeChallengeMethod::S256),
    ///     )
    ///     .await?;
    ///
    /// println!("New key: {}", exchanged.key);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_auth_code(
        &self,
        request: &auth::CreateAuthCodeRequest,
    ) -> Result<auth::AuthCodeData, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            auth::create_auth_code(&self.base_url, api_key, request).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// List guardrails (`GET /guardrails`). Requires a management key.
    pub async fn list_guardrails(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<guardrails::GuardrailListResponse, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::list_guardrails(&self.base_url, management_key, offset, limit).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Create a guardrail (`POST /guardrails`). Requires a management key.
    pub async fn create_guardrail(
        &self,
        request: &guardrails::CreateGuardrailRequest,
    ) -> Result<guardrails::Guardrail, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::create_guardrail(&self.base_url, management_key, request).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Get a guardrail by ID (`GET /guardrails/{id}`). Requires a management key.
    pub async fn get_guardrail(&self, id: &str) -> Result<guardrails::Guardrail, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::get_guardrail(&self.base_url, management_key, id).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Update a guardrail (`PATCH /guardrails/{id}`). Requires a management key.
    pub async fn update_guardrail(
        &self,
        id: &str,
        request: &guardrails::UpdateGuardrailRequest,
    ) -> Result<guardrails::Guardrail, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::update_guardrail(&self.base_url, management_key, id, request).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Delete a guardrail (`DELETE /guardrails/{id}`). Requires a management key.
    pub async fn delete_guardrail(&self, id: &str) -> Result<bool, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::delete_guardrail(&self.base_url, management_key, id).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// List key assignments for a guardrail (`GET /guardrails/{id}/assignments/keys`).
    pub async fn list_guardrail_key_assignments(
        &self,
        id: &str,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<guardrails::GuardrailKeyAssignmentsResponse, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::list_guardrail_key_assignments(
                &self.base_url,
                management_key,
                id,
                offset,
                limit,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Bulk assign key hashes to a guardrail (`POST /guardrails/{id}/assignments/keys`).
    pub async fn bulk_assign_keys_to_guardrail(
        &self,
        id: &str,
        request: &guardrails::BulkKeyAssignmentRequest,
    ) -> Result<guardrails::AssignedCountResponse, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::bulk_assign_keys_to_guardrail(&self.base_url, management_key, id, request)
                .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Bulk unassign key hashes from a guardrail (`POST /guardrails/{id}/assignments/keys/remove`).
    pub async fn bulk_unassign_keys_from_guardrail(
        &self,
        id: &str,
        request: &guardrails::BulkKeyAssignmentRequest,
    ) -> Result<guardrails::UnassignedCountResponse, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::bulk_unassign_keys_from_guardrail(
                &self.base_url,
                management_key,
                id,
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// List member assignments for a guardrail (`GET /guardrails/{id}/assignments/members`).
    pub async fn list_guardrail_member_assignments(
        &self,
        id: &str,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<guardrails::GuardrailMemberAssignmentsResponse, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::list_guardrail_member_assignments(
                &self.base_url,
                management_key,
                id,
                offset,
                limit,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Bulk assign members to a guardrail (`POST /guardrails/{id}/assignments/members`).
    pub async fn bulk_assign_members_to_guardrail(
        &self,
        id: &str,
        request: &guardrails::BulkMemberAssignmentRequest,
    ) -> Result<guardrails::AssignedCountResponse, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::bulk_assign_members_to_guardrail(
                &self.base_url,
                management_key,
                id,
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Bulk unassign members from a guardrail (`POST /guardrails/{id}/assignments/members/remove`).
    pub async fn bulk_unassign_members_from_guardrail(
        &self,
        id: &str,
        request: &guardrails::BulkMemberAssignmentRequest,
    ) -> Result<guardrails::UnassignedCountResponse, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::bulk_unassign_members_from_guardrail(
                &self.base_url,
                management_key,
                id,
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// List all key assignments (`GET /guardrails/assignments/keys`). Requires a management key.
    pub async fn list_key_assignments(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<guardrails::GuardrailKeyAssignmentsResponse, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::list_key_assignments(&self.base_url, management_key, offset, limit).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// List all member assignments (`GET /guardrails/assignments/members`). Requires a management key.
    pub async fn list_member_assignments(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<guardrails::GuardrailMemberAssignmentsResponse, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            guardrails::list_member_assignments(&self.base_url, management_key, offset, limit).await
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

    /// Send a legacy completion request to a selected model (text-only format).
    ///
    /// # Arguments
    ///
    /// * `request` - The completion request built using CompletionRequest::builder().
    ///
    /// # Returns
    ///
    /// * `Result<completion::CompletionsResponse, OpenRouterError>` - The response from the completion request, containing the generated text and other details.
    ///
    /// # Example
    ///
    /// ```
    /// use openrouter_rs::api::legacy::completion::CompletionRequest;
    ///
    /// let client = OpenRouterClient::builder().api_key("your_api_key").build()?;
    /// let request = CompletionRequest::builder()
    ///     .model("deepseek/deepseek-chat-v3-0324:free")
    ///     .prompt("Once upon a time")
    ///     .max_tokens(100)
    ///     .temperature(0.7)
    ///     .build()?;
    /// let response = client.legacy().completions().create(&request).await?;
    /// println!("{:?}", response);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "legacy-completions")]
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

    /// List all providers.
    ///
    /// This endpoint is public, but this SDK method still requires `api_key`
    /// for consistency with other client operations.
    pub async fn list_providers(&self) -> Result<Vec<discovery::Provider>, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            discovery::list_providers(&self.base_url, api_key).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// List models filtered by user provider preferences, privacy settings, and guardrails.
    ///
    /// Equivalent to `GET /models/user`.
    pub async fn list_models_for_user(&self) -> Result<Vec<discovery::UserModel>, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            discovery::list_models_for_user(&self.base_url, api_key).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Get the total number of available models.
    ///
    /// Equivalent to `GET /models/count`.
    pub async fn count_models(&self) -> Result<discovery::ModelsCountData, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            discovery::count_models(&self.base_url, api_key).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Preview ZDR-compatible endpoints.
    ///
    /// Equivalent to `GET /endpoints/zdr`.
    pub async fn list_zdr_endpoints(
        &self,
    ) -> Result<Vec<discovery::PublicEndpoint>, OpenRouterError> {
        if let Some(api_key) = &self.api_key {
            discovery::list_zdr_endpoints(&self.base_url, api_key).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }

    /// Get activity grouped by endpoint for the last 30 UTC days.
    ///
    /// Equivalent to `GET /activity`.
    ///
    /// Requires a management API key. In this SDK, configure that via
    /// `OpenRouterClientBuilder::management_key(...)`.
    ///
    /// `date` is optional and should be `YYYY-MM-DD`.
    pub async fn get_activity(
        &self,
        date: Option<&str>,
    ) -> Result<Vec<discovery::ActivityItem>, OpenRouterError> {
        if let Some(management_key) = &self.management_key {
            discovery::get_activity(&self.base_url, management_key, date).await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }
}

/// Domain client for chat completions.
#[derive(Debug, Clone, Copy)]
pub struct ChatClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> ChatClient<'a> {
    /// Create a chat completion (`POST /chat/completions`).
    pub async fn create(
        &self,
        request: &chat::ChatCompletionRequest,
    ) -> Result<CompletionsResponse, OpenRouterError> {
        self.client.send_chat_completion(request).await
    }

    /// Stream chat completion chunks.
    pub async fn stream(
        &self,
        request: &chat::ChatCompletionRequest,
    ) -> Result<BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>, OpenRouterError>
    {
        self.client.stream_chat_completion(request).await
    }

    /// Stream chat completion chunks with tool-call-aware aggregation.
    pub async fn stream_tool_aware(
        &self,
        request: &chat::ChatCompletionRequest,
    ) -> Result<ToolAwareStream, OpenRouterError> {
        self.client.stream_chat_completion_tool_aware(request).await
    }
}

/// Domain client for OpenRouter Responses API.
#[derive(Debug, Clone, Copy)]
pub struct ResponsesClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> ResponsesClient<'a> {
    /// Create a response (`POST /responses`).
    pub async fn create(
        &self,
        request: &responses::ResponsesRequest,
    ) -> Result<responses::ResponsesResponse, OpenRouterError> {
        self.client.create_response(request).await
    }

    /// Stream response events (`POST /responses`, `stream=true`).
    pub async fn stream(
        &self,
        request: &responses::ResponsesRequest,
    ) -> Result<
        BoxStream<'static, Result<responses::ResponsesStreamEvent, OpenRouterError>>,
        OpenRouterError,
    > {
        self.client.stream_response(request).await
    }
}

/// Domain client for Anthropic-compatible Messages API.
#[derive(Debug, Clone, Copy)]
pub struct MessagesClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> MessagesClient<'a> {
    /// Create a non-streaming message (`POST /messages`).
    pub async fn create(
        &self,
        request: &messages::AnthropicMessagesRequest,
    ) -> Result<messages::AnthropicMessagesResponse, OpenRouterError> {
        self.client.create_message(request).await
    }

    /// Stream SSE events from `/messages`.
    pub async fn stream(
        &self,
        request: &messages::AnthropicMessagesRequest,
    ) -> Result<
        BoxStream<'static, Result<messages::AnthropicMessagesSseEvent, OpenRouterError>>,
        OpenRouterError,
    > {
        self.client.stream_messages(request).await
    }
}

/// Domain client for model/discovery/embedding endpoints.
#[derive(Debug, Clone, Copy)]
pub struct ModelsClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> ModelsClient<'a> {
    /// List all models (`GET /models`).
    pub async fn list(&self) -> Result<Vec<models::Model>, OpenRouterError> {
        self.client.list_models().await
    }

    /// List models by category (`GET /models?category=...`).
    pub async fn list_by_category(
        &self,
        category: ModelCategory,
    ) -> Result<Vec<models::Model>, OpenRouterError> {
        self.client.list_models_by_category(category).await
    }

    /// List models by supported parameter (`GET /models?supported_parameters=...`).
    pub async fn list_by_parameters(
        &self,
        supported_parameters: SupportedParameters,
    ) -> Result<Vec<models::Model>, OpenRouterError> {
        self.client
            .list_models_by_parameters(supported_parameters)
            .await
    }

    /// List model endpoints (`GET /models/{author}/{slug}/endpoints`).
    pub async fn list_endpoints(
        &self,
        author: &str,
        slug: &str,
    ) -> Result<models::EndpointData, OpenRouterError> {
        self.client.list_model_endpoints(author, slug).await
    }

    /// List providers (`GET /providers`).
    pub async fn list_providers(&self) -> Result<Vec<discovery::Provider>, OpenRouterError> {
        self.client.list_providers().await
    }

    /// List user-filtered models (`GET /models/user`).
    pub async fn list_for_user(&self) -> Result<Vec<discovery::UserModel>, OpenRouterError> {
        self.client.list_models_for_user().await
    }

    /// Count available models (`GET /models/count`).
    pub async fn count(&self) -> Result<discovery::ModelsCountData, OpenRouterError> {
        self.client.count_models().await
    }

    /// List ZDR-compatible endpoints (`GET /endpoints/zdr`).
    pub async fn list_zdr_endpoints(
        &self,
    ) -> Result<Vec<discovery::PublicEndpoint>, OpenRouterError> {
        self.client.list_zdr_endpoints().await
    }

    /// Create an embedding (`POST /embeddings`).
    pub async fn create_embedding(
        &self,
        request: &embeddings::EmbeddingRequest,
    ) -> Result<embeddings::EmbeddingResponse, OpenRouterError> {
        self.client.create_embedding(request).await
    }

    /// List embedding models (`GET /embeddings/models`).
    pub async fn list_embedding_models(&self) -> Result<Vec<models::Model>, OpenRouterError> {
        self.client.list_embedding_models().await
    }
}

/// Domain client for management endpoints.
#[derive(Debug, Clone, Copy)]
pub struct ManagementClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> ManagementClient<'a> {
    /// Create a managed API key (`POST /keys`).
    pub async fn create_api_key(
        &self,
        name: &str,
        limit: Option<f64>,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        self.client.create_api_key(name, limit).await
    }

    /// Get current key session info (`GET /key`).
    pub async fn get_current_api_key_info(
        &self,
    ) -> Result<api_keys::ApiKeyDetails, OpenRouterError> {
        self.client.get_current_api_key_info().await
    }

    /// Delete an API key (`DELETE /keys/{hash}`).
    pub async fn delete_api_key(&self, hash: &str) -> Result<bool, OpenRouterError> {
        self.client.delete_api_key(hash).await
    }

    /// Update an API key (`PATCH /keys/{hash}`).
    pub async fn update_api_key(
        &self,
        hash: &str,
        name: Option<String>,
        disabled: Option<bool>,
        limit: Option<f64>,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        self.client
            .update_api_key(hash, name, disabled, limit)
            .await
    }

    /// List API keys (`GET /keys`).
    pub async fn list_api_keys(
        &self,
        offset: Option<f64>,
        include_disabled: Option<bool>,
    ) -> Result<Vec<api_keys::ApiKey>, OpenRouterError> {
        self.client.list_api_keys(offset, include_disabled).await
    }

    /// Get an API key (`GET /keys/{hash}`).
    pub async fn get_api_key(&self, hash: &str) -> Result<api_keys::ApiKey, OpenRouterError> {
        self.client.get_api_key(hash).await
    }

    /// Create OAuth auth code (`POST /auth/keys/code`).
    pub async fn create_auth_code(
        &self,
        request: &auth::CreateAuthCodeRequest,
    ) -> Result<auth::AuthCodeData, OpenRouterError> {
        self.client.create_auth_code(request).await
    }

    /// Exchange auth code for API key (`POST /auth/keys`).
    pub async fn exchange_code_for_api_key(
        &self,
        code: &str,
        code_verifier: Option<&str>,
        code_challenge_method: Option<auth::CodeChallengeMethod>,
    ) -> Result<auth::AuthResponse, OpenRouterError> {
        self.client
            .exchange_code_for_api_key(code, code_verifier, code_challenge_method)
            .await
    }

    /// Create a Coinbase charge (`POST /credits/coinbase`).
    pub async fn create_coinbase_charge(
        &self,
        request: &credits::CoinbaseChargeRequest,
    ) -> Result<credits::CoinbaseChargeData, OpenRouterError> {
        self.client.create_coinbase_charge(request).await
    }

    /// Get credits (`GET /credits`).
    pub async fn get_credits(&self) -> Result<credits::CreditsData, OpenRouterError> {
        self.client.get_credits().await
    }

    /// Get generation metadata (`GET /generation?id=...`).
    pub async fn get_generation(
        &self,
        id: impl Into<String>,
    ) -> Result<generation::GenerationData, OpenRouterError> {
        self.client.get_generation(id).await
    }

    /// Get endpoint usage activity (`GET /activity`).
    pub async fn get_activity(
        &self,
        date: Option<&str>,
    ) -> Result<Vec<discovery::ActivityItem>, OpenRouterError> {
        self.client.get_activity(date).await
    }

    /// List guardrails (`GET /guardrails`).
    pub async fn list_guardrails(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<guardrails::GuardrailListResponse, OpenRouterError> {
        self.client.list_guardrails(offset, limit).await
    }

    /// Create a guardrail (`POST /guardrails`).
    pub async fn create_guardrail(
        &self,
        request: &guardrails::CreateGuardrailRequest,
    ) -> Result<guardrails::Guardrail, OpenRouterError> {
        self.client.create_guardrail(request).await
    }

    /// Get a guardrail (`GET /guardrails/{id}`).
    pub async fn get_guardrail(&self, id: &str) -> Result<guardrails::Guardrail, OpenRouterError> {
        self.client.get_guardrail(id).await
    }

    /// Update a guardrail (`PATCH /guardrails/{id}`).
    pub async fn update_guardrail(
        &self,
        id: &str,
        request: &guardrails::UpdateGuardrailRequest,
    ) -> Result<guardrails::Guardrail, OpenRouterError> {
        self.client.update_guardrail(id, request).await
    }

    /// Delete a guardrail (`DELETE /guardrails/{id}`).
    pub async fn delete_guardrail(&self, id: &str) -> Result<bool, OpenRouterError> {
        self.client.delete_guardrail(id).await
    }

    /// List key assignments for a guardrail.
    pub async fn list_guardrail_key_assignments(
        &self,
        id: &str,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<guardrails::GuardrailKeyAssignmentsResponse, OpenRouterError> {
        self.client
            .list_guardrail_key_assignments(id, offset, limit)
            .await
    }

    /// Bulk assign key hashes to a guardrail.
    pub async fn bulk_assign_keys_to_guardrail(
        &self,
        id: &str,
        request: &guardrails::BulkKeyAssignmentRequest,
    ) -> Result<guardrails::AssignedCountResponse, OpenRouterError> {
        self.client.bulk_assign_keys_to_guardrail(id, request).await
    }

    /// Bulk unassign key hashes from a guardrail.
    pub async fn bulk_unassign_keys_from_guardrail(
        &self,
        id: &str,
        request: &guardrails::BulkKeyAssignmentRequest,
    ) -> Result<guardrails::UnassignedCountResponse, OpenRouterError> {
        self.client
            .bulk_unassign_keys_from_guardrail(id, request)
            .await
    }

    /// List member assignments for a guardrail.
    pub async fn list_guardrail_member_assignments(
        &self,
        id: &str,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<guardrails::GuardrailMemberAssignmentsResponse, OpenRouterError> {
        self.client
            .list_guardrail_member_assignments(id, offset, limit)
            .await
    }

    /// Bulk assign members to a guardrail.
    pub async fn bulk_assign_members_to_guardrail(
        &self,
        id: &str,
        request: &guardrails::BulkMemberAssignmentRequest,
    ) -> Result<guardrails::AssignedCountResponse, OpenRouterError> {
        self.client
            .bulk_assign_members_to_guardrail(id, request)
            .await
    }

    /// Bulk unassign members from a guardrail.
    pub async fn bulk_unassign_members_from_guardrail(
        &self,
        id: &str,
        request: &guardrails::BulkMemberAssignmentRequest,
    ) -> Result<guardrails::UnassignedCountResponse, OpenRouterError> {
        self.client
            .bulk_unassign_members_from_guardrail(id, request)
            .await
    }

    /// List global key assignments.
    pub async fn list_key_assignments(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<guardrails::GuardrailKeyAssignmentsResponse, OpenRouterError> {
        self.client.list_key_assignments(offset, limit).await
    }

    /// List global member assignments.
    pub async fn list_member_assignments(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<guardrails::GuardrailMemberAssignmentsResponse, OpenRouterError> {
        self.client.list_member_assignments(offset, limit).await
    }
}

/// Domain client for legacy APIs (`legacy-completions` feature only).
#[cfg(feature = "legacy-completions")]
#[derive(Debug, Clone, Copy)]
pub struct LegacyClient<'a> {
    client: &'a OpenRouterClient,
}

#[cfg(feature = "legacy-completions")]
impl<'a> LegacyClient<'a> {
    /// Domain client for legacy text completions (`POST /completions`).
    pub fn completions(&self) -> LegacyCompletionsClient<'a> {
        LegacyCompletionsClient {
            client: self.client,
        }
    }
}

/// Domain client for legacy text completions (`legacy-completions` feature only).
#[cfg(feature = "legacy-completions")]
#[derive(Debug, Clone, Copy)]
pub struct LegacyCompletionsClient<'a> {
    client: &'a OpenRouterClient,
}

#[cfg(feature = "legacy-completions")]
impl<'a> LegacyCompletionsClient<'a> {
    /// Create a legacy text completion (`POST /completions`).
    pub async fn create(
        &self,
        request: &completion::CompletionRequest,
    ) -> Result<CompletionsResponse, OpenRouterError> {
        self.client.send_completion_request(request).await
    }
}
