use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    strip_option_vec_setter,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

#[derive(Serialize)]
struct ListObservabilityDestinationsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
}

/// Structured observability routing rules.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ObservabilityFilterRulesConfig {
    pub groups: Vec<ObservabilityFilterGroup>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

impl ObservabilityFilterRulesConfig {
    pub fn builder() -> ObservabilityFilterRulesConfigBuilder {
        ObservabilityFilterRulesConfigBuilder::default()
    }
}

/// One observability filter group.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ObservabilityFilterGroup {
    pub rules: Vec<ObservabilityFilterRule>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logic: Option<String>,
}

impl ObservabilityFilterGroup {
    pub fn builder() -> ObservabilityFilterGroupBuilder {
        ObservabilityFilterGroupBuilder::default()
    }
}

/// One observability filter rule.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ObservabilityFilterRule {
    #[builder(setter(into))]
    pub field: String,
    #[builder(setter(into))]
    pub operator: String,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

impl ObservabilityFilterRule {
    pub fn builder() -> ObservabilityFilterRuleBuilder {
        ObservabilityFilterRuleBuilder::default()
    }
}

/// Observability destination returned by `/observability/destinations`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ObservabilityDestination {
    pub id: String,
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub enabled: bool,
    pub privacy_mode: bool,
    pub sampling_rate: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_hashes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_rules: Option<ObservabilityFilterRulesConfig>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(rename = "type")]
    pub destination_type: String,
    pub config: Value,
}

/// Paginated observability destination list response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ObservabilityDestinationListResponse {
    pub data: Vec<ObservabilityDestination>,
    pub total_count: u64,
}

/// Request payload for creating an observability destination.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateObservabilityDestinationRequest {
    #[serde(rename = "type")]
    #[builder(setter(into))]
    pub destination_type: String,
    #[builder(setter(into))]
    pub name: String,
    pub config: Value,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_hashes: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_mode: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_rate: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_rules: Option<ObservabilityFilterRulesConfig>,
}

impl CreateObservabilityDestinationRequest {
    pub fn builder() -> CreateObservabilityDestinationRequestBuilder {
        CreateObservabilityDestinationRequestBuilder::default()
    }
}

impl CreateObservabilityDestinationRequestBuilder {
    strip_option_vec_setter!(api_key_hashes, String);
}

/// Request payload for updating an observability destination.
#[derive(Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateObservabilityDestinationRequest {
    #[builder(setter(into, strip_option), default)]
    pub name: Option<String>,
    #[builder(setter(strip_option), default)]
    pub config: Option<Value>,
    #[builder(setter(custom), default)]
    pub api_key_hashes: Option<Vec<String>>,
    #[serde(skip)]
    #[builder(setter(custom), default)]
    clear_api_key_hashes: bool,
    #[builder(setter(strip_option), default)]
    pub enabled: Option<bool>,
    #[builder(setter(strip_option), default)]
    pub privacy_mode: Option<bool>,
    #[builder(setter(strip_option), default)]
    pub sampling_rate: Option<f64>,
    #[builder(setter(custom), default)]
    pub filter_rules: Option<ObservabilityFilterRulesConfig>,
    #[serde(skip)]
    #[builder(setter(custom), default)]
    clear_filter_rules: bool,
}

impl Serialize for UpdateObservabilityDestinationRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(value) = &self.name {
            map.serialize_entry("name", value)?;
        }
        if let Some(value) = &self.config {
            map.serialize_entry("config", value)?;
        }
        if self.clear_api_key_hashes {
            map.serialize_entry("api_key_hashes", &Option::<Vec<String>>::None)?;
        } else if let Some(value) = &self.api_key_hashes {
            map.serialize_entry("api_key_hashes", value)?;
        }
        if let Some(value) = &self.enabled {
            map.serialize_entry("enabled", value)?;
        }
        if let Some(value) = &self.privacy_mode {
            map.serialize_entry("privacy_mode", value)?;
        }
        if let Some(value) = &self.sampling_rate {
            map.serialize_entry("sampling_rate", value)?;
        }
        if self.clear_filter_rules {
            map.serialize_entry(
                "filter_rules",
                &Option::<ObservabilityFilterRulesConfig>::None,
            )?;
        } else if let Some(value) = &self.filter_rules {
            map.serialize_entry("filter_rules", value)?;
        }
        map.end()
    }
}

