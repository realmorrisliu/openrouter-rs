//! Server tool catalog (`/tools`, `/tools/{name}`).

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use openrouter_rs::{
    api::tools::{
        ServerToolApiFormat, ServerToolByok, ServerToolExecutedBy, ServerToolPricingSource,
        ServerToolStatus,
    },
    types::PaginationOptions,
};
use serde_json::json;

use crate::test_client;

struct CapturedRequest {
    request_line: String,
}

fn spawn_json_server(
    response_body: &str,
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
    let (tx, rx) = mpsc::channel::<CapturedRequest>();

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
        let request_line = request_text.lines().next().unwrap_or_default().to_string();
        tx.send(CapturedRequest { request_line })
            .expect("server should send captured request");

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("server should write response");
    });

    (format!("http://{addr}/api/v1"), rx, server)
}

fn tool_json(with_models: bool) -> serde_json::Value {
    let native_support = if with_models {
        json!({
            "endpoint_count": 18,
            "model_count": 1,
            "models": [{"slug": "openai/gpt-5", "providers": ["OpenAI"]}]
        })
    } else {
        json!({"endpoint_count": 18, "model_count": 12})
    };
    json!({
        "id": "openrouter:web_search",
        "name": "Web search",
        "summary": "Search the web",
        "tool_description": "Runs a web search",
        "status": "active",
        "docs_url": "https://openrouter.ai/docs/tools/web-search",
        "aliases": [
            {
                "type": "openrouter:web_search",
                "api_formats": ["responses", "chat-completions"],
                "output_names": [
                    {"api_format": "responses", "type": "web_search_call"},
                    {"api_format": "anthropic-messages", "name": "web_search", "type": "server_tool_use"}
                ]
            },
            {
                "type": "web_search_20250305",
                "api_formats": ["anthropic-messages"],
                "output_names": [{"api_format": "anthropic-messages", "name": "web_search", "type": "server_tool_use"}]
            }
        ],
        "supported_api_formats": ["responses", "chat-completions"],
        "default_engine": "exa",
        "fallback_engine": null,
        "parameters_schema": {"type": "object"},
        "input_schema": {"type": "object"},
        "output_schema": null,
        "engines": [{
            "id": "exa",
            "name": "Exa",
            "executed_by": "openrouter",
            "data_regions": ["us"],
            "byok": "optional",
            "default_mode": "auto",
            "pricing_source": "openrouter",
            "pricing_doc_url": null,
            "pricing": [{"mode": "search", "unit": "call", "price": "0.005", "max_units_per_call": 1}]
        }],
        "native_support": native_support
    })
}

#[test]
fn list_response_decodes_catalog_entries() {
    let raw = json!({
        "data": [tool_json(false)],
        "total_count": 1,
        "links": {"next": null}
    });

    let parsed: openrouter_rs::api::tools::ListToolsResponse =
        serde_json::from_value(raw).expect("catalog list should deserialize");
    assert_eq!(parsed.total_count, 1);
    assert!(parsed.links.next.is_none());

    let tool = &parsed.data[0];
    assert_eq!(tool.id, "openrouter:web_search");
    assert_eq!(tool.name, "Web search");
    assert_eq!(tool.status, ServerToolStatus::Active);
    assert_eq!(tool.aliases.len(), 2);
    assert_eq!(tool.aliases[0].input_type, "openrouter:web_search");
    assert_eq!(
        tool.aliases[0].api_formats,
        ["responses", "chat-completions"]
    );
    assert_eq!(
        tool.aliases[0].output_names[0].output_type,
        "web_search_call"
    );
    assert!(tool.aliases[0].output_names[0].name.is_none());
    assert_eq!(
        tool.aliases[1].output_names[0].name.as_deref(),
        Some("web_search")
    );
    assert_eq!(tool.default_engine.as_deref(), Some("exa"));
    assert!(tool.fallback_engine.is_none());
    assert!(tool.output_schema.is_none());
    assert_eq!(tool.native_support.model_count, 12);
    assert!(tool.native_support.models.is_none());

    let engine = &tool.engines[0];
    assert_eq!(engine.executed_by, ServerToolExecutedBy::OpenRouter);
    assert_eq!(engine.byok, ServerToolByok::Optional);
    assert_eq!(engine.pricing_source, ServerToolPricingSource::OpenRouter);
    assert_eq!(engine.pricing[0].price, "0.005");
    assert_eq!(engine.pricing[0].max_units_per_call, 1);
}

#[test]
fn detail_response_adds_native_models() {
    let parsed: openrouter_rs::api::tools::ServerTool =
        serde_json::from_value(tool_json(true)).expect("tool detail should deserialize");

    let models = parsed
        .native_support
        .models
        .expect("detail response carries native models");
    assert_eq!(models[0].slug, "openai/gpt-5");
    assert_eq!(models[0].providers, ["OpenAI"]);
}

#[test]
fn api_format_values_match_the_wire() {
    let cases = [
        (ServerToolApiFormat::Responses, "responses"),
        (ServerToolApiFormat::ChatCompletions, "chat-completions"),
        (ServerToolApiFormat::AnthropicMessages, "anthropic-messages"),
    ];
    for (value, wire) in cases {
        assert_eq!(serde_json::to_value(value).unwrap(), json!(wire));
        assert_eq!(
            serde_json::from_value::<ServerToolApiFormat>(json!(wire)).unwrap(),
            value
        );
    }

    assert_eq!(
        serde_json::to_value(ServerToolByok::Unsupported).unwrap(),
        json!("none")
    );
    assert_eq!(
        serde_json::to_value(ServerToolPricingSource::NoCharge).unwrap(),
        json!("none")
    );
    assert_eq!(
        serde_json::to_value(ServerToolExecutedBy::Client).unwrap(),
        json!("client")
    );
    assert_eq!(
        serde_json::to_value(ServerToolStatus::Deprecated).unwrap(),
        json!("deprecated")
    );
}

#[tokio::test]
async fn list_tools_sends_pagination_and_api_format_filter() {
    let (base_url, rx, server) =
        spawn_json_server(r#"{"data":[],"total_count":0,"links":{"next":null}}"#);

    let result = test_client(&base_url, "api-key")
        .tools()
        .list(
            Some(PaginationOptions::new(Some(10), Some(25))),
            Some(ServerToolApiFormat::ChatCompletions),
        )
        .await
        .expect("catalog list should succeed");
    assert_eq!(result.total_count, 0);

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/tools?offset=10&limit=25&api_format=chat-completions HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn list_tools_without_filters_uses_base_path() {
    let (base_url, rx, server) =
        spawn_json_server(r#"{"data":[],"total_count":0,"links":{"next":null}}"#);

    test_client(&base_url, "api-key")
        .tools()
        .list(None, None)
        .await
        .expect("catalog list should succeed");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(captured.request_line, "GET /api/v1/tools HTTP/1.1");

    server.join().expect("server thread should finish");
}

#[tokio::test]
async fn get_tool_encodes_the_name_and_unwraps_data() {
    let body = json!({"data": tool_json(true)}).to_string();
    let (base_url, rx, server) = spawn_json_server(&body);

    let tool = test_client(&base_url, "api-key")
        .tools()
        .get("openrouter:web_search")
        .await
        .expect("tool lookup should succeed");
    assert_eq!(tool.id, "openrouter:web_search");

    let captured = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("should capture request");
    assert_eq!(
        captured.request_line,
        "GET /api/v1/tools/openrouter%3Aweb_search HTTP/1.1"
    );

    server.join().expect("server thread should finish");
}
