use derive_builder::Builder;
use futures_util::stream::BoxStream;

#[cfg(feature = "legacy-completions")]
use crate::api::legacy::completion;
use crate::{
    api::{
        analytics, api_keys, audio, auth, batches, byok, chat, credits, decisions, discovery,
        embeddings, external_users, files, generation, guardrails, images, messages, models,
        observability, organization, presets, private_providers, rerank, responses, scim, vault,
        videos, workspaces,
    },
    error::OpenRouterError,
    strip_option_vec_setter,
    types::{
        ModelCategory, PaginationOptions, SupportedParameters,
        completion::CompletionsResponse,
        stream::{UnifiedStream, adapt_chat_stream, adapt_messages_stream, adapt_responses_stream},
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
    #[builder(
        setter(into, strip_option),
        default = "Some(String::from(\"openrouter-rs\"))"
    )]
    x_title: Option<String>,
    #[builder(setter(custom), default)]
    app_categories: Option<Vec<String>>,
    #[builder(setter(custom), default = "crate::transport::new_client()?")]
    http_client: crate::transport::HttpClient,
}

impl OpenRouterClient {
    pub fn builder() -> OpenRouterClientBuilder {
        OpenRouterClientBuilder::default()
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
    /// # use openrouter_rs::OpenRouterClient;
    /// let mut client = OpenRouterClient::builder().build()?;
    /// client.set_api_key("your_api_key");
    /// # Ok::<(), openrouter_rs::error::OpenRouterError>(())
    /// ```
    pub fn set_api_key(&mut self, api_key: impl Into<String>) {
        self.api_key = Some(api_key.into());
    }

    /// Clears the currently set API key.
    ///
    /// # Example
    ///
    /// ```
    /// # use openrouter_rs::OpenRouterClient;
    /// let mut client = OpenRouterClient::builder().api_key("your_api_key").build()?;
    /// client.clear_api_key();
    /// # Ok::<(), openrouter_rs::error::OpenRouterError>(())
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
    /// # use openrouter_rs::OpenRouterClient;
    /// let mut client = OpenRouterClient::builder().build()?;
    /// client.set_management_key("your_management_key");
    /// # Ok::<(), openrouter_rs::error::OpenRouterError>(())
    /// ```
    pub fn set_management_key(&mut self, management_key: impl Into<String>) {
        self.management_key = Some(management_key.into());
    }

    /// Clears the currently set management key.
    ///
    /// # Example
    ///
    /// ```
    /// # use openrouter_rs::OpenRouterClient;
    /// let mut client = OpenRouterClient::builder().build()?;
    /// client.set_management_key("your_management_key");
    /// client.clear_management_key();
    /// # Ok::<(), openrouter_rs::error::OpenRouterError>(())
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

    /// Domain client for rerank operations.
    pub fn rerank(&self) -> RerankClient<'_> {
        RerankClient { client: self }
    }

    /// Domain client for audio operations.
    pub fn audio(&self) -> AudioClient<'_> {
        AudioClient { client: self }
    }

    /// Domain client for image generation operations.
    pub fn images(&self) -> ImagesClient<'_> {
        ImagesClient { client: self }
    }

    /// Domain client for text-to-speech operations.
    #[deprecated(note = "use client.audio().speech()")]
    pub fn tts(&self) -> SpeechClient<'_> {
        self.audio().speech()
    }

    /// Domain client for video generation operations.
    pub fn videos(&self) -> VideosClient<'_> {
        VideosClient { client: self }
    }

    /// Domain client for file upload, metadata, content, and deletion operations.
    pub fn files(&self) -> FilesClient<'_> {
        FilesClient { client: self }
    }

    /// Domain client for model/discovery/embedding operations.
    pub fn models(&self) -> ModelsClient<'_> {
        ModelsClient { client: self }
    }

    /// Domain client for management-governed endpoints.
    pub fn management(&self) -> ManagementClient<'_> {
        ManagementClient { client: self }
    }

    /// Domain client for asynchronous batch operations.
    pub fn batches(&self) -> BatchesClient<'_> {
        BatchesClient { client: self }
    }

    /// Domain client for workspace and intern vault secrets.
    pub fn vault(&self) -> VaultClient<'_> {
        VaultClient { client: self }
    }

    /// Domain client for Decisions and System One requests.
    pub fn decisions(&self) -> DecisionsClient<'_> {
        DecisionsClient { client: self }
    }

    /// Domain client for legacy endpoint access (`legacy-completions` feature).
    #[cfg(feature = "legacy-completions")]
    pub fn legacy(&self) -> LegacyClient<'_> {
        LegacyClient { client: self }
    }

    pub(crate) fn http_client(&self) -> &crate::transport::HttpClient {
        &self.http_client
    }

    pub(crate) fn request_context<'a>(
        &'a self,
        credential: &'a str,
    ) -> crate::transport::RequestContext<'a> {
        crate::transport::RequestContext {
            http_client: &self.http_client,
            base_url: &self.base_url,
            credential,
            x_title: &self.x_title,
            http_referer: &self.http_referer,
            app_categories: &self.app_categories,
        }
    }

    fn require_api_key(&self) -> Result<&str, OpenRouterError> {
        self.api_key
            .as_deref()
            .ok_or(OpenRouterError::KeyNotConfigured)
    }
}

impl OpenRouterClientBuilder {
    strip_option_vec_setter!(app_categories, String);

    /// Sends requests through a caller-configured `reqwest::Client` (feature `reqwest`).
    #[cfg(feature = "reqwest")]
    pub fn http_client<VALUE: Into<reqwest::Client>>(&mut self, value: VALUE) -> &mut Self {
        self.http_client = Some(crate::transport::HttpClient::Reqwest(value.into()));
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BatchesClient<'a> {
    client: &'a OpenRouterClient,
}

impl BatchesClient<'_> {
    pub async fn list(
        &self,
        params: &batches::ListBatchesParams,
    ) -> Result<batches::ListBatchesResponse, OpenRouterError> {
        batches::list_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            params,
        )
        .await
    }
    pub async fn create(
        &self,
        request: &batches::CreateBatchRequest,
    ) -> Result<batches::Batch, OpenRouterError> {
        batches::create_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }
    pub async fn get(&self, batch_id: &str) -> Result<batches::Batch, OpenRouterError> {
        batches::get_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            batch_id,
        )
        .await
    }
    pub async fn delete(
        &self,
        batch_id: &str,
    ) -> Result<batches::DeleteBatchResponse, OpenRouterError> {
        batches::delete_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            batch_id,
        )
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VaultClient<'a> {
    client: &'a OpenRouterClient,
}

