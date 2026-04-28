use derive_builder::Builder;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    strip_option_vec_setter,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

/// Guardrail model.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Guardrail {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_providers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr: Option<bool>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

/// Paginated guardrails list response.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardrailListResponse {
    pub data: Vec<Guardrail>,
    pub total_count: f64,
}

/// Request payload for creating a guardrail (`POST /guardrails`).
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct CreateGuardrailRequest {
    #[builder(setter(into))]
    name: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit_usd: Option<f64>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    reset_interval: Option<String>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_providers: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_models: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_zdr: Option<bool>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
}

impl CreateGuardrailRequestBuilder {
    strip_option_vec_setter!(allowed_providers, String);
    strip_option_vec_setter!(allowed_models, String);
}

impl CreateGuardrailRequest {
    pub fn builder() -> CreateGuardrailRequestBuilder {
        CreateGuardrailRequestBuilder::default()
    }
}

/// Request payload for updating a guardrail (`PATCH /guardrails/{id}`).
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct UpdateGuardrailRequest {
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit_usd: Option<f64>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    reset_interval: Option<String>,
    #[builder(setter(custom), default)]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::utils::serialize_optional_empty_vec_as_null"
    )]
    allowed_providers: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::utils::serialize_optional_empty_vec_as_null"
    )]
    allowed_models: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_zdr: Option<bool>,
}

impl UpdateGuardrailRequestBuilder {
    strip_option_vec_setter!(allowed_providers, String);
    strip_option_vec_setter!(allowed_models, String);

    pub fn clear_allowed_providers(&mut self) -> &mut Self {
        self.allowed_providers = Some(Some(Vec::new()));
        self
    }

    pub fn clear_allowed_models(&mut self) -> &mut Self {
        self.allowed_models = Some(Some(Vec::new()));
        self
    }
}

impl UpdateGuardrailRequest {
    pub fn builder() -> UpdateGuardrailRequestBuilder {
        UpdateGuardrailRequestBuilder::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DeleteGuardrailResponse {
    deleted: bool,
}

/// Key assignment model.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardrailKeyAssignment {
    pub id: String,
    pub key_hash: String,
    pub guardrail_id: String,
    pub key_name: String,
    pub key_label: String,
    pub assigned_by: String,
    pub created_at: String,
}

/// Member assignment model.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardrailMemberAssignment {
    pub id: String,
    pub user_id: String,
    pub organization_id: String,
    pub guardrail_id: String,
    pub assigned_by: String,
    pub created_at: String,
}

/// Paginated key assignment list response.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardrailKeyAssignmentsResponse {
    pub data: Vec<GuardrailKeyAssignment>,
    pub total_count: f64,
}

/// Paginated member assignment list response.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardrailMemberAssignmentsResponse {
    pub data: Vec<GuardrailMemberAssignment>,
    pub total_count: f64,
}

/// Request payload for key bulk assignment endpoints.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct BulkKeyAssignmentRequest {
    key_hashes: Vec<String>,
}

impl BulkKeyAssignmentRequest {
    pub fn builder() -> BulkKeyAssignmentRequestBuilder {
        BulkKeyAssignmentRequestBuilder::default()
    }
}

/// Request payload for member bulk assignment endpoints.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
pub struct BulkMemberAssignmentRequest {
    member_user_ids: Vec<String>,
}

impl BulkMemberAssignmentRequest {
    pub fn builder() -> BulkMemberAssignmentRequestBuilder {
        BulkMemberAssignmentRequestBuilder::default()
    }
}

/// Response payload for assignment endpoints.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssignedCountResponse {
    pub assigned_count: f64,
}

/// Response payload for unassignment endpoints.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnassignedCountResponse {
    pub unassigned_count: f64,
}

#[derive(Serialize)]
struct ListGuardrailsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
}

fn with_pagination(url: String, pagination: Option<PaginationOptions>) -> String {
    let params = pagination
        .map(PaginationOptions::to_query_pairs)
        .unwrap_or_default()
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();

    if params.is_empty() {
        url
    } else {
        format!("{url}?{}", params.join("&"))
    }
}

pub async fn list_guardrails(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailListResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_guardrails_in_workspace_with_client(
        &http_client,
        base_url,
        management_key,
        pagination,
        None,
    )
    .await
}

pub(crate) async fn list_guardrails_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailListResponse, OpenRouterError> {
    list_guardrails_in_workspace_with_client(
        http_client,
        base_url,
        management_key,
        pagination,
        None,
    )
    .await
}

pub async fn list_guardrails_in_workspace(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
    workspace_id: Option<&str>,
) -> Result<GuardrailListResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_guardrails_in_workspace_with_client(
        &http_client,
        base_url,
        management_key,
        pagination,
        workspace_id,
    )
    .await
}

pub(crate) async fn list_guardrails_in_workspace_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
    workspace_id: Option<&str>,
) -> Result<GuardrailListResponse, OpenRouterError> {
    let url = format!("{base_url}/guardrails");
    let query = ListGuardrailsQuery {
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
        transport_response::parse_json_response(response, "guardrail list").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn create_guardrail(
    base_url: &str,
    management_key: &str,
    request: &CreateGuardrailRequest,
) -> Result<Guardrail, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    create_guardrail_with_client(&http_client, base_url, management_key, request).await
}

pub(crate) async fn create_guardrail_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    request: &CreateGuardrailRequest,
) -> Result<Guardrail, OpenRouterError> {
    let url = format!("{base_url}/guardrails");
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<Guardrail> =
            transport_response::parse_json_response(response, "guardrail creation").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn get_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<Guardrail, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    get_guardrail_with_client(&http_client, base_url, management_key, id).await
}

