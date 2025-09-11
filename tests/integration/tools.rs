use openrouter_rs::{
    api::chat::{ChatCompletionRequest, Message},
    types::{Role, Tool, ToolChoice},
};
use serde_json::json;

use crate::test_utils::create_test_client;

#[tokio::test]
async fn test_tool_definition_serialization() {
    let tool = Tool::builder()
        .name("test_function")
        .description("A test function")
        .parameters(json!({
            "type": "object",
            "properties": {
                "param1": {"type": "string"}
            },
            "required": ["param1"]
        }))
        .build()
        .unwrap();

    // Test serialization
    let serialized = serde_json::to_string(&tool).unwrap();
    assert!(serialized.contains("test_function"));
    assert!(serialized.contains("A test function"));

    // Test deserialization
    let deserialized: Tool = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.function.name, "test_function");
    assert_eq!(deserialized.function.description, "A test function");
}

#[tokio::test]
async fn test_tool_choice_serialization() {
    // Test string variants
    let auto_choice = ToolChoice::auto();
    let none_choice = ToolChoice::none();
    let required_choice = ToolChoice::required();

    assert_eq!(serde_json::to_string(&auto_choice).unwrap(), r#""auto""#);
    assert_eq!(serde_json::to_string(&none_choice).unwrap(), r#""none""#);
    assert_eq!(serde_json::to_string(&required_choice).unwrap(), r#""required""#);

    // Test specific tool choice
    let specific_choice = ToolChoice::force_tool("my_function");
    let serialized = serde_json::to_string(&specific_choice).unwrap();
    assert!(serialized.contains("my_function"));
    assert!(serialized.contains("function"));
}

#[tokio::test]
async fn test_chat_request_with_tools() {
    let tool = Tool::builder()
        .name("weather")
        .description("Get weather")
        .parameters(json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            },
            "required": ["location"]
        }))
        .build()
        .unwrap();

    let request = ChatCompletionRequest::builder()
        .model("openai/gpt-4o")
        .messages(vec![Message::new(Role::User, "What's the weather?")])
        .tool(tool)
        .tool_choice_auto()
        .parallel_tool_calls(true)
        .build()
        .unwrap();

    // Test that tools are properly included
    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("tools"));
    assert!(serialized.contains("weather"));
    assert!(serialized.contains("tool_choice"));
    assert!(serialized.contains("parallel_tool_calls"));
}

#[tokio::test]
async fn test_tool_message_creation() {
    // Test regular message
    let user_msg = Message::new(Role::User, "Hello");
    assert_eq!(user_msg.role, Role::User);
    assert_eq!(user_msg.content, "Hello");
    assert!(user_msg.name.is_none());
    assert!(user_msg.tool_call_id.is_none());

    // Test tool response message
    let tool_msg = Message::tool_response("call_123", "Tool result");
    assert_eq!(tool_msg.role, Role::Tool);
    assert_eq!(tool_msg.content, "Tool result");
    assert_eq!(tool_msg.tool_call_id, Some("call_123".to_string()));
    assert!(tool_msg.name.is_none());

    // Test named tool response
    let named_tool_msg = Message::tool_response_named("call_456", "weather", "Sunny");
    assert_eq!(named_tool_msg.role, Role::Tool);
    assert_eq!(named_tool_msg.content, "Sunny");
    assert_eq!(named_tool_msg.tool_call_id, Some("call_456".to_string()));
    assert_eq!(named_tool_msg.name, Some("weather".to_string()));
}