impl VaultClient<'_> {
    pub async fn list(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<vault::VaultSecretListResponse, OpenRouterError> {
        vault::list_secrets_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            None,
            limit,
            offset,
        )
        .await
    }

    pub async fn list_for_intern(
        &self,
        intern_id: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<vault::VaultSecretListResponse, OpenRouterError> {
        vault::list_secrets_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            Some(intern_id),
            limit,
            offset,
        )
        .await
    }

    pub async fn store(
        &self,
        name: &str,
        request: &vault::VaultSecretWriteRequest,
    ) -> Result<vault::VaultSecretResponse, OpenRouterError> {
        vault::store_secret_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            None,
            name,
            request,
        )
        .await
    }

    pub async fn store_for_intern(
        &self,
        intern_id: &str,
        name: &str,
        request: &vault::VaultSecretWriteRequest,
    ) -> Result<vault::VaultSecretResponse, OpenRouterError> {
        vault::store_secret_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            Some(intern_id),
            name,
            request,
        )
        .await
    }

    pub async fn delete(&self, name: &str) -> Result<(), OpenRouterError> {
        vault::delete_secret_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            None,
            name,
        )
        .await
    }

    pub async fn delete_for_intern(
        &self,
        intern_id: &str,
        name: &str,
    ) -> Result<(), OpenRouterError> {
        vault::delete_secret_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            Some(intern_id),
            name,
        )
        .await
    }

    /// List the connections in scope for an intern.
    pub async fn list_connections_for_intern(
        &self,
        intern_id: &str,
    ) -> Result<vault::VaultInternConnectionListResponse, OpenRouterError> {
        vault::list_connections_for_intern_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            intern_id,
        )
        .await
    }

    pub async fn copy_to_intern(
        &self,
        intern_id: &str,
        request: &vault::VaultSecretCopyRequest,
    ) -> Result<vault::VaultSecretCopyResponse, OpenRouterError> {
        vault::copy_secrets_to_intern_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            intern_id,
            request,
        )
        .await
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DecisionsClient<'a> {
    client: &'a OpenRouterClient,
}

impl DecisionsClient<'_> {
    pub async fn create(
        &self,
        request: &decisions::DecisionsRequest,
    ) -> Result<decisions::DecisionsResponse, OpenRouterError> {
        decisions::create_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    pub async fn create_system_one(
        &self,
        request: &decisions::DecisionsRequest,
    ) -> Result<decisions::DecisionsResponse, OpenRouterError> {
        decisions::create_system_one_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
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
        chat::send_chat_completion_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// Stream chat completion chunks.
    pub async fn stream(
        &self,
        request: &chat::ChatCompletionRequest,
    ) -> Result<BoxStream<'static, Result<CompletionsResponse, OpenRouterError>>, OpenRouterError>
    {
        chat::stream_chat_completion_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// Stream chat events through the unified event model.
    ///
    /// The terminal event carries the fully assembled tool calls.
    pub async fn stream_unified(
        &self,
        request: &chat::ChatCompletionRequest,
    ) -> Result<UnifiedStream, OpenRouterError> {
        let raw = chat::stream_chat_completion_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await?;
        Ok(adapt_chat_stream(raw))
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
        responses::create_response_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// Stream response events (`POST /responses`, `stream=true`).
    pub async fn stream(
        &self,
        request: &responses::ResponsesRequest,
    ) -> Result<
        BoxStream<'static, Result<responses::ResponsesStreamEvent, OpenRouterError>>,
        OpenRouterError,
    > {
        responses::stream_response_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// Stream response events through the unified event model.
    pub async fn stream_unified(
        &self,
        request: &responses::ResponsesRequest,
    ) -> Result<UnifiedStream, OpenRouterError> {
        let raw = responses::stream_response_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await?;
        Ok(adapt_responses_stream(raw))
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
        messages::create_message_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// Stream SSE events from `/messages`.
    pub async fn stream(
        &self,
        request: &messages::AnthropicMessagesRequest,
    ) -> Result<
        BoxStream<'static, Result<messages::AnthropicMessagesSseEvent, OpenRouterError>>,
        OpenRouterError,
    > {
        messages::stream_messages_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// Stream messages events through the unified event model.
    pub async fn stream_unified(
        &self,
        request: &messages::AnthropicMessagesRequest,
    ) -> Result<UnifiedStream, OpenRouterError> {
        let raw = messages::stream_messages_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await?;
        Ok(adapt_messages_stream(raw))
    }
}

/// Domain client for rerank endpoints.
#[derive(Debug, Clone, Copy)]
pub struct RerankClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> RerankClient<'a> {
    /// Create a rerank result set (`POST /rerank`).
    pub async fn create(
        &self,
        request: &rerank::RerankRequest,
    ) -> Result<rerank::RerankResponse, OpenRouterError> {
        rerank::create_rerank_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }
}

/// Domain client for audio endpoints.
#[derive(Debug, Clone, Copy)]
pub struct AudioClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> AudioClient<'a> {
    /// Domain client for speech generation operations.
    pub fn speech(&self) -> SpeechClient<'a> {
        SpeechClient {
            client: self.client,
        }
    }

    /// Domain client for audio transcription operations.
    pub fn transcriptions(&self) -> TranscriptionsClient<'a> {
        TranscriptionsClient {
            client: self.client,
        }
    }
}

/// Domain client for audio speech endpoints.
#[derive(Debug, Clone, Copy)]
pub struct SpeechClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> SpeechClient<'a> {
    /// Create speech audio bytes (`POST /audio/speech`).
    pub async fn create(&self, request: &audio::SpeechRequest) -> Result<Vec<u8>, OpenRouterError> {
        audio::create_speech_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }
}

#[deprecated(note = "use SpeechClient")]
pub type TtsClient<'a> = SpeechClient<'a>;

/// Domain client for audio transcription endpoints.
#[derive(Debug, Clone, Copy)]
pub struct TranscriptionsClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> TranscriptionsClient<'a> {
    /// Create an audio transcription (`POST /audio/transcriptions`).
    pub async fn create(
        &self,
        request: &audio::TranscriptionRequest,
    ) -> Result<audio::TranscriptionResponse, OpenRouterError> {
        audio::create_transcription_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }
}

/// Domain client for image generation endpoints.
#[derive(Debug, Clone, Copy)]
pub struct ImagesClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> ImagesClient<'a> {
    /// Submit an image generation request (`POST /images`).
    pub async fn create(
        &self,
        request: &images::ImageGenerationRequest,
    ) -> Result<images::ImageGenerationResponse, OpenRouterError> {
        images::create_image_generation_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// Stream image generation events (`POST /images`, `stream=true`).
    pub async fn stream(
        &self,
        request: &images::ImageGenerationRequest,
    ) -> Result<
        BoxStream<'static, Result<images::ImageStreamingResponse, OpenRouterError>>,
        OpenRouterError,
    > {
        images::stream_image_generation_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// List available image generation models (`GET /images/models`).
    pub async fn list_models(&self) -> Result<Vec<images::ImageModel>, OpenRouterError> {
        images::list_image_models_with_client(
            &self.client.request_context(self.client.require_api_key()?),
        )
        .await
    }

    /// List provider endpoints for one image generation model.
    pub async fn list_model_endpoints(
        &self,
        author: &str,
        slug: &str,
    ) -> Result<images::ImageModelEndpointsResponse, OpenRouterError> {
        images::list_image_model_endpoints_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            author,
            slug,
        )
        .await
    }
}

