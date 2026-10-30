use derive_builder::Builder;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use urlencoding::encode;

use crate::transport::RequestContext;
use crate::{
    error::OpenRouterError,
    strip_option_vec_setter,
    transport::{request as transport_request, response as transport_response},
    types::{ApiResponse, PaginationOptions},
};

/// Action for a custom regex content filter.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum ContentFilterAction {
    Redact,
    Block,
    Flag,
}

/// Action for a built-in content filter.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum ContentFilterBuiltinAction {
    Redact,
    Block,
    Flag,
}

/// Built-in content filter identifier.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub enum ContentFilterBuiltinSlug {
    Email,
    Phone,
    Ssn,
    CreditCard,
    IpAddress,
    PersonName,
    Address,
    RegexPromptInjection,
    Secrets,
}

/// Built-in content filter entry used by guardrail create/update requests and responses.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ContentFilterBuiltinEntry {
    pub slug: ContentFilterBuiltinSlug,
    pub action: ContentFilterBuiltinAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl ContentFilterBuiltinEntry {
    pub fn new(slug: ContentFilterBuiltinSlug, action: ContentFilterBuiltinAction) -> Self {
        Self {
            slug,
            action,
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Custom regex content filter entry used by guardrail create/update requests and responses.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ContentFilterEntry {
    pub pattern: String,
    pub action: ContentFilterAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl ContentFilterEntry {
    pub fn new(pattern: impl Into<String>, action: ContentFilterAction) -> Self {
        Self {
            pattern: pattern.into(),
            action,
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Guardrail model.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Guardrail {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_usd: Option<f64>,
    #[serde(default)]
    pub include_byok_in_budgets: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_free_model_publication: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_free_model_training: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_paid_model_training: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_providers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_data_regions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_filter_builtins: Option<Vec<ContentFilterBuiltinEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_filters: Option<Vec<ContentFilterEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr_anthropic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr_openai: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr_google: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr_other: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr_xai: Option<bool>,
    /// Whether the guardrail applies to first-party harness model traffic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforce_first_party_harness_models: Option<bool>,
    /// Whether ZDR-only `google/*` requests may use endpoints that retain prompts for safety.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_safety_retention_google: Option<bool>,
    /// Whether ZDR-only non-Google requests may use endpoints that retain prompts for safety.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_safety_retention_other: Option<bool>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

/// Paginated guardrails list response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GuardrailListResponse {
    pub data: Vec<Guardrail>,
    pub total_count: f64,
}

/// Request payload for creating a guardrail (`POST /guardrails`).
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct CreateGuardrailRequest {
    #[builder(setter(into))]
    name: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit_usd: Option<f64>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    include_byok_in_budgets: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_free_model_publication: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_free_model_training: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_paid_model_training: Option<bool>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    reset_interval: Option<String>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_providers: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_models: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_data_regions: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    content_filter_builtins: Option<Vec<ContentFilterBuiltinEntry>>,
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    content_filters: Option<Vec<ContentFilterEntry>>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_zdr: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_zdr_anthropic: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_zdr_openai: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_zdr_google: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_zdr_other: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_zdr_xai: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enforce_first_party_harness_models: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_safety_retention_google: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_safety_retention_other: Option<bool>,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
}

impl CreateGuardrailRequestBuilder {
    strip_option_vec_setter!(allowed_providers, String);
    strip_option_vec_setter!(allowed_models, String);
    strip_option_vec_setter!(allowed_data_regions, String);
    strip_option_vec_setter!(content_filter_builtins, ContentFilterBuiltinEntry);
    strip_option_vec_setter!(content_filters, ContentFilterEntry);
}

impl CreateGuardrailRequest {
    pub fn builder() -> CreateGuardrailRequestBuilder {
        CreateGuardrailRequestBuilder::default()
    }
}

/// Request payload for updating a guardrail (`PATCH /guardrails/{id}`).
#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
pub struct UpdateGuardrailRequest {
    #[builder(setter(into, strip_option), default)]
    name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    description: Option<String>,
    #[builder(setter(strip_option), default)]
    limit_usd: Option<f64>,
    #[builder(setter(strip_option), default)]
    include_byok_in_budgets: Option<bool>,
    #[builder(setter(strip_option), default)]
    enable_free_model_publication: Option<bool>,
    #[builder(setter(strip_option), default)]
    enable_free_model_training: Option<bool>,
    #[builder(setter(strip_option), default)]
    enable_paid_model_training: Option<bool>,
    #[builder(setter(into, strip_option), default)]
    reset_interval: Option<String>,
    #[builder(setter(custom), default)]
    allowed_providers: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    clear_allowed_providers: bool,
    #[builder(setter(custom), default)]
    allowed_models: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    clear_allowed_models: bool,
    #[builder(setter(custom), default)]
    allowed_data_regions: Option<Vec<String>>,
    #[builder(setter(custom), default)]
    clear_allowed_data_regions: bool,
    #[builder(setter(custom), default)]
    content_filter_builtins: Option<Vec<ContentFilterBuiltinEntry>>,
    #[builder(setter(custom), default)]
    clear_content_filter_builtins: bool,
    #[builder(setter(custom), default)]
    content_filters: Option<Vec<ContentFilterEntry>>,
    #[builder(setter(custom), default)]
    clear_content_filters: bool,
    #[builder(setter(strip_option), default)]
    enforce_zdr: Option<bool>,
    #[builder(setter(strip_option), default)]
    enforce_zdr_anthropic: Option<bool>,
    #[builder(setter(strip_option), default)]
    enforce_zdr_openai: Option<bool>,
    #[builder(setter(strip_option), default)]
    enforce_zdr_google: Option<bool>,
    #[builder(setter(strip_option), default)]
    enforce_zdr_other: Option<bool>,
    #[builder(setter(strip_option), default)]
    enforce_zdr_xai: Option<bool>,
    #[builder(setter(strip_option), default)]
    enforce_first_party_harness_models: Option<bool>,
    #[builder(setter(custom), default)]
    allow_safety_retention_google: Option<bool>,
    #[builder(setter(custom), default)]
    clear_allow_safety_retention_google: bool,
    #[builder(setter(custom), default)]
    allow_safety_retention_other: Option<bool>,
    #[builder(setter(custom), default)]
    clear_allow_safety_retention_other: bool,
}

crate::utils::deserialize_nullable_patch!(
    UpdateGuardrailRequest,
    {
        name: String,
        description: String,
        limit_usd: f64,
        include_byok_in_budgets: bool,
        enable_free_model_publication: bool,
        enable_free_model_training: bool,
        enable_paid_model_training: bool,
        reset_interval: String,
        enforce_zdr: bool,
        enforce_zdr_anthropic: bool,
        enforce_zdr_openai: bool,
        enforce_zdr_google: bool,
        enforce_zdr_other: bool,
        enforce_zdr_xai: bool,
        enforce_first_party_harness_models: bool,
    },
    {
        allowed_providers: Vec<String> => clear_allowed_providers,
        allowed_models: Vec<String> => clear_allowed_models,
        allowed_data_regions: Vec<String> => clear_allowed_data_regions,
        content_filter_builtins: Vec<ContentFilterBuiltinEntry> => clear_content_filter_builtins,
        content_filters: Vec<ContentFilterEntry> => clear_content_filters,
        allow_safety_retention_google: bool => clear_allow_safety_retention_google,
        allow_safety_retention_other: bool => clear_allow_safety_retention_other,
    }
);

impl Serialize for UpdateGuardrailRequest {
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
        if let Some(value) = &self.limit_usd {
            map.serialize_entry("limit_usd", value)?;
        }
        if let Some(value) = &self.include_byok_in_budgets {
            map.serialize_entry("include_byok_in_budgets", value)?;
        }
        if let Some(value) = &self.enable_free_model_publication {
            map.serialize_entry("enable_free_model_publication", value)?;
        }
        if let Some(value) = &self.enable_free_model_training {
            map.serialize_entry("enable_free_model_training", value)?;
        }
        if let Some(value) = &self.enable_paid_model_training {
            map.serialize_entry("enable_paid_model_training", value)?;
        }
        if let Some(value) = &self.reset_interval {
            map.serialize_entry("reset_interval", value)?;
        }
        if self.clear_allowed_providers {
            map.serialize_entry("allowed_providers", &Option::<Vec<String>>::None)?;
        } else if let Some(value) = &self.allowed_providers {
            map.serialize_entry("allowed_providers", value)?;
        }
        if self.clear_allowed_models {
            map.serialize_entry("allowed_models", &Option::<Vec<String>>::None)?;
        } else if let Some(value) = &self.allowed_models {
            map.serialize_entry("allowed_models", value)?;
        }
        if self.clear_allowed_data_regions {
            map.serialize_entry("allowed_data_regions", &Option::<Vec<String>>::None)?;
        } else if let Some(value) = &self.allowed_data_regions {
            map.serialize_entry("allowed_data_regions", value)?;
        }
        if self.clear_content_filter_builtins {
            map.serialize_entry(
                "content_filter_builtins",
                &Option::<Vec<ContentFilterBuiltinEntry>>::None,
            )?;
        } else if let Some(value) = &self.content_filter_builtins {
            map.serialize_entry("content_filter_builtins", value)?;
        }
        if self.clear_content_filters {
            map.serialize_entry("content_filters", &Option::<Vec<ContentFilterEntry>>::None)?;
        } else if let Some(value) = &self.content_filters {
            map.serialize_entry("content_filters", value)?;
        }
        if let Some(value) = &self.enforce_zdr {
            map.serialize_entry("enforce_zdr", value)?;
        }
        if let Some(value) = &self.enforce_zdr_anthropic {
            map.serialize_entry("enforce_zdr_anthropic", value)?;
        }
        if let Some(value) = &self.enforce_zdr_openai {
            map.serialize_entry("enforce_zdr_openai", value)?;
        }
        if let Some(value) = &self.enforce_zdr_google {
            map.serialize_entry("enforce_zdr_google", value)?;
        }
        if let Some(value) = &self.enforce_zdr_other {
            map.serialize_entry("enforce_zdr_other", value)?;
        }
        if let Some(value) = &self.enforce_zdr_xai {
            map.serialize_entry("enforce_zdr_xai", value)?;
        }
        if let Some(value) = &self.enforce_first_party_harness_models {
            map.serialize_entry("enforce_first_party_harness_models", value)?;
        }
        if self.clear_allow_safety_retention_google {
            map.serialize_entry("allow_safety_retention_google", &Option::<bool>::None)?;
        } else if let Some(value) = &self.allow_safety_retention_google {
            map.serialize_entry("allow_safety_retention_google", value)?;
        }
        if self.clear_allow_safety_retention_other {
            map.serialize_entry("allow_safety_retention_other", &Option::<bool>::None)?;
        } else if let Some(value) = &self.allow_safety_retention_other {
            map.serialize_entry("allow_safety_retention_other", value)?;
        }
        map.end()
    }
}

impl UpdateGuardrailRequestBuilder {
    pub fn allowed_providers<T, S>(&mut self, items: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_providers = Some(Some(items.into_iter().map(Into::into).collect()));
        self.clear_allowed_providers = Some(false);
        self
    }

    pub fn allowed_models<T, S>(&mut self, items: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_models = Some(Some(items.into_iter().map(Into::into).collect()));
        self.clear_allowed_models = Some(false);
        self
    }

    pub fn allowed_data_regions<T, S>(&mut self, items: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_data_regions = Some(Some(items.into_iter().map(Into::into).collect()));
        self.clear_allowed_data_regions = Some(false);
        self
    }

    pub fn allow_safety_retention_google(&mut self, value: bool) -> &mut Self {
        self.allow_safety_retention_google = Some(Some(value));
        self.clear_allow_safety_retention_google = Some(false);
        self
    }

    pub fn clear_allow_safety_retention_google(&mut self) -> &mut Self {
        self.allow_safety_retention_google = Some(None);
        self.clear_allow_safety_retention_google = Some(true);
        self
    }

    pub fn allow_safety_retention_other(&mut self, value: bool) -> &mut Self {
        self.allow_safety_retention_other = Some(Some(value));
        self.clear_allow_safety_retention_other = Some(false);
        self
    }

    pub fn clear_allow_safety_retention_other(&mut self) -> &mut Self {
        self.allow_safety_retention_other = Some(None);
        self.clear_allow_safety_retention_other = Some(true);
        self
    }

    pub fn clear_allowed_providers(&mut self) -> &mut Self {
        self.allowed_providers = Some(None);
        self.clear_allowed_providers = Some(true);
        self
    }

    pub fn clear_allowed_models(&mut self) -> &mut Self {
        self.allowed_models = Some(None);
        self.clear_allowed_models = Some(true);
        self
    }

    pub fn clear_allowed_data_regions(&mut self) -> &mut Self {
        self.allowed_data_regions = Some(None);
        self.clear_allowed_data_regions = Some(true);
        self
    }

    pub fn content_filter_builtins<T, S>(&mut self, items: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<ContentFilterBuiltinEntry>,
    {
        self.content_filter_builtins = Some(Some(items.into_iter().map(Into::into).collect()));
        self.clear_content_filter_builtins = Some(false);
        self
    }

    pub fn content_filters<T, S>(&mut self, items: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: Into<ContentFilterEntry>,
    {
        self.content_filters = Some(Some(items.into_iter().map(Into::into).collect()));
        self.clear_content_filters = Some(false);
        self
    }

    pub fn clear_content_filter_builtins(&mut self) -> &mut Self {
        self.content_filter_builtins = Some(None);
        self.clear_content_filter_builtins = Some(true);
        self
    }

    pub fn clear_content_filters(&mut self) -> &mut Self {
        self.content_filters = Some(None);
        self.clear_content_filters = Some(true);
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub struct GuardrailKeyAssignmentsResponse {
    pub data: Vec<GuardrailKeyAssignment>,
    pub total_count: f64,
}

/// Paginated member assignment list response.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GuardrailMemberAssignmentsResponse {
    pub data: Vec<GuardrailMemberAssignment>,
    pub total_count: f64,
}

/// Request payload for key bulk assignment endpoints.
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "OpenRouterError"))]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub struct AssignedCountResponse {
    pub assigned_count: f64,
}

/// Response payload for unassignment endpoints.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
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

pub(crate) async fn list_guardrails_with_client(
    ctx: &RequestContext<'_>,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailListResponse, OpenRouterError> {
    list_guardrails_in_workspace_with_client(ctx, pagination, None).await
}

pub(crate) async fn list_guardrails_in_workspace_with_client(
    ctx: &RequestContext<'_>,
    pagination: Option<PaginationOptions>,
    workspace_id: Option<&str>,
) -> Result<GuardrailListResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
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
    let req = if query.offset.is_none() && query.limit.is_none() && query.workspace_id.is_none() {
        req
    } else {
        req.query(&query)
    };
    transport_response::send_json(req, "guardrail list").await
}

pub(crate) async fn create_guardrail_with_client(
    ctx: &RequestContext<'_>,
    request: &CreateGuardrailRequest,
) -> Result<Guardrail, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails");
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request);
    let payload: ApiResponse<Guardrail> =
        transport_response::send_json(req, "guardrail creation").await?;
    Ok(payload.data)
}

pub(crate) async fn get_guardrail_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
) -> Result<Guardrail, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/{}", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    let payload: ApiResponse<Guardrail> =
        transport_response::send_json(req, "guardrail lookup").await?;
    Ok(payload.data)
}

pub(crate) async fn update_guardrail_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    request: &UpdateGuardrailRequest,
) -> Result<Guardrail, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/{}", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::patch(http_client, &url),
        management_key,
    )
    .json(request);
    let payload: ApiResponse<Guardrail> =
        transport_response::send_json(req, "guardrail update").await?;
    Ok(payload.data)
}

