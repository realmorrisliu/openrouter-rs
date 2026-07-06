use openrouter_rs::types::tool::{FunctionDefinition, ServerTool, Tool, ToolChoice};
use serde_json::json;

#[test]
fn test_tool_builder_preserves_fields_when_name_is_last() {
    let tool = Tool::builder()
        .description("Get current weather for a location")
        .parameters(json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            }
        }))
        .name("get_weather")
        .build()
        .expect("tool should build when name is set last");

    assert_eq!(tool.function.name, "get_weather");
    assert_eq!(
        tool.function.description,
        "Get current weather for a location"
    );
    assert_eq!(tool.function.parameters["type"], "object");
    assert_eq!(
        tool.function.parameters["properties"]["location"]["type"],
        "string"
    );
}

#[test]
fn test_tool_builder_renaming_does_not_reset_other_fields() {
    let tool = Tool::builder()
        .name("draft_name")
        .description("Performs a calculation")
        .parameters(json!({"type": "object"}))
        .name("calculator")
        .build()
        .expect("tool should build after rename");

    assert_eq!(tool.function.name, "calculator");
    assert_eq!(tool.function.description, "Performs a calculation");
    assert_eq!(tool.function.parameters["type"], "object");
}

#[test]
fn test_tool_builder_accepts_full_function_definition_override() {
    let function = FunctionDefinition::builder()
        .name("lookup_user")
        .description("Find a user by id")
        .parameters(json!({
            "type": "object",
            "required": ["user_id"]
        }))
        .build()
        .expect("function definition should build");

    let tool = Tool::builder()
        .tool_type("function")
        .function(function)
        .build()
        .expect("tool should build from full function definition");

    assert_eq!(tool.function.name, "lookup_user");
    assert_eq!(tool.function.description, "Find a user by id");
    assert_eq!(tool.function.parameters["required"][0], "user_id");
}

#[test]
fn test_tool_builder_serializes_strict_and_cache_control() {
    let tool = Tool::builder()
        .name("lookup_user")
        .description("Find a user by id")
        .parameters(json!({"type": "object"}))
        .strict(true)
        .cache_control(json!({"type": "ephemeral", "ttl": "1h"}))
        .build()
        .expect("tool should build");

    let value = serde_json::to_value(tool).expect("tool should serialize");
    assert_eq!(value["type"], "function");
    assert_eq!(value["function"]["strict"], true);
    assert_eq!(value["cache_control"]["type"], "ephemeral");
    assert_eq!(value["cache_control"]["ttl"], "1h");
}

#[test]
fn test_server_tool_helpers_serialize_openapi_shape() {
    let tool = ServerTool::web_search_with_parameters(json!({
        "max_results": 5,
        "search_context_size": "high"
    }))
    .option("allowed_domains", json!(["example.com"]));

    let value = serde_json::to_value(tool).expect("server tool should serialize");
    assert_eq!(value["type"], "openrouter:web_search");
    assert_eq!(value["parameters"]["max_results"], 5);
    assert_eq!(value["parameters"]["search_context_size"], "high");
    assert_eq!(value["allowed_domains"][0], "example.com");
}

#[test]
fn test_server_tool_choice_serializes_openapi_shape() {
    let choice = ToolChoice::force_server_tool("openrouter:web_search");
    let value = serde_json::to_value(choice).expect("tool choice should serialize");
    assert_eq!(value, json!({"type": "openrouter:web_search"}));
}

#[test]
fn test_tool_builder_requires_name() {
    let err = Tool::builder()
        .description("No name set")
        .parameters(json!({"type": "object"}))
        .build()
        .expect_err("tool should not build without a name");

    assert!(err.to_string().contains("Tool name is required"));
}
