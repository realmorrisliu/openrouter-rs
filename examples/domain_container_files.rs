//! List files generated in a container, then promote one to reusable Files storage.
//! Set OPENROUTER_API_KEY and OPENROUTER_CONTAINER_ID. Promotion requires
//! OPENROUTER_PROMOTE_CONTAINER_FILE=1.
use openrouter_rs::{OpenRouterClient, error::OpenRouterError};

#[tokio::main]
async fn main() -> Result<(), OpenRouterError> {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY is required");
    let container_id =
        std::env::var("OPENROUTER_CONTAINER_ID").expect("OPENROUTER_CONTAINER_ID is required");
    let client = OpenRouterClient::builder().api_key(api_key).build()?;
    let files = client
        .files()
        .list_container_files(&container_id, None, Some(10))
        .await?;
    for file in &files.data {
        println!("{}: {} ({} bytes)", file.id, file.path, file.bytes);
    }
    if std::env::var("OPENROUTER_PROMOTE_CONTAINER_FILE").as_deref() == Ok("1") {
        if let Some(file) = files.data.first() {
            let promoted = client
                .files()
                .promote_container_file(&container_id, &file.id)
                .await?;
            println!("Reusable file: {}", promoted.id);
        }
    }
    Ok(())
}
