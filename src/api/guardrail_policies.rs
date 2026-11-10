//! Guardrail policies (`/guardrails/policies*`).
//!
//! Guardrail policies are the account- and workspace-scoped successors to the
//! flat guardrail surface in [`crate::api::guardrails`]. Every endpoint here
//! requires a management key.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urlencoding::encode;

use crate::transport::{RequestBuilder, RequestContext};
use crate::{
    api::guardrails::{ContentFilterBuiltinEntry, ContentFilterEntry},
    error::OpenRouterError,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

/// Guardrail policy family accepted by the policy endpoints.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum GuardrailPolicyType {
    Budget,
    ModelAvailability,
    Moderation,
    RateLimit,
}

/// Interval at which a budget policy's limit resets.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum GuardrailBudgetResetInterval {
    Daily,
    Weekly,
    Monthly,
}

/// Budget policy config as returned by the API.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
#[serde(default)]
pub struct GuardrailBudgetPolicyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_interval: Option<GuardrailBudgetResetInterval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_byok_in_budgets: Option<bool>,
}

/// Budget policy config accepted by create/update.
///
/// `limit_usd` and `reset_interval` are nullable but required, so they are sent
/// as explicit `null` when unset.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct GuardrailBudgetPolicyAuthoringConfig {
    pub limit_usd: Option<f64>,
    pub reset_interval: Option<GuardrailBudgetResetInterval>,
    pub include_byok_in_budgets: bool,
}

impl GuardrailBudgetPolicyAuthoringConfig {
    pub fn new(
        limit_usd: Option<f64>,
        reset_interval: Option<GuardrailBudgetResetInterval>,
        include_byok_in_budgets: bool,
    ) -> Self {
        Self {
            limit_usd,
            reset_interval,
            include_byok_in_budgets,
        }
    }
}

/// Model availability policy config.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct GuardrailModelAvailabilityPolicyConfig {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_models: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_providers: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_models: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_providers: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_data_regions: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_banned_authors: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_banned_models: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_banned_providers: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_eu_data_region: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr_anthropic: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr_google: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr_openai: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr_other: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr_xai: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_first_party_harness_models: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_free_model_publication: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_free_model_training: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_paid_model_training: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_safety_retention_google: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_safety_retention_other: Option<bool>,
}

impl GuardrailModelAvailabilityPolicyConfig {
    pub fn builder() -> GuardrailModelAvailabilityPolicyConfigBuilder {
        GuardrailModelAvailabilityPolicyConfigBuilder::default()
    }
}

/// Moderation policy config.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct GuardrailModerationPolicyConfig {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_filter_builtins: Option<Vec<ContentFilterBuiltinEntry>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_filters: Option<Vec<ContentFilterEntry>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_filter_patterns: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_filter_blocks_requests: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pii_filters: Option<Vec<Value>>,
    /// Model Armor settings; the upstream schema is still moving, so the raw
    /// payload is kept.
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armor: Option<Value>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcast_model_armor_flagged_requests: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_model_armor_injection_protection: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_armor_injection_confidence_level: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_armor_injection_protection_mode: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lakera_injection_protection_mode: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_moderated_providers: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_image_moderated_providers: Option<Vec<String>>,
}

impl GuardrailModerationPolicyConfig {
    pub fn builder() -> GuardrailModerationPolicyConfigBuilder {
        GuardrailModerationPolicyConfigBuilder::default()
    }
}

/// Policy as returned by create/update.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct GuardrailPolicy<C> {
    pub id: String,
    pub name: String,
    pub config: C,
}

/// Policy summary returned by the workspace policy list endpoints.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct GuardrailPolicySummary<C> {
    pub id: String,
    pub name: String,
    pub config: C,
    pub updated_at: String,
    pub member_count: u64,
    pub api_key_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_workspace_default: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_legacy_mapped: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_guardrail_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaced_legacy_guardrail_ids: Option<Vec<String>>,
}

/// Paginated workspace policy list.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct GuardrailPolicyList<C> {
    pub data: Vec<GuardrailPolicySummary<C>>,
    pub total_count: u64,
}

/// Policy with its member and API key assignments.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct GuardrailPolicyDetail<C> {
    pub id: String,
    pub name: String,
    pub config: C,
    pub updated_at: String,
    #[serde(default)]
    pub member_ids: Vec<String>,
    #[serde(default)]
    pub api_key_hashes: Vec<String>,
    #[serde(default)]
    pub opted_out_api_key_hashes: Vec<String>,
    #[serde(default)]
    pub overridden_api_key_hashes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opted_out_member_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overridden_member_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_workspace_default: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_account_policy: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces_legacy_default: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<Value>,
}

