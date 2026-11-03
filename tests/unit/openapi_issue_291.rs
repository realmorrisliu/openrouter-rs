//! Surface added by the 2026-11-02 OpenAPI drift: model `endpoints`, ZDR endpoint
//! `safety_retention`, Responses `access_programs`, and the upstream removal of
//! private-provider `data_policy`.

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
};

use openrouter_rs::{
    api::{
        discovery::{PublicEndpoint, UserModel},
        models::Model,
        private_providers::CreatePrivateProviderRequest,
        responses::{AccessPrograms, ResponsesRequest, ResponsesResponse},
    },
    types::ApiResponse,
};
use serde_json::{Value, json};

use crate::test_client;

const MODEL: &str = r#"{
    "data": {
        "id": "openai/gpt-4",
        "canonical_slug": "openai/gpt-4",
        "name": "GPT-4",
        "created": 1692901234,
        "description": "",
        "context_length": 8192,
        "architecture": {"modality": "text->text"},
        "top_provider": {"context_length": 8192, "max_completion_tokens": 4096, "is_moderated": true},
        "pricing": {"prompt": "0.00003", "completion": "0.00006"},
        "supported_parameters": [],
        "endpoints": ["/api/v1/chat/completions", "/api/v1/responses"]
    }
}"#;

#[test]
fn models_report_the_paths_that_serve_them() {
    let payload: ApiResponse<Model> = serde_json::from_str(MODEL).unwrap();
    assert_eq!(
        payload.data.endpoints,
        ["/api/v1/chat/completions", "/api/v1/responses"]
    );

    let user_model: ApiResponse<UserModel> = serde_json::from_str(MODEL).unwrap();
    assert_eq!(user_model.data.endpoints.len(), 2);

    // An older payload without `endpoints` still decodes.
    let bare: ApiResponse<Model> =
        serde_json::from_value(json!({"data": {"id": "a", "canonical_slug": "a", "name": "A", "created": 1,
            "architecture": {}, "top_provider": {"context_length": 1, "max_completion_tokens": 1, "is_moderated": false},
            "pricing": {"prompt": "0", "completion": "0"}}}))
        .unwrap();
    assert!(bare.data.endpoints.is_empty());
}

#[test]
fn zdr_endpoints_report_safety_retention() {
    let payload: ApiResponse<Vec<PublicEndpoint>> = serde_json::from_value(json!({
        "data": [{
            "name": "OpenAI: GPT-4",
            "model_id": "openai/gpt-4",
            "model_name": "GPT-4",
            "context_length": 8192,
            "pricing": {"prompt": "0.00003", "completion": "0.00006"},
            "provider_name": "OpenAI",
            "tag": "openai",
            "supports_implicit_caching": true,
            "supported_parameters": [],
            "safety_retention": true
        }]
    }))
    .unwrap();
    assert_eq!(payload.data[0].safety_retention, Some(true));

    // The same endpoint type backs `GET /models/{author}/{slug}/endpoints`,
    // which does not send the field.
    let payload: ApiResponse<Vec<PublicEndpoint>> = serde_json::from_value(json!({
        "data": [{
            "name": "OpenAI: GPT-4",
            "model_id": "openai/gpt-4",
            "model_name": "GPT-4",
            "context_length": 8192,
            "pricing": {"prompt": "0.00003", "completion": "0.00006"},
            "provider_name": "OpenAI",
            "tag": "openai",
            "supports_implicit_caching": true,
            "supported_parameters": []
        }]
    }))
    .unwrap();
    assert_eq!(payload.data[0].safety_retention, None);
}

fn spawn_json_server(
    response_body: String,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener binds");
    let addr = listener.local_addr().expect("local address");
    let (tx, rx) = mpsc::channel();
    let task = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 2048];
        let header_end = loop {
            let count = stream.read(&mut chunk).expect("read request");
            if count == 0 {
                break None;
            }
            bytes.extend_from_slice(&chunk[..count]);
            if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                break Some(end + 4);
            }
        }
        .expect("request has headers");
        let head = String::from_utf8_lossy(&bytes[..header_end]).to_string();
        let mut body = String::from_utf8_lossy(&bytes[header_end..]).to_string();
        let length = head
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())?
            })
            .unwrap_or(0);
        while body.len() < length {
            let count = stream.read(&mut chunk).expect("read body");
            if count == 0 {
                break;
            }
            body.push_str(&String::from_utf8_lossy(&chunk[..count]));
        }
        tx.send(body).expect("send captured body");
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });
    (format!("http://{addr}/api/v1"), rx, task)
}

#[tokio::test]
async fn responses_carry_access_programs() {
    let body = r#"{"id":"resp_1","access_programs":{"cyber":"daybreak_blue"}}"#.to_string();
    let (base_url, rx, server) = spawn_json_server(body);
    let client = test_client(&base_url, "api-key");

    let request = ResponsesRequest::builder()
        .model("openai/gpt-4")
        .input(json!("hi"))
        .access_programs(
            AccessPrograms::builder()
                .cyber("daybreak_blue")
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    let response: ResponsesResponse = client.responses().create(&request).await.unwrap();

    let captured: Value = serde_json::from_str(&rx.recv().unwrap()).unwrap();
    assert_eq!(
        captured["access_programs"],
        json!({"cyber": "daybreak_blue"})
    );
    assert_eq!(
        response.extra.get("access_programs"),
        Some(&json!({"cyber": "daybreak_blue"}))
    );

    // Omitting the field keeps it off the wire.
    let request = ResponsesRequest::builder()
        .model("openai/gpt-4")
        .input(json!("hi"))
        .build()
        .unwrap();
    assert!(
        serde_json::to_value(&request)
            .unwrap()
            .get("access_programs")
            .is_none()
    );
    server.join().expect("finish server");
}

#[test]
fn private_provider_requests_drop_data_policy() {
    let request = CreatePrivateProviderRequest::builder()
        .name("acme")
        .build()
        .unwrap();
    assert_eq!(
        serde_json::to_value(&request).unwrap(),
        json!({"name": "acme"})
    );
}
