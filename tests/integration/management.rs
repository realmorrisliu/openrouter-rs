use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use openrouter_rs::{
    api::guardrails::{CreateGuardrailRequest, UpdateGuardrailRequest},
    error::OpenRouterError,
};

use super::test_utils::{
    create_management_test_client, rate_limit_delay, should_run_management_tests,
};

const MAX_LIST_PROBES: usize = 5;

static SMOKE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn smoke_name(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let seq = SMOKE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("openrouter-rs-{prefix}-{millis}-{seq}")
}

fn smoke_assert(condition: bool, message: impl Into<String>) -> Result<(), OpenRouterError> {
    if condition {
        Ok(())
    } else {
        Err(OpenRouterError::Unknown(message.into()))
    }
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_management_api_keys_smoke_lifecycle() -> Result<(), OpenRouterError> {
    if !should_run_management_tests() {
        println!(
            "Skipping management keys smoke test; set OPENROUTER_RUN_MANAGEMENT_TESTS=1 to enable"
        );
        return Ok(());
    }

    let Some(client) = create_management_test_client()? else {
        println!(
            "Skipping management keys smoke test; OPENROUTER_MANAGEMENT_KEY is not configured"
        );
        return Ok(());
    };

    let management = client.management();
    let created_name = smoke_name("mgmt-key");
    let updated_name = format!("{created_name}-updated");

    let mut created_hash: Option<String> = None;

    let lifecycle_result: Result<(), OpenRouterError> = async {
        rate_limit_delay().await;
        let created = management.create_api_key(&created_name, None).await?;
        let hash = created.hash.clone().unwrap_or_default().trim().to_string();
        smoke_assert(!hash.is_empty(), "created API key hash should not be empty")?;
        created_hash = Some(hash.clone());

        let mut seen_in_list = false;
        for _ in 0..MAX_LIST_PROBES {
            rate_limit_delay().await;
            let keys = management.list_api_keys(None, Some(true)).await?;
            if keys.iter().any(|key| {
                key.hash
                    .as_deref()
                    .is_some_and(|candidate| candidate.trim() == hash)
            }) {
                seen_in_list = true;
                break;
            }
        }
        smoke_assert(
            seen_in_list,
            "created API key hash should appear in /keys listing",
        )?;

        rate_limit_delay().await;
        let fetched = management.get_api_key(&hash).await?;
        smoke_assert(
            fetched
                .hash
                .as_deref()
                .is_some_and(|fetched_hash| fetched_hash.trim() == hash),
            "GET /keys/{{hash}} should return the created hash",
        )?;

        rate_limit_delay().await;
        let updated = management
            .update_api_key(&hash, Some(updated_name.clone()), Some(false), None)
            .await?;
        smoke_assert(
            updated
                .name
                .as_deref()
                .is_some_and(|name| name.trim() == updated_name),
            "PATCH /keys/{{hash}} should persist updated name",
        )?;

        rate_limit_delay().await;
        let deleted = management.delete_api_key(&hash).await?;
        smoke_assert(deleted, "DELETE /keys/{{hash}} should report success")?;
        created_hash = None;

        println!("Management keys smoke lifecycle passed (hash={hash})");
        Ok(())
    }
    .await;

    let cleanup_error = if let Some(hash) = created_hash.take() {
        rate_limit_delay().await;
        management.delete_api_key(&hash).await.err()
    } else {
        None
    };

    match (lifecycle_result, cleanup_error) {
        (Ok(()), None) => Ok(()),
        (Ok(()), Some(err)) => Err(err),
        (Err(err), None) => Err(err),
        (Err(primary), Some(cleanup)) => Err(OpenRouterError::Unknown(format!(
            "management key lifecycle failed ({primary}); cleanup also failed ({cleanup})"
        ))),
    }
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_management_guardrails_smoke_lifecycle() -> Result<(), OpenRouterError> {
    if !should_run_management_tests() {
        println!(
            "Skipping management guardrails smoke test; set OPENROUTER_RUN_MANAGEMENT_TESTS=1 to enable"
        );
        return Ok(());
    }

    let Some(client) = create_management_test_client()? else {
        println!(
            "Skipping management guardrails smoke test; OPENROUTER_MANAGEMENT_KEY is not configured"
        );
        return Ok(());
    };

    let management = client.management();
    let created_name = smoke_name("guardrail");
    let updated_name = format!("{created_name}-updated");

    let mut created_guardrail_id: Option<String> = None;

    let lifecycle_result: Result<(), OpenRouterError> = async {
        let create_request = CreateGuardrailRequest::builder()
            .name(created_name.clone())
            .description("openrouter-rs management smoke guardrail")
            .build()?;

        rate_limit_delay().await;
        let created = management.create_guardrail(&create_request).await?;
        let guardrail_id = created.id.trim().to_string();
        smoke_assert(
            !guardrail_id.is_empty(),
            "created guardrail id should not be empty",
        )?;
        created_guardrail_id = Some(guardrail_id.clone());

        let mut seen_in_list = false;
        for _ in 0..MAX_LIST_PROBES {
            rate_limit_delay().await;
            let listed = management.list_guardrails(None).await?;
            if listed
                .data
                .iter()
                .any(|guardrail| guardrail.id.trim() == guardrail_id)
            {
                seen_in_list = true;
                break;
            }
        }
        smoke_assert(
            seen_in_list,
            "created guardrail id should appear in /guardrails listing",
        )?;

        rate_limit_delay().await;
        let fetched = management.get_guardrail(&guardrail_id).await?;
        smoke_assert(
            fetched.id.trim() == guardrail_id,
            "GET /guardrails/{id} should return the created guardrail",
        )?;

        let update_request = UpdateGuardrailRequest::builder()
            .name(updated_name.clone())
            .description("openrouter-rs management smoke guardrail updated")
            .build()?;

        rate_limit_delay().await;
        let updated = management
            .update_guardrail(&guardrail_id, &update_request)
            .await?;
        smoke_assert(
            updated.name.trim() == updated_name,
            "PATCH /guardrails/{id} should persist updated name",
        )?;

        rate_limit_delay().await;
        let deleted = management.delete_guardrail(&guardrail_id).await?;
        smoke_assert(deleted, "DELETE /guardrails/{{id}} should report success")?;
        created_guardrail_id = None;

        println!("Management guardrails smoke lifecycle passed (id={guardrail_id})");
        Ok(())
    }
    .await;

    let cleanup_error = if let Some(id) = created_guardrail_id.take() {
        rate_limit_delay().await;
        management.delete_guardrail(&id).await.err()
    } else {
        None
    };

    match (lifecycle_result, cleanup_error) {
        (Ok(()), None) => Ok(()),
        (Ok(()), Some(err)) => Err(err),
        (Err(err), None) => Err(err),
        (Err(primary), Some(cleanup)) => Err(OpenRouterError::Unknown(format!(
            "management guardrail lifecycle failed ({primary}); cleanup also failed ({cleanup})"
        ))),
    }
}