pub type GuardrailBudgetPolicy = GuardrailPolicy<GuardrailBudgetPolicyConfig>;
pub type GuardrailBudgetPolicySummary = GuardrailPolicySummary<GuardrailBudgetPolicyConfig>;
pub type GuardrailBudgetPolicyList = GuardrailPolicyList<GuardrailBudgetPolicyConfig>;
pub type GuardrailBudgetPolicyDetail = GuardrailPolicyDetail<GuardrailBudgetPolicyConfig>;
pub type GuardrailModelAvailabilityPolicy = GuardrailPolicy<GuardrailModelAvailabilityPolicyConfig>;
pub type GuardrailModelAvailabilityPolicySummary =
    GuardrailPolicySummary<GuardrailModelAvailabilityPolicyConfig>;
pub type GuardrailModelAvailabilityPolicyList =
    GuardrailPolicyList<GuardrailModelAvailabilityPolicyConfig>;
pub type GuardrailModelAvailabilityPolicyDetail =
    GuardrailPolicyDetail<GuardrailModelAvailabilityPolicyConfig>;
pub type GuardrailModerationPolicy = GuardrailPolicy<GuardrailModerationPolicyConfig>;
pub type GuardrailModerationPolicySummary = GuardrailPolicySummary<GuardrailModerationPolicyConfig>;
pub type GuardrailModerationPolicyList = GuardrailPolicyList<GuardrailModerationPolicyConfig>;
pub type GuardrailModerationPolicyDetail = GuardrailPolicyDetail<GuardrailModerationPolicyConfig>;

/// Optional assignment fields shared by the workspace policy create/update bodies.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct GuardrailPolicyAssignments {
    /// Make this the workspace default for its policy type.
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_workspace_default: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_ids: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_hashes: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opted_out_member_ids: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opted_out_api_key_hashes: Option<Vec<String>>,
}

impl GuardrailPolicyAssignments {
    pub fn builder() -> GuardrailPolicyAssignmentsBuilder {
        GuardrailPolicyAssignmentsBuilder::default()
    }
}

/// Request body for `POST /guardrails/policies/budget`.
#[derive(Serialize, Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateGuardrailBudgetPolicyRequest {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(skip), default = "GuardrailPolicyType::Budget")]
    pub policy_type: GuardrailPolicyType,
    pub config: GuardrailBudgetPolicyAuthoringConfig,
    #[builder(setter(into))]
    pub workspace_id: String,
    #[builder(setter(strip_option), default)]
    #[serde(flatten)]
    pub assignments: Option<GuardrailPolicyAssignments>,
}

impl CreateGuardrailBudgetPolicyRequest {
    pub fn builder() -> CreateGuardrailBudgetPolicyRequestBuilder {
        CreateGuardrailBudgetPolicyRequestBuilder::default()
    }
}

/// Request body for `PUT /guardrails/policies/budget/{id}`.
#[derive(Serialize, Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateGuardrailBudgetPolicyRequest {
    #[builder(setter(into))]
    pub name: String,
    pub config: GuardrailBudgetPolicyAuthoringConfig,
    #[builder(setter(into))]
    pub workspace_id: String,
    #[builder(setter(strip_option), default)]
    #[serde(flatten)]
    pub assignments: Option<GuardrailPolicyAssignments>,
}

impl UpdateGuardrailBudgetPolicyRequest {
    pub fn builder() -> UpdateGuardrailBudgetPolicyRequestBuilder {
        UpdateGuardrailBudgetPolicyRequestBuilder::default()
    }
}

/// Request body for `POST /guardrails/policies/model-availability`.
#[derive(Serialize, Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateGuardrailModelAvailabilityPolicyRequest {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(skip), default = "GuardrailPolicyType::ModelAvailability")]
    pub policy_type: GuardrailPolicyType,
    pub config: GuardrailModelAvailabilityPolicyConfig,
    #[builder(setter(into))]
    pub workspace_id: String,
    #[builder(setter(strip_option), default)]
    #[serde(flatten)]
    pub assignments: Option<GuardrailPolicyAssignments>,
}

impl CreateGuardrailModelAvailabilityPolicyRequest {
    pub fn builder() -> CreateGuardrailModelAvailabilityPolicyRequestBuilder {
        CreateGuardrailModelAvailabilityPolicyRequestBuilder::default()
    }
}