pub(crate) async fn delete_guardrail_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
) -> Result<bool, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/{}", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::delete(http_client, &url),
        management_key,
    );
    let payload: DeleteGuardrailResponse =
        transport_response::send_json(req, "guardrail deletion").await?;
    Ok(payload.deleted)
}

pub(crate) async fn list_guardrail_key_assignments_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailKeyAssignmentsResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = with_pagination(
        format!("{base_url}/guardrails/{}/assignments/keys", encode(id)),
        pagination,
    );
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    transport_response::send_json(req, "guardrail key assignments").await
}

pub(crate) async fn bulk_assign_keys_to_guardrail_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    request: &BulkKeyAssignmentRequest,
) -> Result<AssignedCountResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/{}/assignments/keys", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request);
    transport_response::send_json(req, "guardrail key bulk assignment").await
}

pub(crate) async fn bulk_unassign_keys_from_guardrail_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    request: &BulkKeyAssignmentRequest,
) -> Result<UnassignedCountResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!(
        "{base_url}/guardrails/{}/assignments/keys/remove",
        encode(id)
    );
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request);
    transport_response::send_json(req, "guardrail key bulk removal").await
}

pub(crate) async fn list_guardrail_member_assignments_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailMemberAssignmentsResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = with_pagination(
        format!("{base_url}/guardrails/{}/assignments/members", encode(id)),
        pagination,
    );
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    transport_response::send_json(req, "guardrail member assignments").await
}

