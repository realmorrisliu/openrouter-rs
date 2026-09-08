use openrouter_rs::error::OpenRouterError;

use super::test_utils::create_test_client;

/// Requires an existing container with a downloadable generated file.
#[tokio::test]
#[allow(clippy::result_large_err)]
async fn test_container_files_live() -> Result<(), OpenRouterError> {
    let Ok(container_id) = std::env::var("OPENROUTER_TEST_CONTAINER_ID") else {
        println!("Skipping container files: OPENROUTER_TEST_CONTAINER_ID is not set");
        return Ok(());
    };
    let client = create_test_client()?;
    let list = client
        .files()
        .list_container_files(&container_id, None, Some(1))
        .await?;
    let file = list
        .data
        .first()
        .expect("test container must contain a file");
    let metadata = client
        .files()
        .get_container_file(&container_id, &file.id)
        .await?;
    assert_eq!(metadata.id, file.id);
    let content = client
        .files()
        .download_container_file(&container_id, &file.id)
        .await?;
    assert_eq!(content.len() as u64, metadata.bytes);
    Ok(())
}
