use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

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

#[derive(Serialize)]
struct VaultSecretsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
}

pub async fn list_secrets(
    base_url: &str,
    api_key: &str,
    intern_id: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<VaultSecretListResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_secrets_with_client(&http_client, base_url, api_key, intern_id, limit, offset).await
}

pub(crate) async fn list_secrets_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    intern_id: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<VaultSecretListResponse, OpenRouterError> {
    let path = match intern_id {
        Some(id) => format!("{base_url}/vault/interns/{}/secrets", encode(id)),
        None => format!("{base_url}/vault/secrets"),
    };
    let response = with_auth(transport_request::get(http_client, &path), api_key)
        .query(&VaultSecretsQuery { limit, offset })
        .send()
        .await?;
    parse_result(response, "list vault secrets").await
}

pub async fn store_secret(
    base_url: &str,
    api_key: &str,
    intern_id: Option<&str>,
    name: &str,
    request: &VaultSecretWriteRequest,
) -> Result<VaultSecretResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    store_secret_with_client(&http_client, base_url, api_key, intern_id, name, request).await
}

pub(crate) async fn store_secret_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    intern_id: Option<&str>,
    name: &str,
    request: &VaultSecretWriteRequest,
) -> Result<VaultSecretResponse, OpenRouterError> {
    let path = match intern_id {
        Some(id) => format!(
            "{base_url}/vault/interns/{}/secrets/{}",
            encode(id),
            encode(name)
        ),
        None => format!("{base_url}/vault/secrets/{}", encode(name)),
    };
    let response = with_auth(transport_request::put(http_client, &path), api_key)
        .json(request)
        .send()
        .await?;
    parse_result(response, "store vault secret").await
}

pub async fn delete_secret(
    base_url: &str,
    api_key: &str,
    intern_id: Option<&str>,
    name: &str,
) -> Result<(), OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_secret_with_client(&http_client, base_url, api_key, intern_id, name).await
}

pub(crate) async fn delete_secret_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    intern_id: Option<&str>,
    name: &str,
) -> Result<(), OpenRouterError> {
    let path = match intern_id {
        Some(id) => format!(
            "{base_url}/vault/interns/{}/secrets/{}",
            encode(id),
            encode(name)
        ),
        None => format!("{base_url}/vault/secrets/{}", encode(name)),
    };
    let response = with_auth(transport_request::delete(http_client, &path), api_key)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(())
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn copy_secrets_to_intern(
    base_url: &str,
    api_key: &str,
    intern_id: &str,
    request: &VaultSecretCopyRequest,
) -> Result<VaultSecretCopyResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    copy_secrets_to_intern_with_client(&http_client, base_url, api_key, intern_id, request).await
}

pub(crate) async fn copy_secrets_to_intern_with_client(
    http_client: &HttpClient,
    base_url: &str,
    api_key: &str,
    intern_id: &str,
    request: &VaultSecretCopyRequest,
) -> Result<VaultSecretCopyResponse, OpenRouterError> {
    let path = format!(
        "{base_url}/vault/interns/{}/secrets/copy",
        encode(intern_id)
    );
    let response = with_auth(transport_request::post(http_client, &path), api_key)
        .json(request)
        .send()
        .await?;
    parse_result(response, "copy vault secrets").await
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
