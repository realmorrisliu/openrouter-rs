use openrouter_rs::types::{ResponseFormat, ResponseFormatType};
use serde_json::json;

#[test]
fn test_response_format_text_serialize() {
    let format = ResponseFormat::text();
    let value = serde_json::to_value(&format).expect("text format should serialize");
    assert_eq!(value["type"], "text");
}

#[test]
fn test_response_format_json_object_serialize() {
    let format = ResponseFormat::json_object();
    let value = serde_json::to_value(&format).expect("json_object format should serialize");
    assert_eq!(value["type"], "json_object");
}

#[test]
fn test_response_format_json_schema_serialize() {
    let format = ResponseFormat::json_schema(
        "math_response",
        true,
        json!({
            "type": "object",
            "properties": {
                "answer": { "type": "number" }
            },
            "required": ["answer"]
        }),
    );

    let value = serde_json::to_value(&format).expect("json_schema format should serialize");
    assert_eq!(value["type"], "json_schema");
    assert_eq!(value["json_schema"]["name"], "math_response");
    assert_eq!(value["json_schema"]["strict"], true);
}

#[test]
fn test_response_format_grammar_serialize() {
    let format = ResponseFormat::grammar("root ::= \"yes\" | \"no\"");
    let value = serde_json::to_value(&format).expect("grammar format should serialize");
    assert_eq!(value["type"], "grammar");
    assert_eq!(value["grammar"], "root ::= \"yes\" | \"no\"");
}

#[test]
fn test_response_format_python_serialize() {
    let format = ResponseFormat::python();
    let value = serde_json::to_value(&format).expect("python format should serialize");
    assert_eq!(value["type"], "python");
}

#[test]
fn test_response_format_legacy_type_only_deserialize() {
    let format: ResponseFormat = serde_json::from_str("\"json_object\"")
        .expect("legacy type-only format should deserialize");

    match format {
        ResponseFormat::TypeOnly(ResponseFormatType::JsonObject) => {}
        _ => panic!("expected legacy TypeOnly(JsonObject)"),
    }
}
