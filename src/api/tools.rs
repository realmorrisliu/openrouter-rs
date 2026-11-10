//! Server tool catalog (`/tools`, `/tools/{name}`).
//!
//! The catalog describes every server tool OpenRouter can run for a model: the
//! `tools[].type` spellings each API format accepts, the engines behind it with
//! their pricing, and how many endpoints run it natively.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

use crate::transport::RequestContext;
use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

/// API format a server tool can be used on.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub enum ServerToolApiFormat {
    Responses,
    ChatCompletions,
    AnthropicMessages,
}

impl ServerToolApiFormat {
    fn as_query_value(self) -> &'static str {
        match self {
            Self::Responses => "responses",
            Self::ChatCompletions => "chat-completions",
            Self::AnthropicMessages => "anthropic-messages",
        }
    }
}

/// Lifecycle status of a catalogued tool.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum ServerToolStatus {
    Active,
    Deprecated,
}

/// Who runs the tool call.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum ServerToolExecutedBy {
    /// The model provider runs it during inference.
    Provider,
    /// OpenRouter runs it.
    OpenRouter,
    /// The caller's application runs it after the call is returned.
    Client,
}

/// Whether a BYOK key is required for the engine.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServerToolByok {
    #[serde(rename = "required")]
    Required,
    #[serde(rename = "optional")]
    Optional,
    #[serde(rename = "none")]
    Unsupported,
}

/// Which side publishes the engine's prices.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServerToolPricingSource {
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "provider")]
    Provider,
    #[serde(rename = "byok")]
    Byok,
    #[serde(rename = "none")]
    NoCharge,
}

/// One price row for an engine.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ServerToolPrice {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    pub unit: String,
    pub price: String,
    pub max_units_per_call: u32,
}

/// One engine that can execute a server tool.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ServerToolEngine {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub executed_by: ServerToolExecutedBy,
    #[serde(default)]
    pub data_regions: Vec<String>,
    pub byok: ServerToolByok,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,
    pub pricing_source: ServerToolPricingSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_doc_url: Option<String>,
    #[serde(default)]
    pub pricing: Vec<ServerToolPrice>,
}

/// Model that runs a server tool natively.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ServerToolNativeModel {
    pub slug: String,
    #[serde(default)]
    pub providers: Vec<String>,
}

/// How many endpoints and models run a server tool natively.
///
/// `GET /tools` reports the counts only; `GET /tools/{name}` adds the models.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ServerToolNativeSupport {
    pub endpoint_count: u32,
    pub model_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<ServerToolNativeModel>>,
}

/// One server tool in the catalog.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ServerTool {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub tool_description: String,
    pub status: ServerToolStatus,
    pub docs_url: String,
    /// Accepted `tools[].type` spellings for this tool.
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub supported_api_formats: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_engine: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_engine: Option<String>,
    /// Schema of the tool parameters as the API accepts them.
    #[serde(default)]
    pub parameters_schema: Value,
    /// Schema of the tool input the model sees.
    #[serde(default)]
    pub input_schema: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
    #[serde(default)]
    pub engines: Vec<ServerToolEngine>,
    pub native_support: ServerToolNativeSupport,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// One page of the tool catalog.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ListToolsResponse {
    pub data: Vec<ServerTool>,
    pub total_count: u64,
    pub links: ServerToolLinks,
}

/// Cursor links for a catalog page.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ServerToolLinks {
    /// Next page URL, or `None` on the last page.
    #[serde(default)]
    pub next: Option<String>,
}

#[derive(Serialize)]
struct ListToolsQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_format: Option<&'a str>,
}

/// List the server tool catalog (`GET /tools`).
pub(crate) async fn list_tools_with_client(
    ctx: &RequestContext<'_>,
    pagination: Option<PaginationOptions>,
    api_format: Option<ServerToolApiFormat>,
) -> Result<ListToolsResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/tools");
    let query = ListToolsQuery {
        offset: pagination.and_then(|p| p.offset),
        limit: pagination.and_then(|p| p.limit),
        api_format: api_format.map(ServerToolApiFormat::as_query_value),
    };
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let req = if query.offset.is_none() && query.limit.is_none() && query.api_format.is_none() {
        req
    } else {
        req.query(&query)
    };
    transport_response::send_json(req, "server tool catalog").await
}

/// Get one server tool by name or alias (`GET /tools/{name}`).
pub(crate) async fn get_tool_with_client(
    ctx: &RequestContext<'_>,
    name: &str,
) -> Result<ServerTool, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/tools/{}", encode(name));
    let req =
        transport_request::with_bearer_auth(transport_request::get(http_client, &url), api_key);
    let payload: ApiResponse<ServerTool> =
        transport_response::send_json(req, "server tool lookup").await?;
    Ok(payload.data)
}
