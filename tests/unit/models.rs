use openrouter_rs::{api::models::EndpointData, types::ApiResponse};

#[test]
fn test_model_endpoints_pricing_allows_missing_optional_fields() {
    let raw = r#"{
        "data": {
            "id": "qwen/qwen3.5-35b-a3b",
            "name": "Qwen 3.5 35B A3B",
            "created": 1735689600,
            "description": "Test endpoint data",
            "architecture": {
                "tokenizer": "Qwen",
                "instruct_type": "chat",
                "modality": "text->text"
            },
            "endpoints": [{
                "name": "Qwen: Qwen 3.5 35B A3B",
                "context_length": 262144,
                "pricing": {
                    "prompt": "0.00000025",
                    "completion": "0.000002",
                    "discount": 0
                },
                "provider_name": "Qwen",
                "supported_parameters": ["temperature", "top_p"],
                "quantization": null,
                "max_completion_tokens": 16384,
                "max_prompt_tokens": 262144,
                "status": 0
            }]
        }
    }"#;

    let parsed: ApiResponse<EndpointData> =
        serde_json::from_str(raw).expect("models endpoints payload should deserialize");

    assert_eq!(parsed.data.id, "qwen/qwen3.5-35b-a3b");
    assert_eq!(parsed.data.endpoints.len(), 1);

    let pricing = &parsed.data.endpoints[0].pricing;
    assert_eq!(pricing.prompt, "0.00000025");
    assert_eq!(pricing.completion, "0.000002");
    assert!(pricing.request.is_none());
    assert!(pricing.image.is_none());
}