/// Domain client for video generation endpoints.
#[derive(Debug, Clone, Copy)]
pub struct VideosClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> VideosClient<'a> {
    /// Submit a video generation request (`POST /videos`).
    pub async fn create(
        &self,
        request: &videos::VideoGenerationRequest,
    ) -> Result<videos::VideoGenerationResponse, OpenRouterError> {
        videos::create_video_generation_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// List available video generation models (`GET /videos/models`).
    pub async fn list_models(&self) -> Result<Vec<videos::VideoModel>, OpenRouterError> {
        videos::list_video_models_with_client(
            &self.client.request_context(self.client.require_api_key()?),
        )
        .await
    }

    /// Poll a submitted video generation (`GET /videos/{jobId}`).
    pub async fn get_generation(
        &self,
        job_id: &str,
    ) -> Result<videos::VideoGenerationResponse, OpenRouterError> {
        videos::get_video_generation_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            job_id,
        )
        .await
    }

    /// Download video output bytes for a completed job (`GET /videos/{jobId}/content`).
    pub async fn get_content(
        &self,
        job_id: &str,
        index: Option<u32>,
    ) -> Result<Vec<u8>, OpenRouterError> {
        videos::get_video_content_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            job_id,
            index,
        )
        .await
    }
}

/// Domain client for file endpoints.
#[derive(Debug, Clone, Copy)]
pub struct FilesClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> FilesClient<'a> {
    /// POST `/containers/{}/files/{}/promote`.
    pub async fn promote_container_file(
        &self,
        container_id: &str,
        file_id: &str,
    ) -> Result<files::ProviderFileMetadata, OpenRouterError> {
        files::promote_container_file_with_client(
            &self.client.request_context(self.api_key()?),
            container_id,
            file_id,
        )
        .await
    }

    /// GET `/containers/{}/files/{}/content`.
    pub async fn download_container_file(
        &self,
        container_id: &str,
        file_id: &str,
    ) -> Result<Vec<u8>, OpenRouterError> {
        files::download_container_file_with_client(
            &self.client.request_context(self.api_key()?),
            container_id,
            file_id,
        )
        .await
    }

    /// GET `/containers/{}/files/{}`.
    pub async fn get_container_file(
        &self,
        container_id: &str,
        file_id: &str,
    ) -> Result<files::ContainerFile, OpenRouterError> {
        files::get_container_file_with_client(
            &self.client.request_context(self.api_key()?),
            container_id,
            file_id,
        )
        .await
    }

    /// GET `/containers/{}/files`.
    pub async fn list_container_files(
        &self,
        container_id: &str,
        after: Option<&str>,
        limit: Option<u32>,
    ) -> Result<files::ContainerFileListResponse, OpenRouterError> {
        files::list_container_files_with_client(
            &self.client.request_context(self.api_key()?),
            container_id,
            after,
            limit,
        )
        .await
    }

    fn api_key(&self) -> Result<&str, OpenRouterError> {
        self.client
            .api_key
            .as_deref()
            .ok_or(OpenRouterError::KeyNotConfigured)
    }

    /// List files (`GET /files`).
    pub async fn list(
        &self,
        limit: Option<u32>,
        cursor: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<files::FileListResponse, OpenRouterError> {
        files::list_files_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            limit,
            cursor,
            workspace_id,
        )
        .await
    }

    /// Upload a file (`POST /files`).
    pub async fn upload(
        &self,
        request: &files::UploadFileRequest,
        workspace_id: Option<&str>,
    ) -> Result<files::FileMetadata, OpenRouterError> {
        files::upload_file_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
            workspace_id,
        )
        .await
    }

    /// Get file metadata (`GET /files/{file_id}`).
    pub async fn get_metadata(
        &self,
        file_id: &str,
        workspace_id: Option<&str>,
    ) -> Result<files::FileMetadata, OpenRouterError> {
        files::get_file_metadata_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            file_id,
            workspace_id,
        )
        .await
    }

    /// Download file content (`GET /files/{file_id}/content`).
    pub async fn download_content(
        &self,
        file_id: &str,
        workspace_id: Option<&str>,
    ) -> Result<Vec<u8>, OpenRouterError> {
        files::download_file_content_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            file_id,
            workspace_id,
        )
        .await
    }

    /// Delete a file (`DELETE /files/{file_id}`).
    pub async fn delete(
        &self,
        file_id: &str,
        workspace_id: Option<&str>,
    ) -> Result<files::FileDeleteResponse, OpenRouterError> {
        files::delete_file_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            file_id,
            workspace_id,
        )
        .await
    }

    /// List files in the response shape negotiated with a backing provider.
    pub async fn list_for_provider(
        &self,
        params: &files::ListFilesParams,
    ) -> Result<files::ProviderFileListResponse, OpenRouterError> {
        files::list_provider_files_with_client(
            &self.client.request_context(self.api_key()?),
            params,
        )
        .await
    }

    /// Upload a file to a selected backing provider.
    pub async fn upload_for_provider(
        &self,
        request: &files::UploadFileRequest,
        query: &files::FileQuery,
    ) -> Result<files::ProviderFileMetadata, OpenRouterError> {
        files::upload_provider_file_with_client(
            &self.client.request_context(self.api_key()?),
            request,
            query,
        )
        .await
    }

    /// Get file metadata in a provider-native response shape.
    pub async fn get_metadata_for_provider(
        &self,
        file_id: &str,
        query: &files::FileQuery,
    ) -> Result<files::ProviderFileMetadata, OpenRouterError> {
        files::get_provider_file_metadata_with_client(
            &self.client.request_context(self.api_key()?),
            file_id,
            query,
        )
        .await
    }

    /// Download file content from a selected backing provider.
    pub async fn download_content_for_provider(
        &self,
        file_id: &str,
        query: &files::FileQuery,
    ) -> Result<Vec<u8>, OpenRouterError> {
        files::download_provider_file_content_with_client(
            &self.client.request_context(self.api_key()?),
            file_id,
            query,
        )
        .await
    }

    /// Delete a file from a selected backing provider.
    pub async fn delete_for_provider(
        &self,
        file_id: &str,
        query: &files::FileQuery,
    ) -> Result<files::ProviderFileDeleteResponse, OpenRouterError> {
        files::delete_provider_file_with_client(
            &self.client.request_context(self.api_key()?),
            file_id,
            query,
        )
        .await
    }
}