/// Request body for `PUT /guardrails/policies/model-availability/{id}`.
#[derive(Serialize, Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateGuardrailModelAvailabilityPolicyRequest {
    #[builder(setter(into))]
    pub name: String,
    pub config: GuardrailModelAvailabilityPolicyConfig,
    #[builder(setter(into))]
    pub workspace_id: String,
    #[builder(setter(strip_option), default)]
    #[serde(flatten)]
    pub assignments: Option<GuardrailPolicyAssignments>,
}

impl UpdateGuardrailModelAvailabilityPolicyRequest {
    pub fn builder() -> UpdateGuardrailModelAvailabilityPolicyRequestBuilder {
        UpdateGuardrailModelAvailabilityPolicyRequestBuilder::default()
    }
}

/// Request body for `POST /guardrails/policies/moderation`.
#[derive(Serialize, Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateGuardrailModerationPolicyRequest {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(skip), default = "GuardrailPolicyType::Moderation")]
    pub policy_type: GuardrailPolicyType,
    pub config: GuardrailModerationPolicyConfig,
    #[builder(setter(into))]
    pub workspace_id: String,
    #[builder(setter(strip_option), default)]
    #[serde(flatten)]
    pub assignments: Option<GuardrailPolicyAssignments>,
}

impl CreateGuardrailModerationPolicyRequest {
    pub fn builder() -> CreateGuardrailModerationPolicyRequestBuilder {
        CreateGuardrailModerationPolicyRequestBuilder::default()
    }
}

/// Request body for `PUT /guardrails/policies/moderation/{id}`.
#[derive(Serialize, Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateGuardrailModerationPolicyRequest {
    #[builder(setter(into))]
    pub name: String,
    pub config: GuardrailModerationPolicyConfig,
    #[builder(setter(into))]
    pub workspace_id: String,
    #[builder(setter(strip_option), default)]
    #[serde(flatten)]
    pub assignments: Option<GuardrailPolicyAssignments>,
}

impl UpdateGuardrailModerationPolicyRequest {
    pub fn builder() -> UpdateGuardrailModerationPolicyRequestBuilder {
        UpdateGuardrailModerationPolicyRequestBuilder::default()
    }
}

/// Account-wide policy as returned by `/guardrails/policies/account`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct GuardrailAccountPolicy<C> {
    pub id: String,
    pub name: String,
    pub config: C,
    pub created_at: String,
    pub updated_at: String,
    pub is_default: bool,
    pub workspace_count: u64,
    #[serde(default)]
    pub workspace_ids: Vec<String>,
    #[serde(default)]
    pub opted_out_workspace_ids: Vec<String>,
    #[serde(default)]
    pub is_synced: bool,
    #[serde(default)]
    pub is_workspace_default_synced: bool,
}

pub type GuardrailAccountBudgetPolicy = GuardrailAccountPolicy<GuardrailBudgetPolicyConfig>;
pub type GuardrailAccountModelAvailabilityPolicy =
    GuardrailAccountPolicy<GuardrailModelAvailabilityPolicyConfig>;
pub type GuardrailAccountModerationPolicy = GuardrailAccountPolicy<GuardrailModerationPolicyConfig>;

/// Account-wide policy, discriminated by `policy_type`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "policy_type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum AccountGuardrailPolicy {
    Budget(GuardrailAccountBudgetPolicy),
    ModelAvailability(GuardrailAccountModelAvailabilityPolicy),
    Moderation(GuardrailAccountModerationPolicy),
}

/// Workspaces moved off an account policy onto another policy in the same write.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct GuardrailPolicyFloorReassignment {
    #[builder(setter(into))]
    pub policy_id: String,
    #[builder(setter(into), default)]
    pub workspace_ids: Vec<String>,
}

impl GuardrailPolicyFloorReassignment {
    pub fn builder() -> GuardrailPolicyFloorReassignmentBuilder {
        GuardrailPolicyFloorReassignmentBuilder::default()
    }
}

/// Body shared by every account policy create/update request.
#[derive(Serialize, Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct GuardrailAccountPolicyRequest<C> {
    #[builder(setter(into))]
    pub name: String,
    pub config: C,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_ids: Option<Vec<String>>,
    /// Cover every workspace except `opted_out_workspace_ids`.
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opted_out_workspace_ids: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_previous_default_workspaces: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_floor_workspace_ids: Option<Vec<String>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floor_reassignment: Option<GuardrailPolicyFloorReassignment>,
}