pub(crate) async fn bulk_assign_members_to_guardrail_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    request: &BulkMemberAssignmentRequest,
) -> Result<AssignedCountResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!("{base_url}/guardrails/{}/assignments/members", encode(id));
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request);
    transport_response::send_json(req, "guardrail member bulk assignment").await
}

pub(crate) async fn bulk_unassign_members_from_guardrail_with_client(
    ctx: &RequestContext<'_>,
    id: &str,
    request: &BulkMemberAssignmentRequest,
) -> Result<UnassignedCountResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = format!(
        "{base_url}/guardrails/{}/assignments/members/remove",
        encode(id)
    );
    let req = transport_request::with_bearer_auth(
        transport_request::post(http_client, &url),
        management_key,
    )
    .json(request);
    transport_response::send_json(req, "guardrail member bulk removal").await
}

pub(crate) async fn list_key_assignments_with_client(
    ctx: &RequestContext<'_>,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailKeyAssignmentsResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = with_pagination(
        format!("{base_url}/guardrails/assignments/keys"),
        pagination,
    );
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    transport_response::send_json(req, "global key assignments").await
}

pub(crate) async fn list_member_assignments_with_client(
    ctx: &RequestContext<'_>,
    pagination: Option<PaginationOptions>,
) -> Result<GuardrailMemberAssignmentsResponse, OpenRouterError> {
    let RequestContext {
        http_client,
        base_url,
        credential: management_key,
        ..
    } = *ctx;
    let url = with_pagination(
        format!("{base_url}/guardrails/assignments/members"),
        pagination,
    );
    let req = transport_request::with_bearer_auth(
        transport_request::get(http_client, &url),
        management_key,
    );
    transport_response::send_json(req, "global member assignments").await
}
