use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use futures_util::StreamExt;
use openrouter_rs::api::images::{
    self, ImageGenerationRequest, ImageGenerationResponse, ImageInputReference, ImageModel,
    ImageModelEndpointsResponse, ImageProviderOptions, ImageStreamEvent, ImageStreamingResponse,
};

struct CapturedRequest {
    request_line: String,
    request_text: String,
    body_text: String,
}

fn spawn_server(
    response_body: &str,
    content_type: &str,
) -> (
    String,
    mpsc::Receiver<CapturedRequest>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let body = response_body.to_string();
    let content_type = content_type.to_string();
    let (tx, rx) = mpsc::channel::<CapturedRequest>();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("server should accept one connection");

        let mut request_bytes = Vec::new();
        let mut chunk = [0_u8; 1024];
        let header_end = loop {
            let read = stream.read(&mut chunk).expect("server should read request");
            if read == 0 {
                break None;
            }
            request_bytes.extend_from_slice(&chunk[..read]);
            if let Some(pos) = request_bytes
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
            {
                break Some(pos + 4);
            }
        }
        .expect("request should contain header terminator");

        let header_text = String::from_utf8_lossy(&request_bytes[..header_end]).to_string();
        let request_line = header_text.lines().next().unwrap_or_default().to_string();
        let content_length = header_text
            .lines()
            .find_map(|line| {
                let lower = line.to_ascii_lowercase();
                if lower.starts_with("content-length:") {
                    line.split(':').nth(1)?.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let mut body_bytes = request_bytes[header_end..].to_vec();
        while body_bytes.len() < content_length {
            let read = stream
                .read(&mut chunk)
                .expect("server should read request body");
            if read == 0 {
                break;
            }
            body_bytes.extend_from_slice(&chunk[..read]);
        }

        let body_text = String::from_utf8_lossy(&body_bytes[..content_length]).to_string();
        let request_text = format!("{header_text}{body_text}");
        tx.send(CapturedRequest {
            request_line,
            request_text,
            body_text,
        })
        .expect("server should send captured request");

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            content_type,
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    (format!("http://{addr}/api/v1"), rx, server)
}

#[test]
fn test_image_generation_request_serialization() {
    let mut provider_options = HashMap::new();
    provider_options.insert(
        "black-forest-labs".to_string(),
        serde_json::json!({
            "guidance": 3,
            "steps": 40
        }),
    );

    let request = ImageGenerationRequest::builder()
        .model("bytedance-seed/seedream-4.5")
        .prompt("a red panda astronaut floating in space")
        .aspect_ratio("16:9")
        .background("transparent")
        .input_references(vec![ImageInputReference::new(
            "https://example.com/reference.png",
        )])
        .n(2)
        .output_compression(80)
        .output_format("webp")
        .provider(ImageProviderOptions::new(provider_options))
        .quality("high")
        .resolution("2K")
        .seed(42)
        .size("2048x1152")
        .build()
        .expect("image generation request should build");

    let value = serde_json::to_value(&request).expect("image request should serialize");
    assert_eq!(value["model"], "bytedance-seed/seedream-4.5");
    assert_eq!(value["prompt"], "a red panda astronaut floating in space");
    assert_eq!(value["aspect_ratio"], "16:9");
    assert_eq!(value["input_references"][0]["type"], "image_url");
    assert_eq!(
        value["input_references"][0]["image_url"]["url"],
        "https://example.com/reference.png"
    );
    assert_eq!(
        value["provider"]["options"]["black-forest-labs"]["steps"],
        40
    );
    assert!(value.get("stream").is_none());
}

#[test]
fn test_image_generation_response_deserialization() {
    let raw = r#"{
        "created": 1748372400,
        "data": [{
            "b64_json": "aW1hZ2U=",
            "media_type": "image/svg+xml"
        }],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 4175,
            "total_tokens": 4175,
            "cost": 0.04,
            "is_byok": false
        }
    }"#;

    let parsed: ImageGenerationResponse =
        serde_json::from_str(raw).expect("image generation response should deserialize");
    assert_eq!(parsed.created, 1748372400);
    assert_eq!(parsed.data[0].b64_json, "aW1hZ2U=");
    assert_eq!(parsed.data[0].media_type.as_deref(), Some("image/svg+xml"));
    assert_eq!(
        parsed.usage.expect("usage should be present").cost,
        Some(0.04)
    );
}

#[test]
fn test_image_models_and_endpoints_deserialization() {
    let models_raw = r#"{
        "data": [{
            "id": "bytedance-seed/seedream-4.5",
            "name": "Seedream 4.5",
            "description": "A text-to-image model.",
            "created": 1692901234,
            "architecture": {
                "input_modalities": ["text", "image"],
                "output_modalities": ["image"]
            },
            "supported_parameters": {
                "resolution": {"type": "enum", "values": ["1K", "2K"]},
                "seed": {"type": "boolean"}
            },
            "supports_streaming": false,
            "endpoints": "/api/v1/images/models/bytedance-seed/seedream-4.5/endpoints"
        }]
    }"#;

    let parsed: openrouter_rs::types::ApiResponse<Vec<ImageModel>> =
        serde_json::from_str(models_raw).expect("image models should deserialize");
    assert_eq!(parsed.data[0].id, "bytedance-seed/seedream-4.5");
    assert_eq!(
        parsed.data[0]
            .supported_parameters
            .get("resolution")
            .expect("resolution should exist")
            .values
            .as_deref(),
        Some(&["1K".to_string(), "2K".to_string()][..])
    );

    let endpoints_raw = r#"{
        "id": "bytedance-seed/seedream-4.5",
        "endpoints": [{
            "provider_name": "Bytedance",
            "provider_slug": "bytedance",
            "provider_tag": null,
            "supported_parameters": {
                "resolution": {"type": "enum", "values": ["1K"]}
            },
            "allowed_passthrough_parameters": [],
            "supports_streaming": false,
            "pricing": [{
                "billable": "output_image",
                "unit": "image",
                "cost_usd": 0.05
            }]
        }]
    }"#;

    let endpoints: ImageModelEndpointsResponse =
        serde_json::from_str(endpoints_raw).expect("image endpoints should deserialize");
    assert_eq!(endpoints.endpoints[0].provider_name, "Bytedance");
    assert_eq!(endpoints.endpoints[0].provider_tag, None);
    assert_eq!(endpoints.endpoints[0].pricing[0].cost_usd, 0.05);
}