impl<C: Clone> GuardrailAccountPolicyRequest<C> {
    pub fn builder() -> GuardrailAccountPolicyRequestBuilder<C> {
        GuardrailAccountPolicyRequestBuilder::default()
    }
}

/// Account policy create/update body, discriminated by `policy_type`.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "policy_type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum AccountGuardrailPolicyRequest {
    Budget(GuardrailAccountPolicyRequest<GuardrailBudgetPolicyAuthoringConfig>),
    ModelAvailability(GuardrailAccountPolicyRequest<GuardrailModelAvailabilityPolicyConfig>),
    Moderation(GuardrailAccountPolicyRequest<GuardrailModerationPolicyConfig>),
}

/// Policies of one type that apply to an API key or member.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct GuardrailPolicyAssignmentState {
    #[builder(setter(into), default)]
    #[serde(default)]
    pub policy_ids: Vec<String>,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub workspace_default_policy_ids: Vec<String>,
    #[builder(setter(strip_option), default)]
    #[serde(default)]
    pub opted_out: bool,
}

impl GuardrailPolicyAssignmentState {
    pub fn builder() -> GuardrailPolicyAssignmentStateBuilder {
        GuardrailPolicyAssignmentStateBuilder::default()
    }
}

/// Request body for the API key and member assignment changes.
#[derive(Serialize, Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct ChangeGuardrailPolicyAssignmentRequest {
    #[builder(setter(into))]
    pub workspace_id: String,
    pub policy_type: GuardrailPolicyType,
    #[builder(setter(into))]
    pub policy_id: String,
    /// `true` assigns the policy, `false` removes it.
    pub assigned: bool,
    /// Assignment state last read; the request fails with 409 if it changed.
    pub expected_assignment: GuardrailPolicyAssignmentState,
}

impl ChangeGuardrailPolicyAssignmentRequest {
    pub fn builder() -> ChangeGuardrailPolicyAssignmentRequestBuilder {
        ChangeGuardrailPolicyAssignmentRequestBuilder::default()
    }
}

#[derive(Serialize)]
struct WorkspacePolicyListQuery<'a> {
    workspace_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
}

#[derive(Serialize)]
struct WorkspacePolicyQuery<'a> {
    workspace_id: &'a str,
}

#[derive(Serialize)]
struct AssignmentQuery<'a> {
    workspace_id: &'a str,
    policy_type: GuardrailPolicyType,
}

#[derive(Serialize)]
struct AccountPolicyListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
}

#[derive(Deserialize)]
struct DeletedPolicyResponse {
    data: bool,
}

#[derive(Deserialize)]
struct AssignmentChangedResponse {
    data: bool,
}

async fn send<T: serde::de::DeserializeOwned>(
    req: RequestBuilder,
    action: &str,
) -> Result<T, OpenRouterError> {
    transport_response::send_json(req, action).await
}

pub(crate) async fn list_account_guardrail_policies_with_client(
    ctx: &RequestContext<'_>,
    pagination: Option<PaginationOptions>,
) -> Result<Vec<AccountGuardrailPolicy>, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/policies/account");
    let query = AccountPolicyListQuery {
        offset: pagination.and_then(|p| p.offset),
        limit: pagination.and_then(|p| p.limit),
    };
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    let req = if query.offset.is_none() && query.limit.is_none() {
        req
    } else {
        req.query(&query)
    };
    let payload: ApiResponse<Vec<AccountGuardrailPolicy>> =
        send(req, "account guardrail policy list").await?;
    Ok(payload.data)
}

pub(crate) async fn create_account_guardrail_policy_with_client(
    ctx: &RequestContext<'_>,
    request: &AccountGuardrailPolicyRequest,
) -> Result<AccountGuardrailPolicy, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/policies/account");
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request);
    let payload: ApiResponse<AccountGuardrailPolicy> =
        send(req, "account guardrail policy creation").await?;
    Ok(payload.data)
}

pub(crate) async fn update_account_guardrail_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    request: &AccountGuardrailPolicyRequest,
) -> Result<AccountGuardrailPolicy, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/policies/account/{}", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::put(http_client, &url),
        management_key,
    )
    .json(request);
    let payload: ApiResponse<AccountGuardrailPolicy> =
        send(req, "account guardrail policy update").await?;
    Ok(payload.data)
}

