use openrouter_rs::{
    api::{
        audio::{TranscriptionInputAudio, TranscriptionRequest},
        batches::{Batch, CreateBatchRequest},
        embeddings::EmbeddingRequest,
        models::ListModelsParams,
        videos::VideoGenerationResponse,
    },
    types::ReasoningConfig,
};
use serde_json::json;

#[test]
fn batch_source_is_exclusive_and_serializes_last() {
    let mut builder = CreateBatchRequest::builder();
    builder
        .endpoint("/v1/audio/speech")
        .model("speech/model")
        .input("gs://input/batch.jsonl")
        .completion_window("24h");
    let batch = builder.build().unwrap();
    let body = serde_json::to_value(batch).unwrap();
    assert_eq!(body["input"]["uri"], "gs://input/batch.jsonl");
    assert!(body.get("requests").is_none());
    let inline = serde_json::to_string(&builder.requests(Vec::new()).build().unwrap()).unwrap();
    assert!(inline.ends_with(r#""requests":[]}"#));
    assert!(!inline.contains(r#""input":"#));
}

#[test]
fn storage_batch_exposes_provider_output_uri() {
    let batch: Batch = serde_json::from_value(json!({
        "id":"batch-1","object":"batch","endpoint":"/v1/audio/speech","model":"tts",
        "completion_window":"24h","status":"completed","created_at":1,
        "request_counts":{"total":1,"completed":1,"failed":0},
        "results":null, "output":{"uri":"gs://bucket/result"}
    }))
    .unwrap();
    assert_eq!(batch.output.unwrap().uri, "gs://bucket/result");
    assert!(batch.results.is_none());
}

#[test]
fn nested_embedding_strings_keep_their_grouping() {
    let request = EmbeddingRequest::builder()
        .model("model")
        .input(vec![vec!["first".to_string(), "second".to_string()]])
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(request).unwrap()["input"],
        json!([["first", "second"]])
    );
}

#[test]
fn benchmark_ranges_serialize_without_other_filters() {
    let params = ListModelsParams::builder()
        .min_engineering_index(12.5)
        .max_openness_index(80.0)
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(params).unwrap(),
        json!({"min_engineering_index":12.5, "max_openness_index":80.0})
    );
}

#[test]
fn reasoning_and_transcription_controls_serialize() {
    let mut reasoning = ReasoningConfig::enabled();
    reasoning.mode = Some("pro".into());
    reasoning.context = Some("current_turn".into());
    assert_eq!(
        serde_json::to_value(reasoning).unwrap(),
        json!({"enabled":true,"mode":"pro","context":"current_turn"})
    );
    let transcription = TranscriptionRequest::builder()
        .model("stt")
        .input_audio(TranscriptionInputAudio::new("audio", "wav"))
        .prompt("medical terminology")
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(transcription).unwrap()["prompt"],
        "medical terminology"
    );
}

#[test]
fn video_poll_preserves_fractional_progress() {
    let response: VideoGenerationResponse = serde_json::from_value(json!({
        "id":"job","polling_url":"/videos/job","status":"in_progress",
        "status_detail":"generating","progress":30.5
    }))
    .unwrap();
    assert_eq!(response.progress, Some(30.5));
    assert_eq!(response.status_detail.as_deref(), Some("generating"));
}