/// Domain client for model/discovery/embedding endpoints.
#[derive(Debug, Clone, Copy)]
pub struct ModelsClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> ModelsClient<'a> {
    /// List user models with pagination and output modality filters.
    pub async fn list_for_user_with_params(
        &self,
        params: &discovery::ListUserModelsParams,
    ) -> Result<Vec<discovery::UserModel>, OpenRouterError> {
        let api_key = self
            .client
            .api_key
            .as_deref()
            .ok_or(OpenRouterError::KeyNotConfigured)?;
        discovery::list_models_for_user_with_params_with_client(
            &self.client.request_context(api_key),
            params,
        )
        .await
    }

    /// List all models (`GET /models`).
    pub async fn list(&self) -> Result<Vec<models::Model>, OpenRouterError> {
        models::list_models_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            None,
            None,
        )
        .await
    }

    /// List models with the extended filter surface (`GET /models?...`).
    pub async fn list_filtered(
        &self,
        params: Option<&models::ListModelsParams>,
    ) -> Result<Vec<models::Model>, OpenRouterError> {
        models::list_models_with_params_and_client(
            &self.client.request_context(self.client.require_api_key()?),
            params,
        )
        .await
    }

    /// List models by category (`GET /models?category=...`).
    pub async fn list_by_category(
        &self,
        category: ModelCategory,
    ) -> Result<Vec<models::Model>, OpenRouterError> {
        models::list_models_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            Some(category),
            None,
        )
        .await
    }

    /// List models by supported parameter (`GET /models?supported_parameters=...`).
    pub async fn list_by_parameters(
        &self,
        supported_parameters: SupportedParameters,
    ) -> Result<Vec<models::Model>, OpenRouterError> {
        models::list_models_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            None,
            Some(supported_parameters),
        )
        .await
    }

    /// List model endpoints (`GET /models/{author}/{slug}/endpoints`).
    pub async fn list_endpoints(
        &self,
        author: &str,
        slug: &str,
    ) -> Result<models::EndpointData, OpenRouterError> {
        models::list_model_endpoints_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            author,
            slug,
        )
        .await
    }

    /// Get metadata about one model (`GET /model/{author}/{slug}`).
    pub async fn get(&self, author: &str, slug: &str) -> Result<models::Model, OpenRouterError> {
        models::get_model_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            author,
            slug,
        )
        .await
    }

    /// List providers (`GET /providers`).
    pub async fn list_providers(&self) -> Result<Vec<discovery::Provider>, OpenRouterError> {
        discovery::list_providers_with_client(
            &self.client.request_context(self.client.require_api_key()?),
        )
        .await
    }

    /// List user-filtered models (`GET /models/user`).
    pub async fn list_user_models(&self) -> Result<Vec<discovery::UserModel>, OpenRouterError> {
        discovery::list_models_for_user_with_client(
            &self.client.request_context(self.client.require_api_key()?),
        )
        .await
    }

    /// Get available model count (`GET /models/count`).
    pub async fn get_model_count(&self) -> Result<discovery::ModelsCountData, OpenRouterError> {
        discovery::count_models_with_client(
            &self.client.request_context(self.client.require_api_key()?),
        )
        .await
    }

    /// Return daily token totals for top public models (`GET /datasets/rankings-daily`).
    pub async fn get_rankings_daily(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<discovery::RankingsDailyResponse, OpenRouterError> {
        discovery::get_rankings_daily_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            start_date,
            end_date,
        )
        .await
    }

    /// Return daily rankings using all upstream filters.
    pub async fn get_rankings_daily_filtered(
        &self,
        params: Option<&discovery::RankingsDailyParams>,
    ) -> Result<discovery::RankingsDailyResponse, OpenRouterError> {
        discovery::get_rankings_daily_with_params_and_client(
            &self.client.request_context(self.client.require_api_key()?),
            params,
        )
        .await
    }

    /// Return aggregated cost-per-session cells (`GET /datasets/session-cost`).
    pub async fn get_session_cost(
        &self,
        params: Option<&discovery::SessionCostParams>,
    ) -> Result<discovery::SessionCostResponse, OpenRouterError> {
        discovery::get_session_cost_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            params,
        )
        .await
    }

    /// Return ranked applications (`GET /datasets/app-rankings`).
    pub async fn get_app_rankings(
        &self,
        params: Option<&discovery::AppRankingsParams>,
    ) -> Result<discovery::AppRankingsResponse, OpenRouterError> {
        discovery::get_app_rankings_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            params,
        )
        .await
    }

    /// Return task classification market-share data (`GET /classifications/task`).
    pub async fn get_task_classifications(
        &self,
        window: Option<&str>,
    ) -> Result<discovery::TaskClassificationsResponse, OpenRouterError> {
        discovery::get_task_classifications_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            window,
        )
        .await
    }

    /// Return benchmark rows from a selected source (`GET /benchmarks`).
    pub async fn get_benchmarks(
        &self,
        params: &discovery::UnifiedBenchmarksParams,
    ) -> Result<discovery::UnifiedBenchmarksResponse, OpenRouterError> {
        discovery::get_benchmarks_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            params,
        )
        .await
    }

    /// Return Artificial Analysis benchmark rows.
    #[deprecated(note = "use get_benchmarks with source `artificial-analysis`")]
    #[allow(deprecated)]
    pub async fn get_benchmarks_artificial_analysis(
        &self,
        max_results: Option<u32>,
    ) -> Result<discovery::BenchmarksAAResponse, OpenRouterError> {
        discovery::get_benchmarks_artificial_analysis_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            max_results,
        )
        .await
    }

    /// Return Design Arena benchmark rows.
    #[deprecated(note = "use get_benchmarks with source `design-arena`")]
    #[allow(deprecated)]
    pub async fn get_benchmarks_design_arena(
        &self,
        arena: Option<&str>,
        category: Option<&str>,
        max_results: Option<u32>,
    ) -> Result<discovery::BenchmarksDAResponse, OpenRouterError> {
        discovery::get_benchmarks_design_arena_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            arena,
            category,
            max_results,
        )
        .await
    }

    /// List ZDR-compatible endpoints (`GET /endpoints/zdr`).
    pub async fn list_zdr_endpoints(
        &self,
    ) -> Result<Vec<discovery::PublicEndpoint>, OpenRouterError> {
        discovery::list_zdr_endpoints_with_client(
            &self.client.request_context(self.client.require_api_key()?),
        )
        .await
    }

    /// Create an embedding (`POST /embeddings`).
    pub async fn create_embedding(
        &self,
        request: &embeddings::EmbeddingRequest,
    ) -> Result<embeddings::EmbeddingResponse, OpenRouterError> {
        embeddings::create_embedding_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// List embedding models (`GET /embeddings/models`).
    pub async fn list_embedding_models(&self) -> Result<Vec<models::Model>, OpenRouterError> {
        embeddings::list_embedding_models_with_client(
            &self.client.request_context(self.client.require_api_key()?),
        )
        .await
    }
}

/// Domain client for management endpoints.
#[derive(Debug, Clone, Copy)]
pub struct ManagementClient<'a> {
    client: &'a OpenRouterClient,
}