pub(crate) async fn delete_account_guardrail_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/policies/account/{}", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::delete(http_client, &url),
        management_key,
    );
    let payload: DeletedPolicyResponse = send(req, "account guardrail policy deletion").await?;
    Ok(payload.data)
}

pub(crate) async fn list_budget_policies_with_client(
    ctx: &RequestContext<'_>,
    workspace_id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailBudgetPolicyList, OpenRouterError> {
    list_workspace_policies(ctx, "budget", workspace_id, pagination).await
}

pub(crate) async fn list_model_availability_policies_with_client(
    ctx: &RequestContext<'_>,
    workspace_id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailModelAvailabilityPolicyList, OpenRouterError> {
    list_workspace_policies(ctx, "model-availability", workspace_id, pagination).await
}

pub(crate) async fn list_moderation_policies_with_client(
    ctx: &RequestContext<'_>,
    workspace_id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailModerationPolicyList, OpenRouterError> {
    list_workspace_policies(ctx, "moderation", workspace_id, pagination).await
}

async fn list_workspace_policies<C>(
    ctx: &RequestContext<'_>,
    segment: &str,
    workspace_id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailPolicyList<C>, OpenRouterError>
where
    C: serde::de::DeserializeOwned,
{
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/policies/{segment}");
    let query = WorkspacePolicyListQuery {
        workspace_id,
        offset: pagination.and_then(|p| p.offset),
        limit: pagination.and_then(|p| p.limit),
    };
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .query(&query);
    send(req, "guardrail policy list").await
}

pub(crate) async fn create_budget_policy_with_client(
    ctx: &RequestContext<'_>,
    request: &CreateGuardrailBudgetPolicyRequest,
) -> Result<GuardrailBudgetPolicy, OpenRouterError> {
    create_workspace_policy(ctx, "budget", request).await
}

pub(crate) async fn create_model_availability_policy_with_client(
    ctx: &RequestContext<'_>,
    request: &CreateGuardrailModelAvailabilityPolicyRequest,
) -> Result<GuardrailModelAvailabilityPolicy, OpenRouterError> {
    create_workspace_policy(ctx, "model-availability", request).await
}

pub(crate) async fn create_moderation_policy_with_client(
    ctx: &RequestContext<'_>,
    request: &CreateGuardrailModerationPolicyRequest,
) -> Result<GuardrailModerationPolicy, OpenRouterError> {
    create_workspace_policy(ctx, "moderation", request).await
}

async fn create_workspace_policy<Req, C>(
    ctx: &RequestContext<'_>,
    segment: &str,
    request: &Req,
) -> Result<GuardrailPolicy<C>, OpenRouterError>
where
    Req: Serialize,
    C: serde::de::DeserializeOwned,
{
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/policies/{segment}");
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request);
    let payload: ApiResponse<GuardrailPolicy<C>> = send(req, "guardrail policy creation").await?;
    Ok(payload.data)
}

pub(crate) async fn get_budget_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    workspace_id: &str,
) -> Result<GuardrailBudgetPolicyDetail, OpenRouterError> {
    get_workspace_policy(ctx, "budget", id, workspace_id).await
}

pub(crate) async fn get_model_availability_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    workspace_id: &str,
) -> Result<GuardrailModelAvailabilityPolicyDetail, OpenRouterError> {
    get_workspace_policy(ctx, "model-availability", id, workspace_id).await
}

pub(crate) async fn get_moderation_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    workspace_id: &str,
) -> Result<GuardrailModerationPolicyDetail, OpenRouterError> {
    get_workspace_policy(ctx, "moderation", id, workspace_id).await
}

async fn get_workspace_policy<C>(
    ctx: &RequestContext<'_>,
    segment: &str,
    id: &str,
    workspace_id: &str,
) -> Result<GuardrailPolicyDetail<C>, OpenRouterError>
where
    C: serde::de::DeserializeOwned,
{
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/policies/{segment}/{}", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .query(&WorkspacePolicyQuery { workspace_id });
    let payload: ApiResponse<GuardrailPolicyDetail<C>> =
        send(req, "guardrail policy lookup").await?;
    Ok(payload.data)
}

pub(crate) async fn update_budget_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    request: &UpdateGuardrailBudgetPolicyRequest,
) -> Result<GuardrailBudgetPolicy, OpenRouterError> {
    update_workspace_policy(ctx, "budget", id, request).await
}

pub(crate) async fn update_model_availability_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    request: &UpdateGuardrailModelAvailabilityPolicyRequest,
) -> Result<GuardrailModelAvailabilityPolicy, OpenRouterError> {
    update_workspace_policy(ctx, "model-availability", id, request).await
}