#[tokio::test]
async fn test_assistant_message_with_tool_calls() {
    use openrouter_rs::types::{ToolCall, FunctionCall};
    
    let tool_call = ToolCall {
        id: "call_123".to_string(),
        type_: "function".to_string(),
        function: FunctionCall {
            name: "test_function".to_string(),
            arguments: r#"{"param": "value"}"#.to_string(),
        },
    };
    
    let assistant_msg = Message::assistant_with_tool_calls("I'll help you with that", vec![tool_call]);
    assert_eq!(assistant_msg.role, Role::Assistant);
    assert_eq!(assistant_msg.content, "I'll help you with that");
    assert!(assistant_msg.tool_calls.is_some());
    assert_eq!(assistant_msg.tool_calls.as_ref().unwrap().len(), 1);
    assert_eq!(assistant_msg.tool_calls.as_ref().unwrap()[0].id, "call_123");
    assert_eq!(assistant_msg.tool_calls.as_ref().unwrap()[0].function.name, "test_function");

    // Test serialization
    let serialized = serde_json::to_string(&assistant_msg).unwrap();
    assert!(serialized.contains("tool_calls"));
    assert!(serialized.contains("call_123"));
    assert!(serialized.contains("test_function"));
}

#[tokio::test]
async fn test_tool_builder_methods() {
    let mut builder = ChatCompletionRequest::builder();
    
    let tool1 = Tool::new("tool1", "First tool", json!({"type": "object"}));
    let tool2 = Tool::new("tool2", "Second tool", json!({"type": "object"}));

    // Test adding single tool
    builder.tool(tool1);
    
    // Test adding another tool
    builder.tool(tool2);
    
    // Test setting tool choice
    builder.tool_choice_required();
    
    // Test setting parallel tool calls
    builder.parallel_tool_calls(false);

    let request = builder
        .model("test")
        .messages(vec![Message::new(Role::User, "test")])
        .build()
        .unwrap();

    // Verify tools were added
    assert!(request.tools().is_some());
    let tools = request.tools().unwrap();
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].function.name, "tool1");
    assert_eq!(tools[1].function.name, "tool2");

    // Verify tool choice
    assert!(request.tool_choice().is_some());
    
    // Verify parallel tool calls
    assert_eq!(request.parallel_tool_calls(), Some(false));
}

#[tokio::test]
async fn test_tool_helper_function() {
    let tool = openrouter_rs::types::tool::create_tool(
        "calculator",
        "Perform calculations",
        json!({
            "operation": {"type": "string"},
            "a": {"type": "number"},
            "b": {"type": "number"}
        }),
        &["operation", "a", "b"]
    );

    assert_eq!(tool.function.name, "calculator");
    assert_eq!(tool.function.description, "Perform calculations");
    
    let params = &tool.function.parameters;
    assert_eq!(params["type"], "object");
    assert_eq!(params["required"], json!(["operation", "a", "b"]));
    assert!(params["properties"]["operation"].is_object());
}

#[tokio::test]
#[ignore = "Requires API key and makes real API calls"]
async fn test_real_tool_call() {
    let client = create_test_client().unwrap();
    
    let tool = Tool::builder()
        .name("echo")
        .description("Echo back the input")
        .parameters(json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo back"
                }
            },
            "required": ["message"]
        }))
        .build()
        .unwrap();

    let request = ChatCompletionRequest::builder()
        .model("openai/gpt-4o-mini")  // Use a cheaper model for testing
        .messages(vec![
            Message::new(Role::System, "You are a helpful assistant. Use the echo tool when asked to echo something."),
            Message::new(Role::User, "Please echo 'Hello, World!'")
        ])
        .tool(tool)
        .tool_choice_auto()
        .max_tokens(200)
        .build()
        .unwrap();

    let response = client.send_chat_completion(&request).await.unwrap();
    
    // Check that we got a response
    assert!(!response.choices.is_empty());
    
    let choice = &response.choices[0];
    
    // The model should either call the tool or provide a response
    assert!(choice.tool_calls().is_some() || choice.content().is_some());
    
    if let Some(tool_calls) = choice.tool_calls() {
        assert!(!tool_calls.is_empty());
        let tool_call = &tool_calls[0];
        assert_eq!(tool_call.function.name, "echo");
        assert!(tool_call.function.arguments.contains("Hello, World!"));
    }
}