impl<'a> ManagementClient<'a> {
    /// Create an API key with external identity and workspace options.
    pub async fn create_api_key_with_options(
        &self,
        request: &api_keys::CreateApiKeyRequest,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        api_keys::create_api_key_with_options_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    /// Fetch public OAuth signing keys. Does not require a management key.
    pub async fn get_oauth_jwks(&self) -> Result<auth::OAuthJwksResponse, OpenRouterError> {
        auth::get_oauth_jwks_with_client(self.client.http_client(), &self.client.base_url).await
    }
    /// Exchange a workload JWT. Authentication is supplied by the subject token.
    pub async fn exchange_oauth_token(
        &self,
        request: &auth::OAuthTokenExchangeRequest,
    ) -> Result<auth::OAuthTokenResponse, OpenRouterError> {
        // Token exchange authenticates with the workload JWT, not a credential header.
        auth::exchange_oauth_token_with_client(&self.client.request_context(""), request).await
    }

    /// GET `/scim/sync-jobs/{}`.
    pub async fn get_scim_sync_job(&self, id: &str) -> Result<scim::ScimSyncJob, OpenRouterError> {
        scim::get_scim_sync_job_with_client(
            &self.client.request_context(self.management_key()?),
            id,
        )
        .await
    }

    /// POST `/scim/sync-jobs`.
    pub async fn create_scim_sync_job(&self) -> Result<scim::ScimSyncJob, OpenRouterError> {
        scim::create_scim_sync_job_with_client(&self.client.request_context(self.management_key()?))
            .await
    }

    /// GET `/private-providers`.
    pub async fn list_private_providers(
        &self,
    ) -> Result<Vec<private_providers::PrivateProvider>, OpenRouterError> {
        private_providers::list_private_providers_with_client(
            &self.client.request_context(self.management_key()?),
        )
        .await
    }

    /// POST `/private-providers`.
    pub async fn create_private_provider(
        &self,
        request: &private_providers::CreatePrivateProviderRequest,
    ) -> Result<private_providers::PrivateProvider, OpenRouterError> {
        private_providers::create_private_provider_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    /// GET `/private-providers/{slug}`.
    pub async fn get_private_provider(
        &self,
        slug: &str,
    ) -> Result<private_providers::PrivateProvider, OpenRouterError> {
        private_providers::get_private_provider_with_client(
            &self.client.request_context(self.management_key()?),
            slug,
        )
        .await
    }

    /// PATCH `/private-providers/{slug}`.
    pub async fn update_private_provider(
        &self,
        slug: &str,
        request: &private_providers::UpdatePrivateProviderRequest,
    ) -> Result<private_providers::PrivateProvider, OpenRouterError> {
        private_providers::update_private_provider_with_client(
            &self.client.request_context(self.management_key()?),
            slug,
            request,
        )
        .await
    }

    /// DELETE `/private-providers/{slug}`.
    pub async fn delete_private_provider(
        &self,
        slug: &str,
    ) -> Result<private_providers::DeletedPrivateProvider, OpenRouterError> {
        private_providers::delete_private_provider_with_client(
            &self.client.request_context(self.management_key()?),
            slug,
        )
        .await
    }

    /// GET `/external-users`.
    pub async fn list_external_users(
        &self,
        params: &external_users::ListExternalUsersParams,
    ) -> Result<external_users::ListExternalUsersResponse, OpenRouterError> {
        external_users::list_external_users_with_client(
            &self.client.request_context(self.management_key()?),
            params,
        )
        .await
    }

    /// POST `/external-users`.
    pub async fn create_external_user(
        &self,
        request: &external_users::CreateExternalUserRequest,
    ) -> Result<external_users::ExternalUser, OpenRouterError> {
        external_users::create_external_user_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    /// GET `/external-users/{user}`.
    pub async fn get_external_user(
        &self,
        user: &str,
    ) -> Result<external_users::ExternalUser, OpenRouterError> {
        external_users::get_external_user_with_client(
            &self.client.request_context(self.management_key()?),
            user,
        )
        .await
    }

    /// PATCH `/external-users/{user}`.
    pub async fn update_external_user(
        &self,
        user: &str,
        request: &external_users::UpdateExternalUserRequest,
    ) -> Result<external_users::ExternalUser, OpenRouterError> {
        external_users::update_external_user_with_client(
            &self.client.request_context(self.management_key()?),
            user,
            request,
        )
        .await
    }

    /// DELETE `/external-users/{user}`. Soft-deactivates registration; does not block inference.
    pub async fn delete_external_user(&self, user: &str) -> Result<(), OpenRouterError> {
        external_users::delete_external_user_with_client(
            &self.client.request_context(self.management_key()?),
            user,
        )
        .await
    }

    /// GET `/organization/settings`.
    pub async fn get_organization_settings(
        &self,
    ) -> Result<organization::OrganizationSettings, OpenRouterError> {
        organization::get_organization_settings_with_client(
            &self.client.request_context(self.management_key()?),
        )
        .await
    }

    /// PATCH `/organization/settings`.
    pub async fn update_organization_settings(
        &self,
        request: &organization::UpdateOrganizationSettingsRequest,
    ) -> Result<organization::OrganizationSettings, OpenRouterError> {
        organization::update_organization_settings_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    fn management_key(&self) -> Result<&str, OpenRouterError> {
        self.client
            .management_key
            .as_deref()
            .ok_or(OpenRouterError::KeyNotConfigured)
    }

    /// List SCIM groups (`GET /scim/groups`).
    pub async fn list_scim_groups(
        &self,
        pagination: Option<PaginationOptions>,
    ) -> Result<scim::ListScimGroupsResponse, OpenRouterError> {
        scim::list_scim_groups_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
        )
        .await
    }

    pub async fn list_scim_groups_with_params(
        &self,
        params: &scim::ListScimGroupsParams,
    ) -> Result<scim::ListScimGroupsResponse, OpenRouterError> {
        scim::list_scim_groups_with_params_with_client(
            &self.client.request_context(self.management_key()?),
            params,
        )
        .await
    }

    /// List SCIM group mappings (`GET /scim/group-mappings`).
    pub async fn list_scim_group_mappings(
        &self,
        pagination: Option<PaginationOptions>,
    ) -> Result<scim::ListScimGroupMappingsResponse, OpenRouterError> {
        scim::list_scim_group_mappings_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
        )
        .await
    }

    /// Create a SCIM group mapping (`POST /scim/group-mappings`).
    pub async fn create_scim_group_mapping(
        &self,
        request: &scim::CreateScimGroupMappingRequest,
    ) -> Result<scim::ScimGroupMapping, OpenRouterError> {
        scim::create_scim_group_mapping_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    /// Get a SCIM group mapping (`GET /scim/group-mappings/{id}`).
    pub async fn get_scim_group_mapping(
        &self,
        id: &str,
    ) -> Result<scim::ScimGroupMapping, OpenRouterError> {
        scim::get_scim_group_mapping_with_client(
            &self.client.request_context(self.management_key()?),
            id,
        )
        .await
    }

    /// Update a SCIM group mapping (`PATCH /scim/group-mappings/{id}`).
    pub async fn update_scim_group_mapping(
        &self,
        id: &str,
        request: &scim::UpdateScimGroupMappingRequest,
    ) -> Result<scim::ScimGroupMapping, OpenRouterError> {
        scim::update_scim_group_mapping_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// Delete a SCIM group mapping (`DELETE /scim/group-mappings/{id}`).
    pub async fn delete_scim_group_mapping(
        &self,
        id: &str,
        keep_members: bool,
    ) -> Result<bool, OpenRouterError> {
        scim::delete_scim_group_mapping_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            keep_members,
        )
        .await
    }

    /// Create a managed API key (`POST /keys`).
    pub async fn create_api_key(
        &self,
        name: &str,
        limit: Option<f64>,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        api_keys::create_api_key_with_client(
            &self.client.request_context(self.management_key()?),
            name,
            limit,
        )
        .await
    }

    /// Create a managed API key in a workspace (`POST /keys`).
    pub async fn create_api_key_in_workspace(
        &self,
        name: &str,
        limit: Option<f64>,
        workspace_id: Option<&str>,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        api_keys::create_api_key_in_workspace_with_client(
            &self.client.request_context(self.management_key()?),
            name,
            limit,
            workspace_id,
        )
        .await
    }

    /// Get current key session info (`GET /key`).
    pub async fn get_current_api_key_info(
        &self,
    ) -> Result<api_keys::ApiKeyDetails, OpenRouterError> {
        api_keys::get_current_api_key_with_client(
            &self.client.request_context(self.client.require_api_key()?),
        )
        .await
    }

    /// Create or update a preset from a chat-completions request body.
    pub async fn create_chat_completion_preset(
        &self,
        slug: &str,
        request: &chat::ChatCompletionRequest,
    ) -> Result<presets::PresetWithDesignatedVersion, OpenRouterError> {
        presets::create_chat_completion_preset_with_client(
            &self.client.request_context(self.management_key()?),
            slug,
            request,
        )
        .await
    }

    /// Create or update a preset from a Responses API request body.
    pub async fn create_response_preset(
        &self,
        slug: &str,
        request: &responses::ResponsesRequest,
    ) -> Result<presets::PresetWithDesignatedVersion, OpenRouterError> {
        presets::create_response_preset_with_client(
            &self.client.request_context(self.management_key()?),
            slug,
            request,
        )
        .await
    }

    /// Create or update a preset from an Anthropic-compatible Messages request body.
    pub async fn create_message_preset(
        &self,
        slug: &str,
        request: &messages::AnthropicMessagesRequest,
    ) -> Result<presets::PresetWithDesignatedVersion, OpenRouterError> {
        presets::create_message_preset_with_client(
            &self.client.request_context(self.management_key()?),
            slug,
            request,
        )
        .await
    }

    /// List presets (`GET /presets`).
    pub async fn list_presets(
        &self,
        pagination: Option<PaginationOptions>,
    ) -> Result<presets::ListPresetsResponse, OpenRouterError> {
        presets::list_presets_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
        )
        .await
    }

    /// Get one preset (`GET /presets/{slug}`).
    pub async fn get_preset(
        &self,
        slug: &str,
    ) -> Result<presets::PresetWithDesignatedVersion, OpenRouterError> {
        presets::get_preset_with_client(&self.client.request_context(self.management_key()?), slug)
            .await
    }

    /// List preset versions (`GET /presets/{slug}/versions`).
    pub async fn list_preset_versions(
        &self,
        slug: &str,
        pagination: Option<PaginationOptions>,
    ) -> Result<presets::ListPresetVersionsResponse, OpenRouterError> {
        presets::list_preset_versions_with_client(
            &self.client.request_context(self.management_key()?),
            slug,
            pagination,
        )
        .await
    }

    /// Get a preset version (`GET /presets/{slug}/versions/{version}`).
    pub async fn get_preset_version(
        &self,
        slug: &str,
        version: &str,
    ) -> Result<presets::PresetDesignatedVersion, OpenRouterError> {
        presets::get_preset_version_with_client(
            &self.client.request_context(self.management_key()?),
            slug,
            version,
        )
        .await
    }

    /// Delete an API key (`DELETE /keys/{hash}`).
    pub async fn delete_api_key(&self, hash: &str) -> Result<bool, OpenRouterError> {
        api_keys::delete_api_key_with_client(
            &self.client.request_context(self.management_key()?),
            hash,
        )
        .await
    }

    /// Update an API key (`PATCH /keys/{hash}`).
    pub async fn update_api_key(
        &self,
        hash: &str,
        name: Option<String>,
        disabled: Option<bool>,
        limit: Option<f64>,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        api_keys::update_api_key_with_client(
            &self.client.request_context(self.management_key()?),
            hash,
            name,
            disabled,
            limit,
        )
        .await
    }

    /// List API keys with expired-key and workspace filters.
    pub async fn list_api_keys_with_params(
        &self,
        params: &api_keys::ListApiKeysParams,
    ) -> Result<Vec<api_keys::ApiKey>, OpenRouterError> {
        api_keys::list_api_keys_with_params_with_client(
            &self.client.request_context(self.management_key()?),
            params,
        )
        .await
    }

    /// Update API key fields, optionally moving the key to another workspace.
    pub async fn update_api_key_with_request(
        &self,
        hash: &str,
        request: &api_keys::UpdateApiKeyRequest,
    ) -> Result<api_keys::ApiKey, OpenRouterError> {
        api_keys::update_api_key_body_with_client(
            &self.client.request_context(self.management_key()?),
            hash,
            request,
        )
        .await
    }

    /// List API keys (`GET /keys`).
    pub async fn list_api_keys(
        &self,
        pagination: Option<PaginationOptions>,
        include_disabled: Option<bool>,
    ) -> Result<Vec<api_keys::ApiKey>, OpenRouterError> {
        api_keys::list_api_keys_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
            include_disabled,
        )
        .await
    }

    /// List API keys scoped to a workspace (`GET /keys?workspace_id=...`).
    pub async fn list_api_keys_in_workspace(
        &self,
        pagination: Option<PaginationOptions>,
        include_disabled: Option<bool>,
        workspace_id: Option<&str>,
    ) -> Result<Vec<api_keys::ApiKey>, OpenRouterError> {
        api_keys::list_api_keys_in_workspace_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
            include_disabled,
            workspace_id,
        )
        .await
    }

