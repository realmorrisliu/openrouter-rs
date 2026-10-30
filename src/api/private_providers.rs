use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::transport::RequestContext;
use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::ApiResponse,
};

/// Data-handling policy a private provider declares.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct PrivateProviderDataPolicy {
    pub training: bool,
    pub retains_prompts: bool,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_retention_days: Option<u32>,
}

impl PrivateProviderDataPolicy {
    pub fn builder() -> PrivateProviderDataPolicyBuilder {
        PrivateProviderDataPolicyBuilder::default()
    }
}

/// A provider you host yourself and register with OpenRouter.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PrivateProvider {
    pub name: String,
    pub slug: String,
    pub display_name: String,
    pub base_url: Option<String>,
    pub data_policy: Option<PrivateProviderDataPolicy>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ListPrivateProvidersResponse {
    pub data: Vec<PrivateProvider>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DeletedPrivateProvider {
    pub slug: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreatePrivateProviderRequest {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_policy: Option<PrivateProviderDataPolicy>,
}

impl CreatePrivateProviderRequest {
    pub fn builder() -> CreatePrivateProviderRequestBuilder {
        CreatePrivateProviderRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdatePrivateProviderRequest {
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_policy: Option<PrivateProviderDataPolicy>,
}

impl UpdatePrivateProviderRequest {
    pub fn builder() -> UpdatePrivateProviderRequestBuilder {
        UpdatePrivateProviderRequestBuilder::default()
    }
}

pub(crate) async fn list_private_providers_with_client(
    ctx: &RequestContext<'_>,
) -> Result<Vec<PrivateProvider>, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/private-providers");
    let req = transport_request::with_client_request_headers(
        transport_request::get(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?;
    let payload: ListPrivateProvidersResponse =
        transport_response::send_json(req, "list_private_providers").await?;
    Ok(payload.data)
}

pub(crate) async fn create_private_provider_with_client(
    ctx: &RequestContext<'_>,
    request: &CreatePrivateProviderRequest,
) -> Result<PrivateProvider, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/private-providers");
    let req = transport_request::with_client_request_headers(
        transport_request::post(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(request);
    let payload: ApiResponse<PrivateProvider> =
        transport_response::send_json(req, "create_private_provider").await?;
    Ok(payload.data)
}

pub(crate) async fn get_private_provider_with_client(
    ctx: &RequestContext<'_>,
    slug: &str,
) -> Result<PrivateProvider, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/private-providers/{}", encode(slug));
    let req = transport_request::with_client_request_headers(
        transport_request::get(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?;
    let payload: ApiResponse<PrivateProvider> =
        transport_response::send_json(req, "get_private_provider").await?;
    Ok(payload.data)
}

pub(crate) async fn update_private_provider_with_client(
    ctx: &RequestContext<'_>,
    slug: &str,
    request: &UpdatePrivateProviderRequest,
) -> Result<PrivateProvider, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/private-providers/{}", encode(slug));
    let req = transport_request::with_client_request_headers(
        transport_request::patch(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(request);
    let payload: ApiResponse<PrivateProvider> =
        transport_response::send_json(req, "update_private_provider").await?;
    Ok(payload.data)
}

pub(crate) async fn delete_private_provider_with_client(
    ctx: &RequestContext<'_>,
    slug: &str,
) -> Result<DeletedPrivateProvider, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let url = format!("{base_url}/private-providers/{}", encode(slug));
    let req = transport_request::with_client_request_headers(
        transport_request::delete(http_client, &url),
        management_key,
        x_title,
        http_referer,
        app_categories,
    )?;
    let payload: ApiResponse<DeletedPrivateProvider> =
        transport_response::send_json(req, "delete_private_provider").await?;
    Ok(payload.data)
}
