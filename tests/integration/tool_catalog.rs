//! Live coverage for the server tool catalog (`GET /tools`, `GET /tools/{name}`).
//!
//! These are read-only, cheap calls, so they run whenever an API key is exported
//! (the `Live Contract Checks` workflow and `just test-live-contract`) and skip
//! in keyless runs.

use crate::test_utils::{create_test_client, live_api_key_available};
use openrouter_rs::{api::tools::ServerToolApiFormat, types::PaginationOptions};

const CATALOG_PAGE: u32 = 5;

fn catalog_client() -> Option<openrouter_rs::OpenRouterClient> {
    if !live_api_key_available() {
        eprintln!("skipping server tool catalog check: no API key configured");
        return None;
    }
    Some(create_test_client().expect("test client should build"))
}

#[tokio::test]
async fn test_tool_catalog_returns_paginated_server_tools() {
    let Some(client) = catalog_client() else {
        return;
    };

    let page = client
        .tools()
        .list(
            Some(PaginationOptions::new(Some(0), Some(CATALOG_PAGE))),
            None,
        )
        .await
        .expect("catalog list should succeed");

    assert!(
        !page.data.is_empty(),
        "the catalog should list at least one server tool"
    );
    assert!(
        page.data.len() as u32 <= CATALOG_PAGE,
        "limit should cap the page, got {} entries",
        page.data.len()
    );
    assert!(
        page.total_count as usize >= page.data.len(),
        "total_count {} should cover the page",
        page.total_count
    );

    for tool in &page.data {
        assert!(
            tool.id.starts_with("openrouter:"),
            "tool ids are canonical openrouter:* names, got {}",
            tool.id
        );
        assert!(!tool.name.is_empty(), "{} should have a label", tool.id);
        assert!(
            !tool.aliases.is_empty(),
            "{} should accept at least one tools[].type spelling",
            tool.id
        );
        assert!(
            tool.aliases.iter().any(|alias| alias.input_type == tool.id),
            "{} should accept its own canonical name",
            tool.id
        );
        assert!(
            tool.aliases
                .iter()
                .all(|alias| !alias.api_formats.is_empty()),
            "{} aliases should declare the formats that accept them",
            tool.id
        );
        assert!(!tool.docs_url.is_empty(), "{} should link docs", tool.id);
        assert!(
            tool.parameters_schema.is_object(),
            "{} should describe its parameters",
            tool.name
        );
        assert!(
            tool.native_support.models.is_none(),
            "the list endpoint reports counts, not models"
        );
        for engine in &tool.engines {
            assert!(
                !engine.name.is_empty(),
                "{} engines should be named",
                tool.id
            );
        }
    }
}

#[tokio::test]
async fn test_tool_catalog_filters_by_api_format() {
    let Some(client) = catalog_client() else {
        return;
    };

    let page = client
        .tools()
        .list(None, Some(ServerToolApiFormat::Responses))
        .await
        .expect("filtered catalog list should succeed");

    assert!(
        !page.data.is_empty(),
        "some server tools should work on the Responses API"
    );
    for tool in &page.data {
        assert!(
            tool.supported_api_formats
                .iter()
                .any(|format| format == "responses"),
            "{} leaked into the responses filter with {:?}",
            tool.id,
            tool.supported_api_formats
        );
    }
}

#[tokio::test]
async fn test_tool_lookup_accepts_canonical_name_and_alias() {
    let Some(client) = catalog_client() else {
        return;
    };

    let page = client
        .tools()
        .list(Some(PaginationOptions::new(Some(0), Some(1))), None)
        .await
        .expect("catalog list should succeed");
    let listed = page
        .data
        .first()
        .expect("the catalog should list at least one server tool")
        .clone();

    let by_name = client
        .tools()
        .get(&listed.id)
        .await
        .expect("lookup by canonical name should succeed");
    assert_eq!(by_name.id, listed.id);
    let native = by_name
        .native_support
        .models
        .as_ref()
        .expect("the detail endpoint adds the models that run the tool natively");
    assert_eq!(
        native.len() as u32,
        by_name.native_support.model_count,
        "model_count should match the listed models"
    );

    if let Some(alias) = listed
        .aliases
        .iter()
        .find(|alias| alias.input_type != listed.id)
    {
        let alias = &alias.input_type;
        let by_alias = client
            .tools()
            .get(alias)
            .await
            .expect("lookup by tools[].type alias should succeed");
        assert_eq!(
            by_alias.id, listed.id,
            "alias {alias} should resolve to {}",
            listed.id
        );
    }
}