    /// Get an API key (`GET /keys/{hash}`).
    pub async fn get_api_key(&self, hash: &str) -> Result<api_keys::ApiKey, OpenRouterError> {
        api_keys::get_api_key_with_client(
            &self.client.request_context(self.management_key()?),
            hash,
        )
        .await
    }

    /// Create OAuth auth code (`POST /auth/keys/code`).
    pub async fn create_auth_code(
        &self,
        request: &auth::CreateAuthCodeRequest,
    ) -> Result<auth::AuthCodeData, OpenRouterError> {
        auth::create_auth_code_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// Create an API key from auth code (`POST /auth/keys`).
    pub async fn create_api_key_from_auth_code(
        &self,
        code: &str,
        code_verifier: Option<&str>,
        code_challenge_method: Option<auth::CodeChallengeMethod>,
    ) -> Result<auth::AuthResponse, OpenRouterError> {
        auth::exchange_code_for_api_key_with_client(
            &self.client.request_context(code),
            code_verifier,
            code_challenge_method,
        )
        .await
    }

    /// Create a Coinbase charge (`POST /credits/coinbase`).
    #[deprecated(note = "upstream deprecated Coinbase charges; use the web credits purchase flow")]
    #[allow(deprecated)]
    pub async fn create_coinbase_charge(
        &self,
        request: &credits::CoinbaseChargeRequest,
    ) -> Result<credits::CoinbaseChargeData, OpenRouterError> {
        credits::create_coinbase_charge_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            request,
        )
        .await
    }

    /// Get credits (`GET /credits`).
    pub async fn get_credits(&self) -> Result<credits::CreditsData, OpenRouterError> {
        credits::get_credits_with_client(
            &self.client.request_context(self.client.require_api_key()?),
        )
        .await
    }

