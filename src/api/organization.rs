use serde::{Deserialize, Serialize};

use crate::{
    error::OpenRouterError,
    types::PaginationOptions,
    utils::{handle_error, parse_json_response, with_bearer_auth},
};

/// One organization member returned by `GET /organization/members`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrganizationMember {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    pub email: String,
    pub role: String,
}

/// Paginated organization member list.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrganizationMembersResponse {
    pub data: Vec<OrganizationMember>,
    pub total_count: u64,
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

/// List organization members for the authenticated management key.
pub async fn list_organization_members(
    base_url: &str,
    management_key: &str,
    pagination: Option<PaginationOptions>,
) -> Result<OrganizationMembersResponse, OpenRouterError> {
    let url = with_pagination(format!("{base_url}/organization/members"), pagination);
    let response = with_bearer_auth(surf::get(url), management_key).await?;

    if response.status().is_success() {
        parse_json_response(response, "organization members").await
    } else {
        handle_error(response).await?;
        unreachable!()
    }
}
