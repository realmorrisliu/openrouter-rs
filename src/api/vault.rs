use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::transport::RequestContext;
use crate::{
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct VaultSecret {
    pub name: String,
    pub hosts: Option<Vec<String>>,
    pub fingerprint: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct VaultSecretListResponse {
    pub data: Vec<VaultSecret>,
    pub has_more: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct VaultSecretResponse {
    pub data: VaultSecret,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct VaultSecretWriteRequest {
    #[builder(setter(into))]
    pub value: String,
    #[builder(setter(custom))]
    pub hosts: Vec<String>,
}

impl VaultSecretWriteRequest {
    pub fn builder() -> VaultSecretWriteRequestBuilder {
        VaultSecretWriteRequestBuilder::default()
    }
}

impl VaultSecretWriteRequestBuilder {
    pub fn hosts<T, S>(&mut self, hosts: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.hosts = Some(hosts.into_iter().map(Into::into).collect());
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct VaultSecretCopyRequest {
    #[builder(setter(custom))]
    pub names: Vec<String>,
}

impl VaultSecretCopyRequest {
    pub fn builder() -> VaultSecretCopyRequestBuilder {
        VaultSecretCopyRequestBuilder::default()
    }
}

impl VaultSecretCopyRequestBuilder {
    pub fn names<T, S>(&mut self, names: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.names = Some(names.into_iter().map(Into::into).collect());
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct VaultSecretCopyResponse {
    pub data: Vec<VaultSecret>,
}

/// One connection available to an intern, keyed by its dashboard card.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct VaultInternConnection {
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct VaultInternConnectionListResponse {
    pub data: Vec<VaultInternConnection>,
}

#[derive(Serialize)]
struct VaultSecretsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
}

pub(crate) async fn list_secrets_with_client(
    ctx: &RequestContext<'_>,
    intern_id: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<VaultSecretListResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let path = match intern_id {
        Some(id) => format!("{base_url}/vault/interns/{}/secrets", encode(id)),
        None => format!("{base_url}/vault/secrets"),
    };
    let response = transport_request::with_client_request_headers(
        transport_request::get(http_client, &path),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .query(&VaultSecretsQuery { limit, offset })
    .send()
    .await?;
    transport_response::parse_or_error(response, "list vault secrets").await
}

pub(crate) async fn store_secret_with_client(
    ctx: &RequestContext<'_>,
    intern_id: Option<&str>,
    name: &str,
    request: &VaultSecretWriteRequest,
) -> Result<VaultSecretResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let path = match intern_id {
        Some(id) => format!(
            "{base_url}/vault/interns/{}/secrets/{}",
            encode(id),
            encode(name)
        ),
        None => format!("{base_url}/vault/secrets/{}", encode(name)),
    };
    let response = transport_request::with_client_request_headers(
        transport_request::put(http_client, &path),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(request)
    .send()
    .await?;
    transport_response::parse_or_error(response, "store vault secret").await
}

pub(crate) async fn delete_secret_with_client(
    ctx: &RequestContext<'_>,
    intern_id: Option<&str>,
    name: &str,
) -> Result<(), OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let path = match intern_id {
        Some(id) => format!(
            "{base_url}/vault/interns/{}/secrets/{}",
            encode(id),
            encode(name)
        ),
        None => format!("{base_url}/vault/secrets/{}", encode(name)),
    };
    transport_response::send_unit(transport_request::with_client_request_headers(
        transport_request::delete(http_client, &path),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?)
    .await
}

pub(crate) async fn list_connections_for_intern_with_client(
    ctx: &RequestContext<'_>,
    intern_id: &str,
) -> Result<VaultInternConnectionListResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let path = format!("{base_url}/vault/interns/{}/connections", encode(intern_id));
    let response = transport_request::with_client_request_headers(
        transport_request::get(http_client, &path),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .send()
    .await?;
    transport_response::parse_or_error(response, "list intern connections").await
}

pub(crate) async fn copy_secrets_to_intern_with_client(
    ctx: &RequestContext<'_>,
    intern_id: &str,
    request: &VaultSecretCopyRequest,
) -> Result<VaultSecretCopyResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: api_key,
        x_title,
        http_referer,
        app_categories,
        ..
    } = *ctx;
    let path = format!(
        "{base_url}/vault/interns/{}/secrets/copy",
        encode(intern_id)
    );
    let response = transport_request::with_client_request_headers(
        transport_request::post(http_client, &path),
        api_key,
        x_title,
        http_referer,
        app_categories,
    )?
    .json(request)
    .send()
    .await?;
    transport_response::parse_or_error(response, "copy vault secrets").await
}
