use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    error::OpenRouterError,
    strip_option_vec_setter,
    types::{ApiResponse, PaginationOptions},
    utils::{handle_error, parse_json_response, with_bearer_auth},
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
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_providers: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_models: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_zdr: Option<bool>,
}

impl UpdateGuardrailRequestBuilder {
    strip_option_vec_setter!(allowed_providers, String);
    strip_option_vec_setter!(allowed_models, String);
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
    let url = with_pagination(format!("{base_url}/guardrails"), pagination);
    let response = with_bearer_auth(surf::get(url), management_key).await?;

    if response.status().is_success() {
        parse_json_response(response, "guardrail list").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn create_guardrail(
    base_url: &str,
    management_key: &str,
    request: &CreateGuardrailRequest,
) -> Result<Guardrail, OpenRouterError> {
    let url = format!("{base_url}/guardrails");
    let response = with_bearer_auth(surf::post(url), management_key)
        .body_json(request)?
        .await?;

    if response.status().is_success() {
        let payload: ApiResponse<Guardrail> =
            parse_json_response(response, "guardrail creation").await?;
        Ok(payload.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn get_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<Guardrail, OpenRouterError> {
    let url = format!("{base_url}/guardrails/{}", encode(id));
    let response = with_bearer_auth(surf::get(url), management_key).await?;

    if response.status().is_success() {
        let payload: ApiResponse<Guardrail> =
            parse_json_response(response, "guardrail lookup").await?;
        Ok(payload.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn update_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &UpdateGuardrailRequest,
) -> Result<Guardrail, OpenRouterError> {
    let url = format!("{base_url}/guardrails/{}", encode(id));
    let response = with_bearer_auth(surf::patch(url), management_key)
        .body_json(request)?
        .await?;

    if response.status().is_success() {
        let payload: ApiResponse<Guardrail> =
            parse_json_response(response, "guardrail update").await?;
        Ok(payload.data)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn delete_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let url = format!("{base_url}/guardrails/{}", encode(id));
    let response = with_bearer_auth(surf::delete(url), management_key).await?;

    if response.status().is_success() {
        let payload: DeleteGuardrailResponse =
            parse_json_response(response, "guardrail deletion").await?;
        Ok(payload.deleted)
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn list_guardrail_key_assignments(
    base_url: &str,
    management_key: &str,
    id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailKeyAssignmentsResponse, OpenRouterError> {
    let url = with_pagination(
        format!("{base_url}/guardrails/{}/assignments/keys", encode(id)),
        pagination,
    );
    let response = with_bearer_auth(surf::get(url), management_key).await?;

    if response.status().is_success() {
        parse_json_response(response, "guardrail key assignments").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn bulk_assign_keys_to_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkKeyAssignmentRequest,
) -> Result<AssignedCountResponse, OpenRouterError> {
    let url = format!("{base_url}/guardrails/{}/assignments/keys", encode(id));
    let response = with_bearer_auth(surf::post(url), management_key)
        .body_json(request)?
        .await?;

    if response.status().is_success() {
        parse_json_response(response, "guardrail key bulk assignment").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn bulk_unassign_keys_from_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkKeyAssignmentRequest,
) -> Result<UnassignedCountResponse, OpenRouterError> {
    let url = format!(
        "{base_url}/guardrails/{}/assignments/keys/remove",
        encode(id)
    );
    let response = with_bearer_auth(surf::post(url), management_key)
        .body_json(request)?
        .await?;

    if response.status().is_success() {
        parse_json_response(response, "guardrail key bulk removal").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn list_guardrail_member_assignments(
    base_url: &str,
    management_key: &str,
    id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailMemberAssignmentsResponse, OpenRouterError> {
    let url = with_pagination(
        format!("{base_url}/guardrails/{}/assignments/members", encode(id)),
        pagination,
    );
    let response = with_bearer_auth(surf::get(url), management_key).await?;

    if response.status().is_success() {
        parse_json_response(response, "guardrail member assignments").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn bulk_assign_members_to_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkMemberAssignmentRequest,
) -> Result<AssignedCountResponse, OpenRouterError> {
    let url = format!("{base_url}/guardrails/{}/assignments/members", encode(id));
    let response = with_bearer_auth(surf::post(url), management_key)
        .body_json(request)?
        .await?;

    if response.status().is_success() {
        parse_json_response(response, "guardrail member bulk assignment").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn bulk_unassign_members_from_guardrail(
    base_url: &str,
    management_key: &str,
    id: &str,
    request: &BulkMemberAssignmentRequest,
) -> Result<UnassignedCountResponse, OpenRouterError> {
    let url = format!(
        "{base_url}/guardrails/{}/assignments/members/remove",
        encode(id)
    );
    let response = with_bearer_auth(surf::post(url), management_key)
        .body_json(request)?
        .await?;

    if response.status().is_success() {
        parse_json_response(response, "guardrail member bulk removal").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn list_key_assignments(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailKeyAssignmentsResponse, OpenRouterError> {
    let url = with_pagination(
        format!("{base_url}/guardrails/assignments/keys"),
        pagination,
    );
    let response = with_bearer_auth(surf::get(url), management_key).await?;

    if response.status().is_success() {
        parse_json_response(response, "global key assignments").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}

pub async fn list_member_assignments(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailMemberAssignmentsResponse, OpenRouterError> {
    let url = with_pagination(
        format!("{base_url}/guardrails/assignments/members"),
        pagination,
    );
    let response = with_bearer_auth(surf::get(url), management_key).await?;

    if response.status().is_success() {
        parse_json_response(response, "global member assignments").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