pub(crate) async fn update_moderation_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    request: &UpdateGuardrailModerationPolicyRequest,
) -> Result<GuardrailModerationPolicy, OpenRouterError> {
    update_workspace_policy(ctx, "moderation", id, request).await
}

async fn update_workspace_policy<Req, C>(
    ctx: &RequestContext<'_>,
    segment: &str,
    id: &str,
    request: &Req,
) -> Result<GuardrailPolicy<C>, OpenRouterError>
where
    Req: Serialize,
    C: serde::de::DeserializeOwned,
{
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/policies/{segment}/{}", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::put(http_client, &url),
        management_key,
    )
    .json(request);
    let payload: ApiResponse<GuardrailPolicy<C>> = send(req, "guardrail policy update").await?;
    Ok(payload.data)
}

pub(crate) async fn delete_budget_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    workspace_id: &str,
) -> Result<bool, OpenRouterError> {
    delete_workspace_policy(ctx, "budget", id, workspace_id).await
}

pub(crate) async fn delete_model_availability_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    workspace_id: &str,
) -> Result<bool, OpenRouterError> {
    delete_workspace_policy(ctx, "model-availability", id, workspace_id).await
}

pub(crate) async fn delete_moderation_policy_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    workspace_id: &str,
) -> Result<bool, OpenRouterError> {
    delete_workspace_policy(ctx, "moderation", id, workspace_id).await
}

async fn delete_workspace_policy(
    ctx: &RequestContext<'_>,
    segment: &str,
    id: &str,
    workspace_id: &str,
) -> Result<bool, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/policies/{segment}/{}", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::delete(http_client, &url),
        management_key,
    )
    .query(&WorkspacePolicyQuery { workspace_id });
    let payload: DeletedPolicyResponse = send(req, "guardrail policy deletion").await?;
    Ok(payload.data)
}

pub(crate) async fn get_api_key_policy_assignment_with_client(
    ctx: &RequestContext<'_>,
    hash: &str,
    workspace_id: &str,
    policy_type: GuardrailPolicyType,
) -> Result<GuardrailPolicyAssignmentState, OpenRouterError> {
    get_policy_assignment(ctx, "api-keys", hash, workspace_id, policy_type).await
}

pub(crate) async fn get_member_policy_assignment_with_client(
    ctx: &RequestContext<'_>,
    member_id: &str,
    workspace_id: &str,
    policy_type: GuardrailPolicyType,
) -> Result<GuardrailPolicyAssignmentState, OpenRouterError> {
    get_policy_assignment(ctx, "members", member_id, workspace_id, policy_type).await
}

async fn get_policy_assignment(
    ctx: &RequestContext<'_>,
    subject: &str,
    id: &str,
    workspace_id: &str,
    policy_type: GuardrailPolicyType,
) -> Result<GuardrailPolicyAssignmentState, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!(
        "{base_url}/guardrails/policies/assignments/{subject}/{}",
        encode(id)
    );
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    )
    .query(&AssignmentQuery {
        workspace_id,
        policy_type,
    });
    let payload: ApiResponse<GuardrailPolicyAssignmentState> =
        send(req, "guardrail policy assignment lookup").await?;
    Ok(payload.data)
}

pub(crate) async fn change_api_key_policy_assignment_with_client(
    ctx: &RequestContext<'_>,
    hash: &str,
    request: &ChangeGuardrailPolicyAssignmentRequest,
) -> Result<bool, OpenRouterError> {
    change_policy_assignment(ctx, "api-keys", hash, request).await
}

pub(crate) async fn change_member_policy_assignment_with_client(
    ctx: &RequestContext<'_>,
    member_id: &str,
    request: &ChangeGuardrailPolicyAssignmentRequest,
) -> Result<bool, OpenRouterError> {
    change_policy_assignment(ctx, "members", member_id, request).await
}

async fn change_policy_assignment(
    ctx: &RequestContext<'_>,
    subject: &str,
    id: &str,
    request: &ChangeGuardrailPolicyAssignmentRequest,
) -> Result<bool, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!(
        "{base_url}/guardrails/policies/assignments/{subject}/{}",
        encode(id)
    );
    let req = transport_request::with_bearer_auth(
        transport_request::put(http_client, &url),
        management_key,
    )
    .json(request);
    let payload: AssignmentChangedResponse =
        send(req, "guardrail policy assignment change").await?;
    Ok(payload.data)
}