    /// Get generation metadata (`GET /generation?id=...`).
    pub async fn get_generation(
        &self,
        id: impl Into<String>,
    ) -> Result<generation::GenerationData, OpenRouterError> {
        generation::get_generation_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            id,
        )
        .await
    }

    /// Get stored generation content (`GET /generation/content?id=...`).
    pub async fn get_generation_content(
        &self,
        id: impl Into<String>,
    ) -> Result<generation::GenerationContentData, OpenRouterError> {
        generation::get_generation_content_with_client(
            &self.client.request_context(self.client.require_api_key()?),
            id,
        )
        .await
    }

    /// Submit structured generation feedback (`POST /generation/feedback`).
    pub async fn submit_generation_feedback(
        &self,
        request: &generation::GenerationFeedbackRequest,
    ) -> Result<generation::GenerationFeedbackData, OpenRouterError> {
        generation::submit_generation_feedback_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    /// Get endpoint usage activity (`GET /activity`).
    pub async fn get_activity(
        &self,
        date: Option<&str>,
    ) -> Result<Vec<discovery::ActivityItem>, OpenRouterError> {
        discovery::get_activity_with_client(
            &self.client.request_context(self.management_key()?),
            date,
        )
        .await
    }

    /// Get endpoint usage activity using the full filter surface (`GET /activity`).
    pub async fn get_activity_with_params(
        &self,
        params: Option<&discovery::ActivityParams>,
    ) -> Result<Vec<discovery::ActivityItem>, OpenRouterError> {
        discovery::get_activity_with_params_and_client(
            &self.client.request_context(self.management_key()?),
            params,
        )
        .await
    }

    /// Get analytics metadata (`GET /analytics/meta`).
    pub async fn get_analytics_meta(&self) -> Result<analytics::AnalyticsMeta, OpenRouterError> {
        analytics::get_analytics_meta_with_client(
            &self.client.request_context(self.management_key()?),
        )
        .await
    }

    /// Query analytics (`POST /analytics/query`).
    pub async fn query_analytics(
        &self,
        request: &analytics::AnalyticsQueryRequest,
    ) -> Result<analytics::AnalyticsQueryResponse, OpenRouterError> {
        analytics::query_analytics_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    /// List BYOK provider credentials (`GET /byok`).
    pub async fn list_byok_keys(
        &self,
        pagination: Option<PaginationOptions>,
        workspace_id: Option<&str>,
        provider: Option<&str>,
    ) -> Result<byok::ByokKeyListResponse, OpenRouterError> {
        byok::list_byok_keys_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
            workspace_id,
            provider,
        )
        .await
    }

    /// Create a BYOK provider credential (`POST /byok`).
    pub async fn create_byok_key(
        &self,
        request: &byok::CreateByokKeyRequest,
    ) -> Result<byok::ByokKey, OpenRouterError> {
        byok::create_byok_key_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    /// Get a BYOK provider credential (`GET /byok/{id}`).
    pub async fn get_byok_key(&self, id: &str) -> Result<byok::ByokKey, OpenRouterError> {
        byok::get_byok_key_with_client(&self.client.request_context(self.management_key()?), id)
            .await
    }

    /// Update a BYOK provider credential (`PATCH /byok/{id}`).
    pub async fn update_byok_key(
        &self,
        id: &str,
        request: &byok::UpdateByokKeyRequest,
    ) -> Result<byok::ByokKey, OpenRouterError> {
        byok::update_byok_key_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// Delete a BYOK provider credential (`DELETE /byok/{id}`).
    pub async fn delete_byok_key(&self, id: &str) -> Result<bool, OpenRouterError> {
        byok::delete_byok_key_with_client(&self.client.request_context(self.management_key()?), id)
            .await
    }

    /// List observability destinations (`GET /observability/destinations`).
    pub async fn list_observability_destinations(
        &self,
        pagination: Option<PaginationOptions>,
        workspace_id: Option<&str>,
    ) -> Result<observability::ObservabilityDestinationListResponse, OpenRouterError> {
        observability::list_observability_destinations_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
            workspace_id,
        )
        .await
    }

    /// Create an observability destination (`POST /observability/destinations`).
    pub async fn create_observability_destination(
        &self,
        request: &observability::CreateObservabilityDestinationRequest,
    ) -> Result<observability::ObservabilityDestination, OpenRouterError> {
        observability::create_observability_destination_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    /// Get an observability destination (`GET /observability/destinations/{id}`).
    pub async fn get_observability_destination(
        &self,
        id: &str,
    ) -> Result<observability::ObservabilityDestination, OpenRouterError> {
        observability::get_observability_destination_with_client(
            &self.client.request_context(self.management_key()?),
            id,
        )
        .await
    }

    /// Update an observability destination (`PATCH /observability/destinations/{id}`).
    pub async fn update_observability_destination(
        &self,
        id: &str,
        request: &observability::UpdateObservabilityDestinationRequest,
    ) -> Result<observability::ObservabilityDestination, OpenRouterError> {
        observability::update_observability_destination_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// Delete an observability destination (`DELETE /observability/destinations/{id}`).
    pub async fn delete_observability_destination(
        &self,
        id: &str,
    ) -> Result<bool, OpenRouterError> {
        observability::delete_observability_destination_with_client(
            &self.client.request_context(self.management_key()?),
            id,
        )
        .await
    }

    /// List guardrails (`GET /guardrails`).
    pub async fn list_guardrails(
        &self,
        pagination: Option<PaginationOptions>,
    ) -> Result<guardrails::GuardrailListResponse, OpenRouterError> {
        guardrails::list_guardrails_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
        )
        .await
    }

    /// List guardrails scoped to a workspace (`GET /guardrails?workspace_id=...`).
    pub async fn list_guardrails_in_workspace(
        &self,
        pagination: Option<PaginationOptions>,
        workspace_id: Option<&str>,
    ) -> Result<guardrails::GuardrailListResponse, OpenRouterError> {
        guardrails::list_guardrails_in_workspace_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
            workspace_id,
        )
        .await
    }

    /// Create a guardrail (`POST /guardrails`).
    pub async fn create_guardrail(
        &self,
        request: &guardrails::CreateGuardrailRequest,
    ) -> Result<guardrails::Guardrail, OpenRouterError> {
        guardrails::create_guardrail_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    /// Get a guardrail (`GET /guardrails/{id}`).
    pub async fn get_guardrail(&self, id: &str) -> Result<guardrails::Guardrail, OpenRouterError> {
        guardrails::get_guardrail_with_client(
            &self.client.request_context(self.management_key()?),
            id,
        )
        .await
    }

    /// Update a guardrail (`PATCH /guardrails/{id}`).
    pub async fn update_guardrail(
        &self,
        id: &str,
        request: &guardrails::UpdateGuardrailRequest,
    ) -> Result<guardrails::Guardrail, OpenRouterError> {
        guardrails::update_guardrail_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// Delete a guardrail (`DELETE /guardrails/{id}`).
    pub async fn delete_guardrail(&self, id: &str) -> Result<bool, OpenRouterError> {
        guardrails::delete_guardrail_with_client(
            &self.client.request_context(self.management_key()?),
            id,
        )
        .await
    }

    /// List key assignments for a guardrail.
    pub async fn list_guardrail_key_assignments(
        &self,
        id: &str,
        pagination: Option<PaginationOptions>,
    ) -> Result<guardrails::GuardrailKeyAssignmentsResponse, OpenRouterError> {
        guardrails::list_guardrail_key_assignments_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            pagination,
        )
        .await
    }

    /// Create key assignments for a guardrail.
    pub async fn create_guardrail_key_assignments(
        &self,
        id: &str,
        request: &guardrails::BulkKeyAssignmentRequest,
    ) -> Result<guardrails::AssignedCountResponse, OpenRouterError> {
        guardrails::bulk_assign_keys_to_guardrail_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// Delete key assignments from a guardrail.
    pub async fn delete_guardrail_key_assignments(
        &self,
        id: &str,
        request: &guardrails::BulkKeyAssignmentRequest,
    ) -> Result<guardrails::UnassignedCountResponse, OpenRouterError> {
        guardrails::bulk_unassign_keys_from_guardrail_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// List member assignments for a guardrail.
    pub async fn list_guardrail_member_assignments(
        &self,
        id: &str,
        pagination: Option<PaginationOptions>,
    ) -> Result<guardrails::GuardrailMemberAssignmentsResponse, OpenRouterError> {
        guardrails::list_guardrail_member_assignments_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            pagination,
        )
        .await
    }

    /// Create member assignments for a guardrail.
    pub async fn create_guardrail_member_assignments(
        &self,
        id: &str,
        request: &guardrails::BulkMemberAssignmentRequest,
    ) -> Result<guardrails::AssignedCountResponse, OpenRouterError> {
        guardrails::bulk_assign_members_to_guardrail_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// Delete member assignments from a guardrail.
    pub async fn delete_guardrail_member_assignments(
        &self,
        id: &str,
        request: &guardrails::BulkMemberAssignmentRequest,
    ) -> Result<guardrails::UnassignedCountResponse, OpenRouterError> {
        guardrails::bulk_unassign_members_from_guardrail_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// List global key assignments.
    pub async fn list_key_assignments(
        &self,
        pagination: Option<PaginationOptions>,
    ) -> Result<guardrails::GuardrailKeyAssignmentsResponse, OpenRouterError> {
        guardrails::list_key_assignments_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
        )
        .await
    }

    /// List global member assignments.
    pub async fn list_member_assignments(
        &self,
        pagination: Option<PaginationOptions>,
    ) -> Result<guardrails::GuardrailMemberAssignmentsResponse, OpenRouterError> {
        guardrails::list_member_assignments_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
        )
        .await
    }

    /// List organization members (`GET /organization/members`).
    pub async fn list_organization_members(
        &self,
        pagination: Option<PaginationOptions>,
    ) -> Result<organization::OrganizationMembersResponse, OpenRouterError> {
        organization::list_organization_members_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
        )
        .await
    }

    /// List workspaces (`GET /workspaces`).
    pub async fn list_workspaces(
        &self,
        pagination: Option<PaginationOptions>,
    ) -> Result<workspaces::WorkspaceListResponse, OpenRouterError> {
        workspaces::list_workspaces_with_client(
            &self.client.request_context(self.management_key()?),
            pagination,
        )
        .await
    }

    /// Create a workspace (`POST /workspaces`).
    pub async fn create_workspace(
        &self,
        request: &workspaces::CreateWorkspaceRequest,
    ) -> Result<workspaces::Workspace, OpenRouterError> {
        workspaces::create_workspace_with_client(
            &self.client.request_context(self.management_key()?),
            request,
        )
        .await
    }

    /// Get a workspace (`GET /workspaces/{id}`).
    pub async fn get_workspace(&self, id: &str) -> Result<workspaces::Workspace, OpenRouterError> {
        workspaces::get_workspace_with_client(
            &self.client.request_context(self.management_key()?),
            id,
        )
        .await
    }

    /// Update a workspace (`PATCH /workspaces/{id}`).
    pub async fn update_workspace(
        &self,
        id: &str,
        request: &workspaces::UpdateWorkspaceRequest,
    ) -> Result<workspaces::Workspace, OpenRouterError> {
        workspaces::update_workspace_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// Update a workspace and clear I/O logging API key filters (`PATCH /workspaces/{id}`).
    pub async fn update_workspace_with_cleared_io_logging_api_key_ids(
        &self,
        id: &str,
        request: &workspaces::UpdateWorkspaceRequest,
    ) -> Result<workspaces::Workspace, OpenRouterError> {
        workspaces::update_workspace_with_cleared_io_logging_api_key_ids_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// Delete a workspace (`DELETE /workspaces/{id}`).
    ///
    /// Pass `Some(true)` for `confirm_default_workspace_deletion` to confirm
    /// deleting the workspace's default settings along with it.
    pub async fn delete_workspace(
        &self,
        id: &str,
        confirm_default_workspace_deletion: Option<bool>,
    ) -> Result<bool, OpenRouterError> {
        workspaces::delete_workspace_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            confirm_default_workspace_deletion,
        )
        .await
    }

    /// List budgets for a workspace (`GET /workspaces/{id}/budgets`).
    pub async fn list_workspace_budgets(
        &self,
        id: &str,
    ) -> Result<workspaces::ListWorkspaceBudgetsResponse, OpenRouterError> {
        workspaces::list_workspace_budgets_with_client(
            &self.client.request_context(self.management_key()?),
            id,
        )
        .await
    }

    /// Retrieve the budget for one interval (`GET /workspaces/{id}/budgets/{interval}`).
    pub async fn get_workspace_budget(
        &self,
        id: &str,
        interval: &str,
    ) -> Result<workspaces::GetWorkspaceBudgetResponse, OpenRouterError> {
        workspaces::get_workspace_budget_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            interval,
        )
        .await
    }

    /// Create or update a workspace budget (`PUT /workspaces/{id}/budgets/{interval}`).
    pub async fn upsert_workspace_budget(
        &self,
        id: &str,
        interval: &str,
        request: &workspaces::UpsertWorkspaceBudgetRequest,
    ) -> Result<workspaces::UpsertWorkspaceBudgetResponse, OpenRouterError> {
        workspaces::upsert_workspace_budget_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            interval,
            request,
        )
        .await
    }

    /// Delete a workspace budget (`DELETE /workspaces/{id}/budgets/{interval}`).
    pub async fn delete_workspace_budget(
        &self,
        id: &str,
        interval: &str,
    ) -> Result<bool, OpenRouterError> {
        workspaces::delete_workspace_budget_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            interval,
        )
        .await
    }

    /// List workspace members (`GET /workspaces/{id}/members`).
    pub async fn list_workspace_members(
        &self,
        id: &str,
        pagination: Option<PaginationOptions>,
    ) -> Result<workspaces::ListWorkspaceMembersResponse, OpenRouterError> {
        workspaces::list_workspace_members_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            pagination,
        )
        .await
    }

    /// Add workspace members (`POST /workspaces/{id}/members/add`).
    pub async fn add_workspace_members(
        &self,
        id: &str,
        request: &workspaces::WorkspaceMembersRequest,
    ) -> Result<workspaces::WorkspaceMembersAddResponse, OpenRouterError> {
        workspaces::add_workspace_members_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
    }

    /// Remove workspace members (`POST /workspaces/{id}/members/remove`).
    pub async fn remove_workspace_members(
        &self,
        id: &str,
        request: &workspaces::WorkspaceMembersRequest,
    ) -> Result<workspaces::WorkspaceMembersRemoveResponse, OpenRouterError> {
        workspaces::remove_workspace_members_with_client(
            &self.client.request_context(self.management_key()?),
            id,
            request,
        )
        .await
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
        if let Some(api_key) = &self.client.api_key {
            completion::send_completion_request_with_client(
                &self.client.request_context(api_key),
                request,
            )
            .await
        } else {
            Err(OpenRouterError::KeyNotConfigured)
        }
    }
}
