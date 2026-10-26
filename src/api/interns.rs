use derive_builder::Builder;
use futures_util::{StreamExt, stream::BoxStream};
use http::StatusCode;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;
use urlencoding::encode;

use crate::transport::RequestContext;
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
    /// Whether the calling user created the intern; absent for organization keys.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_you: Option<bool>,
    /// SHA-256 of the intern icon, or `None` when it has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_sha256: Option<String>,
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

/// Icon bytes for one intern (`GET /interns/{internId}/icon`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InternIcon {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
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

pub(crate) async fn list_interns_with_client(
    ctx: &RequestContext<'_>,
    params: &ListInternsParams,
) -> Result<InternListResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
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
    transport_response::parse_or_error(response, "list interns").await
}

pub(crate) async fn create_intern_with_client(
    ctx: &RequestContext<'_>,
    idempotency_key: Option<&str>,
    request: &CreateInternRequest,
) -> Result<Intern, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let mut builder = with_auth(
        transport_request::post(http_client, &format!("{base_url}/interns")),
        api_key,
    )
    .json(request);
    if let Some(key) = idempotency_key {
        builder = builder.header("Idempotency-Key", key);
    }
    transport_response::parse_or_error(builder.send().await?, "create intern").await
}

pub(crate) async fn get_intern_with_client(
    ctx: &RequestContext<'_>,
    intern_id: &str,
) -> Result<Intern, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/interns/{}", encode(intern_id));
    transport_response::parse_or_error(
        with_auth(transport_request::get(http_client, &url), api_key)
            .send()
            .await?,
        "get intern",
    )
    .await
}

pub(crate) async fn get_intern_icon_with_client(
    ctx: &RequestContext<'_>,
    intern_id: &str,
    version: Option<&str>,
) -> Result<InternIcon, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/interns/{}/icon", encode(intern_id));
    let mut request = transport_request::with_client_request_headers(
        transport_request::get(http_client, &url),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?;
    if let Some(version) = version {
        request = request.query(&[("v", version)]);
    }
    let response = request.send().await?;
    if response.status().is_success() {
        let content_type = response
            .headers()
            .get(http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let bytes = response.bytes().await?.to_vec();
        Ok(InternIcon {
            bytes,
            content_type,
        })
    } else {
        Err(transport_response::error_from_response(response).await)
    }
}

pub(crate) async fn update_intern_with_client(
    ctx: &RequestContext<'_>,
    intern_id: &str,
    request: &UpdateInternRequest,
) -> Result<Intern, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/interns/{}", encode(intern_id));
    let builder = with_auth(transport_request::patch(http_client, &url), api_key).json(request);
    transport_response::parse_or_error(builder.send().await?, "update intern").await
}

pub(crate) async fn delete_intern_with_client(
    ctx: &RequestContext<'_>,
    intern_id: &str,
    request: Option<&DeleteInternRequest>,
) -> Result<DeleteInternResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/interns/{}", encode(intern_id));
    let default_request = DeleteInternRequest::default();
    let request = request.unwrap_or(&default_request);
    let builder = with_auth(transport_request::delete(http_client, &url), api_key).json(request);
    transport_response::parse_or_error(builder.send().await?, "delete intern").await
}

pub(crate) async fn action_with_client<T: DeserializeOwned>(
    ctx: &RequestContext<'_>,
    intern_id: &str,
    action: &str,
) -> Result<T, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/interns/{}/{action}", encode(intern_id));
    let builder = with_auth(transport_request::post(http_client, &url), api_key);
    transport_response::parse_or_error(builder.send().await?, action).await
}

pub(crate) async fn chat_completion_with_client(
    ctx: &RequestContext<'_>,
    intern_id: &str,
    request: &InternChatRequest,
) -> Result<InternChatResult, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/interns/{}/chat/completions", encode(intern_id));
    let mut body = serde_json::to_value(request)?;
    body["stream"] = Value::Bool(true);
    let request = transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        api_key,
        x_title,
        http_referer,
        app_categories,
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
    Err(transport_response::error_from_response(response).await)
}