pub(crate) async fn get_guardrail_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<Guardrail, OpenRouterError> {
    let url = format!("{base_url}/guardrails/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<Guardrail> =
            transport_response::parse_json_response(response, "guardrail lookup").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn update_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateGuardrailRequest,
) -> Result<Guardrail, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    update_guardrail_with_client(&http_client, base_url, management_key, id, request).await
}

pub(crate) async fn update_guardrail_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateGuardrailRequest,
) -> Result<Guardrail, OpenRouterError> {
    let url = format!("{base_url}/guardrails/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::patch(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        let payload: ApiResponse<Guardrail> =
            transport_response::parse_json_response(response, "guardrail update").await?;
        Ok(payload.data)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn delete_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    delete_guardrail_with_client(&http_client, base_url, management_key, id).await
}

pub(crate) async fn delete_guardrail_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let url = format!("{base_url}/guardrails/{}", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::delete(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        let payload: DeleteGuardrailResponse =
            transport_response::parse_json_response(response, "guardrail deletion").await?;
        Ok(payload.deleted)
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn list_guardrail_key_assignments(
    base_url: &str,
    management_key: &str,
    id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailKeyAssignmentsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_guardrail_key_assignments_with_client(
        &http_client,
        base_url,
        management_key,
        id,
        pagination,
    )
    .await
}

pub(crate) async fn list_guardrail_key_assignments_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailKeyAssignmentsResponse, OpenRouterError> {
    let url = with_pagination(
        format!("{base_url}/guardrails/{}/assignments/keys", encode(id)),
        pagination,
    );
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "guardrail key assignments").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn bulk_assign_keys_to_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkKeyAssignmentRequest,
) -> Result<AssignedCountResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    bulk_assign_keys_to_guardrail_with_client(&http_client, base_url, management_key, id, request)
        .await
}

pub(crate) async fn bulk_assign_keys_to_guardrail_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkKeyAssignmentRequest,
) -> Result<AssignedCountResponse, OpenRouterError> {
    let url = format!("{base_url}/guardrails/{}/assignments/keys", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "guardrail key bulk assignment").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn bulk_unassign_keys_from_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkKeyAssignmentRequest,
) -> Result<UnassignedCountResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    bulk_unassign_keys_from_guardrail_with_client(
        &http_client,
        base_url,
        management_key,
        id,
        request,
    )
    .await
}

pub(crate) async fn bulk_unassign_keys_from_guardrail_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkKeyAssignmentRequest,
) -> Result<UnassignedCountResponse, OpenRouterError> {
    let url = format!(
        "{base_url}/guardrails/{}/assignments/keys/remove",
        encode(id)
    );
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "guardrail key bulk removal").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn list_guardrail_member_assignments(
    base_url: &str,
    management_key: &str,
    id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailMemberAssignmentsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_guardrail_member_assignments_with_client(
        &http_client,
        base_url,
        management_key,
        id,
        pagination,
    )
    .await
}

pub(crate) async fn list_guardrail_member_assignments_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailMemberAssignmentsResponse, OpenRouterError> {
    let url = with_pagination(
        format!("{base_url}/guardrails/{}/assignments/members", encode(id)),
        pagination,
    );
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "guardrail member assignments").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn bulk_assign_members_to_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkMemberAssignmentRequest,
) -> Result<AssignedCountResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    bulk_assign_members_to_guardrail_with_client(
        &http_client,
        base_url,
        management_key,
        id,
        request,
    )
    .await
}

pub(crate) async fn bulk_assign_members_to_guardrail_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkMemberAssignmentRequest,
) -> Result<AssignedCountResponse, OpenRouterError> {
    let url = format!("{base_url}/guardrails/{}/assignments/members", encode(id));
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "guardrail member bulk assignment").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn bulk_unassign_members_from_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkMemberAssignmentRequest,
) -> Result<UnassignedCountResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    bulk_unassign_members_from_guardrail_with_client(
        &http_client,
        base_url,
        management_key,
        id,
        request,
    )
    .await
}

pub(crate) async fn bulk_unassign_members_from_guardrail_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkMemberAssignmentRequest,
) -> Result<UnassignedCountResponse, OpenRouterError> {
    let url = format!(
        "{base_url}/guardrails/{}/assignments/members/remove",
        encode(id)
    );
    let response = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request)
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "guardrail member bulk removal").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn list_key_assignments(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailKeyAssignmentsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_key_assignments_with_client(&http_client, base_url, management_key, pagination).await
}

pub(crate) async fn list_key_assignments_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailKeyAssignmentsResponse, OpenRouterError> {
    let url = with_pagination(
        format!("{base_url}/guardrails/assignments/keys"),
        pagination,
    );
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "global key assignments").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}

pub async fn list_member_assignments(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailMemberAssignmentsResponse, OpenRouterError> {
    let http_client = crate::transport::new_client()?;
    list_member_assignments_with_client(&http_client, base_url, management_key, pagination).await
}

pub(crate) async fn list_member_assignments_with_client(
    http_client: &HttpClient,
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailMemberAssignmentsResponse, OpenRouterError> {
    let url = with_pagination(
        format!("{base_url}/guardrails/assignments/members"),
        pagination,
    );
    let response = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .send()
    .await?;

    if response.status().is_success() {
        transport_response::parse_json_response(response, "global member assignments").await
    } else {
        transport_response::handle_error(response).await?;
        unreachable!()
    }
}
