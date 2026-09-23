use derive_builder::Builder;
use futures_util::{StreamExt, stream::BoxStream};
use reqwest::{Client as HttpClient, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    transport::{
        request as transport_request, response as transport_response, sse::response_lines,
    },
    utils::parse_sse_frames,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Intern {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub model: Option<String>,
    pub status: String,
    pub last_failure_message: Option<String>,
    pub progress: Option<InternProgress>,
    pub hostname: Option<String>,
    pub workspace_id: String,
    pub vault_id: Option<String>,
    pub attached_vault_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InternProgress {
    pub step_label: String,
    pub step_number: u32,
    pub total_steps: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InternListResponse {
    pub data: Vec<Intern>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ListInternsParams {
    #[builder(setter(strip_option), default)]
    pub limit: Option<u32>,
    #[builder(setter(custom), default)]
    pub status: Option<Vec<String>>,
    #[builder(setter(into, strip_option), default)]
    pub starting_after: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub workspace_id: Option<String>,
}

impl ListInternsParams {
    pub fn builder() -> ListInternsParamsBuilder {
        ListInternsParamsBuilder::default()
    }
}

impl ListInternsParamsBuilder {
    pub fn status<T, S>(&mut self, statuses: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.status = Some(Some(statuses.into_iter().map(Into::into).collect()));
        self
    }
}

#[derive(Serialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateInternRequest {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provision: Option<bool>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_id: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

impl CreateInternRequest {
    pub fn builder() -> CreateInternRequestBuilder {
        CreateInternRequestBuilder::default()
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateInternRequest {
    #[builder(setter(into, strip_option), default)]
    name: Option<String>,
    #[builder(setter(custom), default)]
    description: Option<Option<String>>,
    #[builder(setter(custom), default)]
    instructions: Option<Option<String>>,
    #[builder(setter(custom), default)]
    model: Option<Option<String>>,
}

impl Serialize for UpdateInternRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(value) = &self.name {
            map.serialize_entry("name", value)?;
        }
        if let Some(value) = &self.description {
            map.serialize_entry("description", value)?;
        }
        if let Some(value) = &self.instructions {
            map.serialize_entry("instructions", value)?;
        }
        if let Some(value) = &self.model {
            map.serialize_entry("model", value)?;
        }
        map.end()
    }
}

impl UpdateInternRequest {
    pub fn builder() -> UpdateInternRequestBuilder {
        UpdateInternRequestBuilder::default()
    }
}

impl UpdateInternRequestBuilder {
    pub fn description(&mut self, value: impl Into<String>) -> &mut Self {
        self.description = Some(Some(Some(value.into())));
        self
    }

    pub fn clear_description(&mut self) -> &mut Self {
        self.description = Some(Some(None));
        self
    }

    pub fn instructions(&mut self, value: impl Into<String>) -> &mut Self {
        self.instructions = Some(Some(Some(value.into())));
        self
    }

    pub fn clear_instructions(&mut self) -> &mut Self {
        self.instructions = Some(Some(None));
        self
    }

    pub fn model(&mut self, value: impl Into<String>) -> &mut Self {
        self.model = Some(Some(Some(value.into())));
        self
    }

    pub fn clear_model(&mut self) -> &mut Self {
        self.model = Some(Some(None));
        self
    }
}

#[derive(Serialize, Debug, Clone, Default, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct DeleteInternRequest {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledge_workspace_loss: Option<bool>,
}

impl DeleteInternRequest {
    pub fn builder() -> DeleteInternRequestBuilder {
        DeleteInternRequestBuilder::default()
    }
}

#[derive(Serialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct InternChatRequest {
    #[builder(setter(custom))]
    pub messages: Vec<Value>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_mode: Option<String>,
    #[builder(setter(skip), default = "true")]
    stream: bool,
}

impl InternChatRequest {
    pub fn builder() -> InternChatRequestBuilder {
        InternChatRequestBuilder::default()
    }
}

impl InternChatRequestBuilder {
    pub fn messages<T, S>(&mut self, messages: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<Value>,
    {
        self.messages = Some(messages.into_iter().map(Into::into).collect());
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InternChatSteeredResponse {
    pub session_id: String,
    pub status: String,
}

pub type InternChatStream = BoxStream<'static, Result<Value, OpenRouterError>>;

pub enum InternChatResult {
    Streaming(InternChatStream),
    Steered(InternChatSteeredResponse),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DeleteInternResponse {
    pub deleting: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ProvisionInternResponse {
    pub provisioning: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SuspendInternResponse {
    pub suspended: bool,
}

#[derive(Serialize)]
struct InternListQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    starting_after: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<&'a str>,
}

pub async fn list_interns(
    base_url: &str,
    api_key: &str,
    params: &ListInternsParams,
) -> Result<InternListResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_interns_with_client(&http_client, base_url, api_key, params).await
}

pub(crate) async fn list_interns_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    params: &ListInternsParams,
) -> Result<InternListResponse, OpenRouterError> {
    let query = InternListQuery {
        limit: params.limit,
        status: params.status.as_ref().map(|items| items.join(",")),
        starting_after: params.starting_after.as_deref(),
        workspace_id: params.workspace_id.as_deref(),
    };
    let response = with_auth(
        transport_request::get(http_client, &format!("{base_url}/interns")),
        api_key,
    )
    .query(&query)
    .send()
    .await?;
    parse_result(response, "list interns").await
}

pub async fn create_intern(
    base_url: &str,
    api_key: &str,
    idempotency_key: Option<&str>,
    request: &CreateInternRequest,
) -> Result<Intern, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_intern_with_client(&http_client, base_url, api_key, idempotency_key, request).await
}

pub(crate) async fn create_intern_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    idempotency_key: Option<&str>,
    request: &CreateInternRequest,
) -> Result<Intern, OpenRouterError> {
    let mut builder = with_auth(
        transport_request::post(http_client, &format!("{base_url}/interns")),
        api_key,
    )
    .json(request);
    if let Some(key) = idempotency_key {
        builder = builder.header("Idempotency-Key", key);
    }
    parse_result(builder.send().await?, "create intern").await
}

pub async fn get_intern(
    base_url: &str,
    api_key: &str,
    intern_id: &str,
) -> Result<Intern, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_intern_with_client(&http_client, base_url, api_key, intern_id).await
}

pub(crate) async fn get_intern_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    intern_id: &str,
) -> Result<Intern, OpenRouterError> {
    let url = format!("{base_url}/interns/{}", encode(intern_id));
    parse_result(
        with_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?,
        "get intern",
    )
    .await
}

pub async fn update_intern(
    base_url: &str,
    api_key: &str,
    intern_id: &str,
    request: &UpdateInternRequest,
) -> Result<Intern, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    update_intern_with_client(&http_client, base_url, api_key, intern_id, request).await
}