fn with_auth(
    builder: crate::transport::RequestBuilder,
    api_key: &str,
) -> crate::transport::RequestBuilder {
    transport_request::with_bearer_auth(builder, api_key)
}

#[derive(Serialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct InternInvokeRequest {
    #[builder(setter(into))]
    pub input: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl InternInvokeRequest {
    pub fn builder() -> InternInvokeRequestBuilder {
        InternInvokeRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct InternInvokeResponse {
    pub session_id: String,
    pub status: String,
}

/// Treat the daemon token as a secret. It is not included in Debug output.
#[derive(Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct InternDaemonAccess {
    pub origin: String,
    pub token: String,
}

pub(crate) async fn invoke_with_client(
    ctx: &RequestContext<'_>,
    intern_id: &str,
    request: &InternInvokeRequest,
) -> Result<InternInvokeResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/interns/{}/invoke", encode(intern_id));
    transport_response::parse_or_error(
        transport_request::with_client_request_headers(
            transport_request::post(http_client, &url),
            api_key,
            x_title,
            http_referer,
            app_categories,
        )?
        .json(request)
        .send()
        .await?,
        "invoke intern",
    )
    .await
}

pub(crate) async fn daemon_with_client(
    ctx: &RequestContext<'_>,
    intern_id: &str,
    path: &str,
) -> Result<InternDaemonAccess, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/interns/{}/{path}", encode(intern_id));
    transport_response::parse_credential_response(
        transport_request::with_client_request_headers(
            transport_request::get(http_client, &url),
            api_key,
            x_title,
            http_referer,
            app_categories,
        )?
        .send()
        .await?,
        "intern daemon access",
    )
    .await
}

/// SHA-256 digest of the exact daemon request body, as lowercase hexadecimal.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct SignInternDaemonRequest {
    #[builder(setter(into))]
    #[serde(rename = "bodySha256")]
    pub body_sha256: String,
    /// Optional daemon route; when set the response also carries a single-use,
    /// route-bound proof (`x-ori-invoke-signature-v2`, `x-ori-invoke-nonce`).
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<SignInternDaemonRoute>,
}

impl SignInternDaemonRequest {
    pub fn builder() -> SignInternDaemonRequestBuilder {
        SignInternDaemonRequestBuilder::default()
    }
}

/// Route a daemon signature proof is bound to.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SignInternDaemonRoute {
    /// Only `POST` is supported upstream today.
    pub method: String,
    /// Percent-encoded request path, without origin or query.
    pub path: String,
}

impl SignInternDaemonRoute {
    pub fn post(path: impl Into<String>) -> Self {
        Self {
            method: "POST".into(),
            path: path.into(),
        }
    }
}

/// Signed daemon request headers. Signature values are omitted from Debug output.
#[derive(Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct SignInternDaemonResponse {
    #[serde(rename = "x-ori-invoke-signature")]
    pub signature: String,
    #[serde(rename = "x-ori-invoke-timestamp")]
    pub timestamp: String,
    #[serde(rename = "x-ori-invoke-user")]
    pub user: String,
    /// Single-use id bound to [`SignInternDaemonResponse::signature_v2`].
    #[serde(
        rename = "x-ori-invoke-nonce",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nonce: Option<String>,
    /// Route-bound proof, present when the request named a `route`.
    #[serde(
        rename = "x-ori-invoke-signature-v2",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub signature_v2: Option<String>,
}

pub(crate) async fn sign_daemon_with_client(
    ctx: &RequestContext<'_>,
    intern_id: &str,
    path: &str,
    request: &SignInternDaemonRequest,
) -> Result<SignInternDaemonResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: session_token,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    if request.body_sha256.len() != 64
        || !request
            .body_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(OpenRouterError::ConfigError(
            "bodySha256 must be 64 lowercase hexadecimal characters".into(),
        ));
    }
    let url = format!("{base_url}/interns/{}/{path}", encode(intern_id));
    transport_response::parse_credential_response(
        transport_request::with_client_request_headers(
            transport_request::post(http_client, &url),
            session_token,
            x_title,
            http_referer,
            app_categories,
        )?
        .json(request)
        .send()
        .await?,
        "intern daemon signature",
    )
    .await
}