impl UpdateObservabilityDestinationRequest {
    pub fn builder() -> UpdateObservabilityDestinationRequestBuilder {
        UpdateObservabilityDestinationRequestBuilder::default()
    }
}

impl UpdateObservabilityDestinationRequestBuilder {
    pub fn api_key_hashes<T, S>(&mut self, items: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.api_key_hashes = Some(Some(items.into_iter().map(Into::into).collect()));
        self.clear_api_key_hashes = Some(false);
        self
    }

    pub fn clear_api_key_hashes(&mut self) -> &mut Self {
        self.api_key_hashes = Some(None);
        self.clear_api_key_hashes = Some(true);
        self
    }

    pub fn filter_rules(&mut self, value: ObservabilityFilterRulesConfig) -> &mut Self {
        self.filter_rules = Some(Some(value));
        self.clear_filter_rules = Some(false);
        self
    }

    pub fn clear_filter_rules(&mut self) -> &mut Self {
        self.filter_rules = Some(None);
        self.clear_filter_rules = Some(true);
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DeleteObservabilityDestinationResponse {
    deleted: bool,
}

/// List observability destinations (`GET /observability/destinations`).
pub async fn list_observability_destinations(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
    workspace_id: Option<&str>,
) -> Result<ObservabilityDestinationListResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_observability_destinations_with_client(
        &http_client,
        base_url,
        management_key,
        pagination,
        workspace_id,
    )
    .await
}

pub(crate) async fn list_observability_destinations_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
    workspace_id: Option<&str>,
) -> Result<ObservabilityDestinationListResponse, OpenRouterError> {
    let url = format!("{base_url}/observability/destinations");
    let query = ListObservabilityDestinationsQuery {
        offset: pagination.and_then(|p| p.offset),
        limit: pagination.and_then(|p| p.limit),
        workspace_id: workspace_id.map(ToOwned::to_owned),
    };
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    let response =
        if query.offset.is_none() && query.limit.is_none() && query.workspace_id.is_none() {
            req.send().await?
        } else {
            req.query(&query).send().await?
        };

    if response.status().is_success() {
        transport_response::parse_json_response(response, "observability destination list").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Create an observability destination (`POST /observability/destinations`).
pub async fn create_observability_destination(
    base_url: &str,
    management_key: &str,
    request: &CreateObservabilityDestinationRequest,
) -> Result<ObservabilityDestination, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_observability_destination_with_client(&http_client, base_url, management_key, request)
        .await
}

pub(crate) async fn create_observability_destination_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    request: &CreateObservabilityDestinationRequest,
) -> Result<ObservabilityDestination, OpenRouterError> {
    let url = format!("{base_url}/observability/destinations");
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<ObservabilityDestination> =
            transport_response::parse_json_response(response, "observability destination creation")
                .await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Get an observability destination (`GET /observability/destinations/{id}`).
pub async fn get_observability_destination(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<ObservabilityDestination, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_observability_destination_with_client(&http_client, base_url, management_key, id).await
}

pub(crate) async fn get_observability_destination_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<ObservabilityDestination, OpenRouterError> {
    let url = format!("{base_url}/observability/destinations/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<ObservabilityDestination> =
            transport_response::parse_json_response(response, "observability destination lookup")
                .await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Update an observability destination (`PATCH /observability/destinations/{id}`).
pub async fn update_observability_destination(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateObservabilityDestinationRequest,
) -> Result<ObservabilityDestination, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    update_observability_destination_with_client(
        &http_client,
        base_url,
        management_key,
        id,
        request,
    )
    .await
}

pub(crate) async fn update_observability_destination_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateObservabilityDestinationRequest,
) -> Result<ObservabilityDestination, OpenRouterError> {
    let url = format!("{base_url}/observability/destinations/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::patch(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<ObservabilityDestination> =
            transport_response::parse_json_response(response, "observability destination update")
                .await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

/// Delete an observability destination (`DELETE /observability/destinations/{id}`).
pub async fn delete_observability_destination(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_observability_destination_with_client(&http_client, base_url, management_key, id).await
}

pub(crate) async fn delete_observability_destination_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let url = format!("{base_url}/observability/destinations/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::delete(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: DeleteObservabilityDestinationResponse =
            transport_response::parse_json_response(response, "observability destination deletion")
                .await?;
        Ok(payload.deleted)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
