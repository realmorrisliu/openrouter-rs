use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::api::videos::{
    self, VideoFrameImage, VideoGenerationRequest, VideoGenerationResponse, VideoInputReference,
    VideoModel, VideoProviderOptions,
};

#[test]
fn test_video_generation_request_serialization() {
    let mut provider_options = HashMap::new();
    provider_options.insert(
        "google-vertex".to_string(),
        serde_json::json!({
            "output_config": {
                "effort": "low"
            }
        }),
    );

    let request = VideoGenerationRequest::builder()
        .model("google/veo-3.1")
        .prompt("A serene mountain landscape at sunset")
        .aspect_ratio("16:9")
        .duration(8)
        .resolution("720p")
        .callback_url("https://example.com/webhooks/video")
        .generate_audio(true)
        .frame_images(vec![VideoFrameImage::new(
            "https://example.com/first.png",
            "first_frame",
        )])
        .input_references(vec![
            VideoInputReference::image("https://example.com/reference.png"),
            VideoInputReference::audio("https://example.com/reference.wav"),
            VideoInputReference::video("https://example.com/reference.mp4"),
        ])
        .provider(VideoProviderOptions::new(provider_options))
        .build()
        .expect("video generation request should build");

    let value = serde_json::to_value(&request).expect("video request should serialize");
    assert_eq!(value["model"], "google/veo-3.1");
    assert_eq!(value["prompt"], "A serene mountain landscape at sunset");
    assert_eq!(value["aspect_ratio"], "16:9");
    assert_eq!(value["callback_url"], "https://example.com/webhooks/video");
    assert_eq!(value["frame_images"][0]["frame_type"], "first_frame");
    assert_eq!(
        value["input_references"][0]["image_url"]["url"],
        "https://example.com/reference.png"
    );
    assert_eq!(
        value["input_references"][1]["audio_url"]["url"],
        "https://example.com/reference.wav"
    );
    assert_eq!(
        value["input_references"][2]["video_url"]["url"],
        "https://example.com/reference.mp4"
    );
    assert_eq!(
        value["provider"]["options"]["google-vertex"]["output_config"]["effort"],
        "low"
    );
}

#[test]
fn test_video_generation_response_deserialization() {
    let raw = r#"{
        "id": "job-abc123",
        "polling_url": "/api/v1/videos/job-abc123",
        "status": "completed",
        "generation_id": "gen-xyz789",
        "unsigned_urls": ["https://storage.example.com/video.mp4"],
        "usage": {
            "cost": 0.5,
            "is_byok": false
        }
    }"#;

    let parsed: VideoGenerationResponse =
        serde_json::from_str(raw).expect("video generation response should deserialize");
    assert_eq!(parsed.id, "job-abc123");
    assert_eq!(parsed.status, "completed");
    assert_eq!(
        parsed.usage.expect("usage should be present").is_byok,
        Some(false)
    );
}

#[test]
fn test_video_models_list_deserialization() {
    let raw = r#"{
        "data": [{
            "id": "google/veo-3.1",
            "canonical_slug": "google/veo-3.1",
            "name": "Veo 3.1",
            "created": 1700000000,
            "description": "Google video generation model",
            "allowed_passthrough_parameters": [],
            "generate_audio": true,
            "seed": null,
            "supported_aspect_ratios": ["16:9"],
            "supported_durations": [5, 8],
            "supported_frame_images": ["first_frame", "last_frame"],
            "supported_resolutions": ["720p"],
            "supported_sizes": null,
            "pricing_skus": {"generate":"0.50"}
        }]
    }"#;

    let parsed: openrouter_rs::types::ApiResponse<Vec<VideoModel>> =
        serde_json::from_str(raw).expect("video models should deserialize");
    assert_eq!(parsed.data.len(), 1);
    assert_eq!(parsed.data[0].canonical_slug, "google/veo-3.1");
    assert_eq!(parsed.data[0].supported_durations, Some(vec![5, 8]));
}

#[tokio::test]
async fn test_create_video_generation_path_body_and_headers() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<(String, String, String)>();

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
        tx.send((request_line, request_text, body_text))
            .expect("server should send captured request");

        let response = r#"{
            "id":"job-abc123",
            "polling_url":"/api/v1/videos/job-abc123",
            "status":"pending"
        }"#;
        let response = format!(
            "HTTP/1.1 202 Accepted\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response.len(),
            response
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let request = VideoGenerationRequest::builder()
        .model("google/veo-3.1")
        .prompt("A serene mountain landscape at sunset")
        .resolution("720p")
        .build()
        .expect("video request should build");

    let response = videos::create_video_generation(
        &base_url,
        "api-key",
        &Some("openrouter-rs".to_string()),
        &Some("https://example.com".to_string()),
        &Some(vec!["cli-agent".to_string()]),
        &request,
    )
    .await
    .expect("video generation request should succeed");
    assert_eq!(response.id, "job-abc123");
    assert_eq!(response.status, "pending");

    let (request_line, request_text, body_text) = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(request_line, "POST /api/v1/videos HTTP/1.1");

    let body_json: serde_json::Value =
        serde_json::from_str(&body_text).expect("body should be valid json");
    assert_eq!(body_json["model"], "google/veo-3.1");
    assert_eq!(body_json["prompt"], "A serene mountain landscape at sunset");
    assert_eq!(body_json["resolution"], "720p");

    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include api key, request:\n{}",
        request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_list_video_models_path_and_auth_header() {
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
    let models = videos::list_video_models(&base_url, "api-key")
        .await
        .expect("list video models should succeed");
    assert!(models.is_empty());

    let request_text = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    let request_line = request_text.lines().next().unwrap_or_default().to_string();
    assert_eq!(request_line, "GET /api/v1/videos/models HTTP/1.1");

    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include api key, request:\n{}",
        request_text
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_video_generation_path_and_auth_header() {
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

        let response = r#"{
            "id":"job-abc123",
            "polling_url":"/api/v1/videos/job-abc123",
            "status":"completed"
        }"#;
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
    let response = videos::get_video_generation(&base_url, "api-key", "job-abc123")
        .await
        .expect("get video generation should succeed");
    assert_eq!(response.status, "completed");

    let request_text = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    let request_line = request_text.lines().next().unwrap_or_default().to_string();
    assert_eq!(request_line, "GET /api/v1/videos/job-abc123 HTTP/1.1");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_get_video_content_path_query_and_auth_header() {
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

        let body = b"video-bytes";
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(header.as_bytes())
            .expect("server should write response header");
        stream
            .write_all(body)
            .expect("server should write response body");
    });

    let base_url = format!("http://{addr}/api/v1");
    let content = videos::get_video_content(&base_url, "api-key", "job-abc123", Some(1))
        .await
        .expect("get video content should succeed");
    assert_eq!(content, b"video-bytes");

    let request_text = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    let request_line = request_text.lines().next().unwrap_or_default().to_string();
    assert_eq!(
        request_line,
        "GET /api/v1/videos/job-abc123/content?index=1 HTTP/1.1"
    );

    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include api key, request:\n{}",
        request_text
    );

    server.join().expect("server thread should finish");
}
