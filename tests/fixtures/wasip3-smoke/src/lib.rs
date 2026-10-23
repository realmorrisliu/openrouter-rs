use futures_util::StreamExt;
use openrouter_rs::{
    OpenRouterClient,
    api::{
        chat::{ChatCompletionRequest, Message},
        files::UploadFileRequest,
    },
    error::OpenRouterError,
    types::Role,
};

wasip3::cli::command::export!(Smoke);
struct Smoke;

impl wasip3::exports::cli::run::Guest for Smoke {
    async fn run() -> Result<(), ()> {
        run().await.expect("WASI transport smoke test failed");
        Ok(())
    }
}

async fn run() -> Result<(), OpenRouterError> {
    let base_url = wasip3::cli::environment::get_environment()
        .into_iter()
        .find(|(key, _)| key == "OPENROUTER_TEST_URL")
        .expect("test server URL required")
        .1;
    let client = OpenRouterClient::builder()
        .base_url(base_url)
        .api_key("smoke-test-only")
        .build()?;
    let files = client.files().list(Some(1), Some("a b&c"), None).await?;
    assert!(files.data.is_empty());
    let upload = UploadFileRequest::builder()
        .filename("smoke.txt")
        .mime_type("text/plain")
        .content(b"upload body".to_vec())
        .build()?;
    assert_eq!(client.files().upload(&upload, None).await?.id, "uploaded");
    assert_eq!(
        client.files().download_content("binary", None).await?,
        b"a\0b"
    );
    assert!(matches!(
        client.files().download_content("error", None).await,
        Err(OpenRouterError::Api(_))
    ));
    assert!(matches!(
        client.files().download_content("truncated", None).await,
        Err(OpenRouterError::HttpRequest(_))
    ));

    let request = ChatCompletionRequest::builder()
        .model("stream")
        .messages(vec![Message::new(Role::User, "smoke")])
        .build()?;
    let mut stream = client.chat().stream(&request).await?;
    assert_eq!(
        stream.next().await.unwrap()?.choices[0].content(),
        Some("first")
    );
    // The server waits for this acknowledgement before sending the second chunk.
    client.files().download_content("ack", None).await?;
    assert_eq!(
        stream.next().await.unwrap()?.choices[0].content(),
        Some("second")
    );
    assert!(stream.next().await.is_none());
    drop(stream);

    let request = ChatCompletionRequest::builder()
        .model("cancel")
        .messages(vec![Message::new(Role::User, "smoke")])
        .build()?;
    let mut stream = client.chat().stream(&request).await?;
    assert!(stream.next().await.unwrap().is_ok());
    drop(stream);
    client.files().download_content("cancelled", None).await?;
    Ok(())
}
