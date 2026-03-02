use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::api::embeddings::{
    self, EmbeddingContentPart, EmbeddingInput, EmbeddingMultimodalInput, EmbeddingRequest,
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

#[tokio::test]
async fn test_list_embedding_models_path_and_auth_header() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<String>();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("server should accept one connection");
        let mut request_bytes = Vec::new();
        let mut chunk = [0_u8; 1024];

        loop {
            let read = stream.read(&mut chunk).expect("server should read request");
            if read == 0 {
                break;
            }
            request_bytes.extend_from_slice(&chunk[..read]);
            if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let request_text = String::from_utf8_lossy(&request_bytes).to_string();
        tx.send(request_text)
            .expect("server should send request text");

        let response = r#"{"data":[]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response.len(),
            response
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let models = embeddings::list_embedding_models(&base_url, "api-key")
        .await
        .expect("list embedding models should succeed");
    assert!(models.is_empty(), "response payload should parse");

    let request_text = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    let request_line = request_text.lines().next().unwrap_or_default().to_string();
    assert_eq!(request_line, "GET /api/v1/embeddings/models HTTP/1.1");

    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include API key, request:\n{}",
        request_text
    );

    server.join().expect("server thread should finish");
}
