use openrouter_rs::{
    OpenRouterClient,
    api::batches::{BatchRequest, CreateBatchRequest},
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenRouterClient::builder()
        .api_key(std::env::var("OPENROUTER_API_KEY")?)
        .build()?;
    let body =
        json!({"model":"openai/gpt-4o-mini","messages":[{"role":"user","content":"Say hello."}]});
    let request = CreateBatchRequest::builder()
        .endpoint("/v1/chat/completions")
        .model("openai/gpt-4o-mini")
        .requests(vec![BatchRequest::new(
            "hello-1",
            body.as_object().unwrap().clone(),
        )])
        .build()?;
    let batch = client.batches().create(&request).await?;
    println!("{}: {}", batch.id, batch.status);
    // Poll get(&batch.id) later for results. Deletion is for terminal batches only.
    Ok(())
}
