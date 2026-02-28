use openrouter_rs::api::embeddings::{
    EmbeddingContentPart, EmbeddingInput, EmbeddingMultimodalInput, EmbeddingRequest,
    EmbeddingResponse, EmbeddingVector,
};

#[test]
fn test_embedding_request_text_input_serialize() {
    let request = EmbeddingRequest::builder()
        .model("openai/text-embedding-3-large")
        .input("hello embeddings")
        .dimensions(1024)
        .user("user-123")
        .input_type("query")
        .build()
        .expect("embedding request should build");

    let value = serde_json::to_value(&request).expect("embedding request should serialize");
    assert_eq!(value["model"], "openai/text-embedding-3-large");
    assert_eq!(value["input"], "hello embeddings");
    assert_eq!(value["dimensions"], 1024);
    assert_eq!(value["user"], "user-123");
    assert_eq!(value["input_type"], "query");
}

#[test]
fn test_embedding_request_multimodal_input_serialize() {
    let input = EmbeddingInput::MultimodalArray(vec![EmbeddingMultimodalInput {
        content: vec![
            EmbeddingContentPart::Text {
                text: "caption this".to_string(),
            },
            EmbeddingContentPart::ImageUrl {
                image_url: openrouter_rs::api::embeddings::EmbeddingImageUrl {
                    url: "https://example.com/image.jpg".to_string(),
                },
            },
        ],
    }]);

    let request = EmbeddingRequest::new("openai/text-embedding-3-large", input);
    let value = serde_json::to_value(&request).expect("embedding request should serialize");

    assert_eq!(value["input"][0]["content"][0]["type"], "text");
    assert_eq!(value["input"][0]["content"][0]["text"], "caption this");
    assert_eq!(value["input"][0]["content"][1]["type"], "image_url");
    assert_eq!(
        value["input"][0]["content"][1]["image_url"]["url"],
        "https://example.com/image.jpg"
    );
}

#[test]
fn test_embedding_response_float_deserialization() {
    let raw = r#"{
        "id": "emb-001",
        "object": "list",
        "data": [
            {"object":"embedding","embedding":[0.1,0.2,0.3],"index":0}
        ],
        "model": "openai/text-embedding-3-large",
        "usage": {"prompt_tokens": 8, "total_tokens": 8, "cost": 0.00001}
    }"#;

    let response: EmbeddingResponse =
        serde_json::from_str(raw).expect("embedding response should deserialize");
    assert_eq!(response.object, "list");
    assert_eq!(response.data.len(), 1);

    match &response.data[0].embedding {
        EmbeddingVector::Float(values) => assert_eq!(values.len(), 3),
        EmbeddingVector::Base64(_) => panic!("expected float vector"),
    }
}

#[test]
fn test_embedding_response_base64_deserialization() {
    let raw = r#"{
        "object": "list",
        "data": [
            {"object":"embedding","embedding":"AAAAAA==","index":0}
        ],
        "model": "openai/text-embedding-3-large",
        "usage": {"prompt_tokens": 8, "total_tokens": 8}
    }"#;

    let response: EmbeddingResponse =
        serde_json::from_str(raw).expect("embedding response should deserialize");
    assert_eq!(response.object, "list");

    match &response.data[0].embedding {
        EmbeddingVector::Base64(value) => assert_eq!(value, "AAAAAA=="),
        EmbeddingVector::Float(_) => panic!("expected base64 vector"),
    }
}