#[test]
fn test_image_streaming_response_deserialization() {
    let raw = r#"{
        "data": {
            "type": "image_generation.partial_image",
            "partial_image_index": 0,
            "b64_json": "cGFydGlhbA=="
        }
    }"#;

    let parsed: ImageStreamingResponse =
        serde_json::from_str(raw).expect("streaming response should deserialize");
    match parsed.data {
        ImageStreamEvent::PartialImage(event) => {
            assert_eq!(event.partial_image_index, 0);
            assert_eq!(event.b64_json, "cGFydGlhbA==");
        }
        other => panic!("expected partial image event, got {other:?}"),
    }
}

#[tokio::test]
async fn test_create_image_generation_path_body_and_headers() {
    let response = r#"{
        "created": 1748372400,
        "data": [{"b64_json":"aW1hZ2U="}]
    }"#;
    let (base_url, rx, server) = spawn_server(response, "application/json");
    let request = ImageGenerationRequest::builder()
        .model("bytedance-seed/seedream-4.5")
        .prompt("a red panda astronaut")
        .resolution("2K")
        .build()
        .expect("image request should build");

    let response = images::create_image_generation(
        &base_url,
        "api-key",
        &Some("openrouter-rs".to_string()),
        &Some("https://example.com".to_string()),
        &Some(vec!["cli-agent".to_string()]),
        &request,
    )
    .await
    .expect("image generation should succeed");
    assert_eq!(response.data[0].b64_json, "aW1hZ2U=");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "POST /api/v1/images HTTP/1.1");
    let body_json: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("body should be valid json");
    assert_eq!(body_json["model"], "bytedance-seed/seedream-4.5");
    assert_eq!(body_json["prompt"], "a red panda astronaut");
    assert_eq!(body_json["stream"], false);

    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include api key, request:\n{}",
        captured.request_text
    );
    assert!(
        request_lower.contains("x-openrouter-title: openrouter-rs"),
        "metadata header should be sent, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_stream_image_generation_path_body_and_sse() {
    let response = concat!(
        "data: {\"data\":{\"type\":\"image_generation.partial_image\",\"partial_image_index\":0,\"b64_json\":\"cGFydGlhbA==\"}}\n\n",
        "data: [DONE]\n\n"
    );
    let (base_url, rx, server) = spawn_server(response, "text/event-stream");
    let request = ImageGenerationRequest::builder()
        .model("openai/gpt-image-1")
        .prompt("stream an image")
        .build()
        .expect("image request should build");

    let mut stream =
        images::stream_image_generation(&base_url, "api-key", &None, &None, &None, &request)
            .await
            .expect("stream should open");
    let event = stream
        .next()
        .await
        .expect("stream should emit one event")
        .expect("event should deserialize");
    assert!(matches!(event.data, ImageStreamEvent::PartialImage(_)));
    assert!(stream.next().await.is_none());

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "POST /api/v1/images HTTP/1.1");
    let body_json: serde_json::Value =
        serde_json::from_str(&captured.body_text).expect("body should be valid json");
    assert_eq!(body_json["stream"], true);

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_image_models_path_and_auth_header() {
    let (base_url, rx, server) = spawn_server(r#"{"data":[]}"#, "application/json");
    let models = images::list_image_models(&base_url, "api-key")
        .await
        .expect("list image models should succeed");
    assert!(models.is_empty());

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/images/models HTTP/1.1");
    let request_lower = captured.request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include api key, request:\n{}",
        captured.request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_image_model_endpoints_path_and_auth_header() {
    let response = r#"{"id":"author/model","endpoints":[]}"#;
    let (base_url, rx, server) = spawn_server(response, "application/json");
    let endpoints = images::list_image_model_endpoints(&base_url, "api-key", "author", "model")
        .await
        .expect("list image endpoints should succeed");
    assert_eq!(endpoints.id, "author/model");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/images/models/author/model/endpoints HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}