pub(crate) async fn update_intern_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    intern_id: &str,
    request: &UpdateInternRequest,
) -> Result<Intern, OpenRouterError> {
    let url = format!("{base_url}/interns/{}", encode(intern_id));
    let builder = with_auth(transport_request::patch(http_client, &url), api_key).json(request);
    parse_result(builder.send().await?, "update intern").await
}

pub async fn delete_intern(
    base_url: &str,
    api_key: &str,
    intern_id: &str,
    request: Option<&DeleteInternRequest>,
) -> Result<DeleteInternResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_intern_with_client(&http_client, base_url, api_key, intern_id, request).await
}

pub(crate) async fn delete_intern_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    intern_id: &str,
    request: Option<&DeleteInternRequest>,
) -> Result<DeleteInternResponse, OpenRouterError> {
    let url = format!("{base_url}/interns/{}", encode(intern_id));
    let default_request = DeleteInternRequest::default();
    let request = request.unwrap_or(&default_request);
    let builder = with_auth(transport_request::delete(http_client, &url), api_key).json(request);
    parse_result(builder.send().await?, "delete intern").await
}

pub async fn provision_intern(
    base_url: &str,
    api_key: &str,
    intern_id: &str,
) -> Result<ProvisionInternResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    action_with_client(&http_client, base_url, api_key, intern_id, "provision").await
}

pub async fn suspend_intern(
    base_url: &str,
    api_key: &str,
    intern_id: &str,
) -> Result<SuspendInternResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    action_with_client(&http_client, base_url, api_key, intern_id, "suspend").await
}

pub(crate) async fn action_with_client<T: DeserializeOwned>(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    intern_id: &str,
    action: &str,
) -> Result<T, OpenRouterError> {
    let url = format!("{base_url}/interns/{}/{action}", encode(intern_id));
    let builder = with_auth(transport_request::post(http_client, &url), api_key);
    parse_result(builder.send().await?, action).await
}

pub async fn chat_completion(
    base_url: &str,
    api_key: &str,
    x_title: &Option<String>,
    http_referer: &Option<String>,
    app_categories: &Option<Vec<String>>,
    intern_id: &str,
    request: &InternChatRequest,
) -> Result<InternChatResult, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    chat_completion_with_client(
        &http_client,
        base_url,
        api_key,
        (x_title, http_referer, app_categories),
        intern_id,
        request,
    )
    .await
}

pub(crate) async fn chat_completion_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    request_metadata: (&Option<String>, &Option<String>, &Option<Vec<String>>),
    intern_id: &str,
    request: &InternChatRequest,
) -> Result<InternChatResult, OpenRouterError> {
    let url = format!("{base_url}/interns/{}/chat/completions", encode(intern_id));
    let mut body = serde_json::to_value(request)?;
    body["stream"] = Value::Bool(true);
    let request = transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        api_key,
        request_metadata.0,
        request_metadata.1,
        request_metadata.2,
    )?;
    let response = request.json(&body).send().await?;
    if response.status() == StatusCode::ACCEPTED {
        return transport_response::parse_json_response(response, "intern chat")
            .await
            .map(InternChatResult::Steered);
    }
    if response.status().is_success() {
        let events = parse_sse_frames(response_lines(response))
            .filter_map(async |frame| match frame {
                Ok(frame) if frame.data == "[DONE]" => None,
                Ok(frame) => Some(
                    serde_json::from_str::<Value>(&frame.data)
                        .map_err(OpenRouterError::Serialization),
                ),
                Err(error) => Some(Err(error)),
            })
            .boxed();
        return Ok(InternChatResult::Streaming(events));
    }
    transport_response::handle_error(response).await?;
    unreachable!()
}

fn with_auth(builder: reqwest::RequestBuilder, api_key: &str) -> reqwest::RequestBuilder {
    transport_request::with_bearer_auth(builder, api_key)
}

async fn parse_result<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
    context: &str,
) -> Result<T, OpenRouterError> {
    if response.status().is_success() {
        transport_response::parse_json_response(response, context).await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
