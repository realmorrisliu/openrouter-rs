use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use http::StatusCode;
use openrouter_rs::api::audio::{
    self, SpeechProviderOptions, SpeechRequest, SpeechResponseFormat, TranscriptionInputAudio,
    TranscriptionProviderOptions, TranscriptionRequest,
};

#[test]
fn test_speech_request_serialization() {
    let mut provider_options = HashMap::new();
    provider_options.insert(
        "openai".to_string(),
        serde_json::json!({
            "instructions": "Speak clearly"
        }),
    );

    let request = SpeechRequest::builder()
        .model("elevenlabs/eleven-turbo-v2")
        .input("Hello world")
        .voice("alloy")
        .response_format(SpeechResponseFormat::Mp3)
        .speed(1.1)
        .provider(SpeechProviderOptions::new(provider_options))
        .build()
        .expect("speech request should build");

    let value = serde_json::to_value(&request).expect("speech request should serialize");
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

#[test]
fn test_transcription_request_serialization() {
    let mut provider_options = HashMap::new();
    provider_options.insert(
        "openai".to_string(),
        serde_json::json!({
            "prompt": "Use product names verbatim"
        }),
    );

    let request = TranscriptionRequest::builder()
        .model("openai/whisper-large-v3")
        .input_audio(TranscriptionInputAudio::new("UklGRiQA...", "wav"))
        .language("en")
        .temperature(0.0)
        .provider(TranscriptionProviderOptions::new(provider_options))
        .build()
        .expect("transcription request should build");

    let value = serde_json::to_value(&request).expect("transcription request should serialize");
    assert_eq!(value["model"], "openai/whisper-large-v3");
    assert_eq!(value["input_audio"]["data"], "UklGRiQA...");
    assert_eq!(value["input_audio"]["format"], "wav");
    assert_eq!(value["language"], "en");
    assert_eq!(value["temperature"], 0.0);
    assert_eq!(
        value["provider"]["options"]["openai"]["prompt"],
        "Use product names verbatim"
    );
}

#[tokio::test]
async fn test_create_transcription_path_body_headers_and_response() {
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
            "text": "Hello from audio",
            "usage": {
                "cost": 0.000508,
                "input_tokens": 83,
                "output_tokens": 30,
                "seconds": 9.2,
                "total_tokens": 113
            }
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
    let request = TranscriptionRequest::builder()
        .model("openai/whisper-large-v3")
        .input_audio(TranscriptionInputAudio::new("UklGRiQA...", "wav"))
        .language("en")
        .build()
        .expect("transcription request should build");

    let response = audio::create_transcription(
        &base_url,
        "api-key",
        &Some("openrouter-rs".to_string()),
        &Some("https://example.com".to_string()),
        &Some(vec!["cli-agent".to_string()]),
        &request,
    )
    .await
    .expect("transcription request should succeed");
    assert_eq!(response.text, "Hello from audio");
    assert_eq!(
        response.usage.as_ref().and_then(|usage| usage.total_tokens),
        Some(113)
    );

    let (request_line, request_text, body_text) = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(request_line, "POST /api/v1/audio/transcriptions HTTP/1.1");

    let body_json: serde_json::Value =
        serde_json::from_str(&body_text).expect("body should be valid json");
    assert_eq!(body_json["model"], "openai/whisper-large-v3");
    assert_eq!(body_json["input_audio"]["data"], "UklGRiQA...");
    assert_eq!(body_json["input_audio"]["format"], "wav");
    assert_eq!(body_json["language"], "en");

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

#[tokio::test]
async fn test_create_speech_path_body_headers_and_binary_response() {
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
    let request = SpeechRequest::builder()
        .model("elevenlabs/eleven-turbo-v2")
        .input("Hello world")
        .voice("alloy")
        .response_format(SpeechResponseFormat::Mp3)
        .speed(1.0)
        .build()
        .expect("speech request should build");

    let response = audio::create_speech(
        &base_url,
        "api-key",
        &Some("openrouter-rs".to_string()),
        &Some("https://example.com".to_string()),
        &Some(vec!["cli-agent".to_string()]),
        &request,
    )
    .await
    .expect("speech request should succeed");
    assert_eq!(response, b"ID3fake-audio-data");

    let (request_line, request_text, body_text) = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(request_line, "POST /api/v1/audio/speech HTTP/1.1");

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

#[tokio::test]
async fn test_create_speech_falls_back_to_legacy_path_when_official_path_is_missing() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<String>();
    let audio_bytes = b"ID3legacy-fallback".to_vec();

    let server = thread::spawn(move || {
        for attempt in 0..2 {
            let (mut stream, _) = listener.accept().expect("server should accept request");
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
            let request_line = request_text.lines().next().unwrap_or_default().to_string();
            let content_length = request_text
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
            let header_len = request_bytes
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|pos| pos + 4)
                .unwrap_or(request_bytes.len());
            let mut body_bytes = request_bytes[header_len..].to_vec();
            while body_bytes.len() < content_length {
                let read = stream
                    .read(&mut chunk)
                    .expect("server should drain request body");
                if read == 0 {
                    break;
                }
                body_bytes.extend_from_slice(&chunk[..read]);
            }

            tx.send(request_line.clone())
                .expect("server should send captured request");

            if attempt == 0 {
                let body = r#"{"error":{"code":404,"message":"Not Found"}}"#;
                let response = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("server should write fallback trigger response");
            } else {
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
            }
        }
    });

    let base_url = format!("http://{addr}/api/v1");
    let request = SpeechRequest::builder()
        .model("elevenlabs/eleven-turbo-v2")
        .input("Hello world")
        .voice("alloy")
        .response_format(SpeechResponseFormat::Mp3)
        .build()
        .expect("speech request should build");

    let response = audio::create_speech(&base_url, "api-key", &None, &None, &None, &request)
        .await
        .expect("speech request should fall back and succeed");
    assert_eq!(response, b"ID3legacy-fallback");

    let first_request_line = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture official request");
    let second_request_line = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture fallback request");
    assert_eq!(first_request_line, "POST /api/v1/audio/speech HTTP/1.1");
    assert_eq!(second_request_line, "POST /api/v1/tts HTTP/1.1");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_create_speech_does_not_fall_back_for_request_level_404() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<String>();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("server should accept request");
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
        let request_line = request_text.lines().next().unwrap_or_default().to_string();
        tx.send(request_line)
            .expect("server should send captured request");

        let body = r#"{"error":{"code":404,"message":"Model not found"}}"#;
        let response = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write error response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let request = SpeechRequest::builder()
        .model("elevenlabs/eleven-turbo-v2")
        .input("Hello world")
        .voice("alloy")
        .response_format(SpeechResponseFormat::Mp3)
        .build()
        .expect("speech request should build");

    let error = audio::create_speech(&base_url, "api-key", &None, &None, &None, &request)
        .await
        .expect_err("speech request should surface the official error");

    match error {
        openrouter_rs::error::OpenRouterError::Api(api_error) => {
            assert_eq!(api_error.status, StatusCode::NOT_FOUND);
            assert_eq!(api_error.message, "Model not found");
        }
        other => panic!("expected api error, got {other:?}"),
    }

    let first_request_line = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture official request");
    assert_eq!(first_request_line, "POST /api/v1/audio/speech HTTP/1.1");
    assert!(
        rx.recv_timeout(Duration::from_millis(250)).is_err(),
        "should not issue a legacy fallback request for request-level errors"
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_create_speech_does_not_fall_back_for_plain_text_request_level_404() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<String>();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("server should accept request");
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
        let request_line = request_text.lines().next().unwrap_or_default().to_string();
        tx.send(request_line)
            .expect("server should send captured request");

        let body = "404 not found: invalid model";
        let response = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write error response");
    });

    let base_url = format!("http://{addr}/api/v1");
    let request = SpeechRequest::builder()
        .model("elevenlabs/eleven-turbo-v2")
        .input("Hello world")
        .voice("alloy")
        .response_format(SpeechResponseFormat::Mp3)
        .build()
        .expect("speech request should build");

    let error = audio::create_speech(&base_url, "api-key", &None, &None, &None, &request)
        .await
        .expect_err("speech request should surface the official error");

    match error {
        openrouter_rs::error::OpenRouterError::Api(api_error) => {
            assert_eq!(api_error.status, StatusCode::NOT_FOUND);
            assert_eq!(api_error.message, "404 not found: invalid model");
        }
        other => panic!("expected api error, got {other:?}"),
    }

    let first_request_line = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture official request");
    assert_eq!(first_request_line, "POST /api/v1/audio/speech HTTP/1.1");
    assert!(
        rx.recv_timeout(Duration::from_millis(250)).is_err(),
        "should not issue a legacy fallback request for plain-text request-level errors"
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn test_create_speech_falls_back_for_plain_text_404_page_not_found() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener
        .local_addr()
        .expect("listener should have local addr");
    let (tx, rx) = mpsc::channel::<String>();
    let audio_bytes = b"ID3plain-text-fallback".to_vec();

    let server = thread::spawn(move || {
        for attempt in 0..2 {
            let (mut stream, _) = listener.accept().expect("server should accept request");
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
            let request_line = request_text.lines().next().unwrap_or_default().to_string();
            tx.send(request_line.clone())
                .expect("server should send captured request");

            if attempt == 0 {
                let body = "404 page not found";
                let response = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("server should write fallback trigger response");
            } else {
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
            }
        }
    });

    let base_url = format!("http://{addr}/api/v1");
    let request = SpeechRequest::builder()
        .model("elevenlabs/eleven-turbo-v2")
        .input("Hello world")
        .voice("alloy")
        .response_format(SpeechResponseFormat::Mp3)
        .build()
        .expect("speech request should build");

    let response = audio::create_speech(&base_url, "api-key", &None, &None, &None, &request)
        .await
        .expect("speech request should fall back and succeed");
    assert_eq!(response, b"ID3plain-text-fallback");

    let first_request_line = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture official request");
    let second_request_line = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture fallback request");
    assert_eq!(first_request_line, "POST /api/v1/audio/speech HTTP/1.1");
    assert_eq!(second_request_line, "POST /api/v1/tts HTTP/1.1");

    server.join().expect("server thread should finish");
}
