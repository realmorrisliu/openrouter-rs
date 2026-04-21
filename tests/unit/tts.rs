use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::api::tts::{self, TtsProviderOptions, TtsRequest, TtsResponseFormat};

#[test]
fn test_tts_request_serialization() {
    let mut provider_options = HashMap::new();
    provider_options.insert(
        "openai".to_string(),
        serde_json::json!({
            "instructions": "Speak clearly"
        }),
    );

    let request = TtsRequest::builder()
        .model("elevenlabs/eleven-turbo-v2")
        .input("Hello world")
        .voice("alloy")
        .response_format(TtsResponseFormat::Mp3)
        .speed(1.1)
        .provider(TtsProviderOptions {
            options: Some(provider_options),
        })
        .build()
        .expect("tts request should build");

    let value = serde_json::to_value(&request).expect("tts request should serialize");
    assert_eq!(value["model"], "elevenlabs/eleven-turbo-v2");
    assert_eq!(value["input"], "Hello world");
    assert_eq!(value["voice"], "alloy");
    assert_eq!(value["response_format"], "mp3");
    assert_eq!(value["speed"], 1.1);
    assert_eq!(
        value["provider"]["options"]["openai"]["instructions"],
        "Speak clearly"
    );
}

#[tokio::test]
async fn test_create_tts_path_body_headers_and_binary_response() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<(String, String, String)>();
    let audio_bytes = b"ID3fake-audio-data".to_vec();

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

        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: audio/mpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            audio_bytes.len()
        );
        stream
            .write_all(header.as_bytes())
            .expect("server should write response header");
        stream
            .write_all(&audio_bytes)
            .expect("server should write binary response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let request = TtsRequest::builder()
        .model("elevenlabs/eleven-turbo-v2")
        .input("Hello world")
        .voice("alloy")
        .response_format(TtsResponseFormat::Mp3)
        .speed(1.0)
        .build()
        .expect("tts request should build");

    let response = tts::create_tts(
        &base_url,
        "api-key",
        &Some("openrouter-rs".to_string()),
        &Some("https://example.com".to_string()),
        &Some(vec!["cli-agent".to_string()]),
        &request,
    )
    .await
    .expect("tts request should succeed");
    assert_eq!(response, b"ID3fake-audio-data");

    let (request_line, request_text, body_text) = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(request_line, "POST /api/v1/tts HTTP/1.1");

    let body_json: serde_json::Value =
        serde_json::from_str(&body_text).expect("body should be valid json");
    assert_eq!(body_json["model"], "elevenlabs/eleven-turbo-v2");
    assert_eq!(body_json["input"], "Hello world");
    assert_eq!(body_json["voice"], "alloy");
    assert_eq!(body_json["response_format"], "mp3");
    assert_eq!(body_json["speed"], 1.0);

    let request_lower = request_text.to_ascii_lowercase();
    assert!(
        request_lower.contains("authorization: bearer api-key")
            || request_lower.contains("authorization:bearer api-key"),
        "authorization header should include api key, request:\n{}",
        request_text
    );
    assert!(
        request_lower.contains("x-title: openrouter-rs")
            || request_lower.contains("x-title:openrouter-rs"),
        "x-title header should be present, request:\n{}",
        request_text
    );
    assert!(
        request_lower.contains("http-referer: https://example.com")
            || request_lower.contains("http-referer:https://example.com"),
        "http-referer header should be present, request:\n{}",
        request_text
    );
    assert!(
        request_lower.contains("x-openrouter-categories: cli-agent")
            || request_lower.contains("x-openrouter-categories:cli-agent"),
        "x-openrouter-categories header should be present, request:\n{}",
        request_text
    );

    server.join().expect("server thread should finish");
}
