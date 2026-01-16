//! Unit tests for the OpenAI provider.

use any_llm::{Content, ContentPart, Message, ProviderConfig, Role, Tool, ToolCall, ToolChoice};
use serde_json::json;

/// Test that we can create an OpenAI provider with config.
#[test]
fn test_openai_provider_creation_with_api_key() {
    let config = ProviderConfig::new("test-api-key");
    let provider = any_llm::providers::openai::OpenAIProvider::new(config);
    assert!(provider.is_ok());
}

/// Test that we can create an OpenAI provider with custom API base.
#[test]
fn test_openai_provider_creation_with_api_base() {
    let config = ProviderConfig::new("test-api-key").with_api_base("https://custom.openai.com/v1");
    let provider = any_llm::providers::openai::OpenAIProvider::new(config);
    assert!(provider.is_ok());
}

/// Test that provider creation fails without API key.
#[test]
fn test_openai_provider_creation_without_api_key_fails() {
    // Clear environment variable temporarily for this test
    let original = std::env::var("OPENAI_API_KEY").ok();
    std::env::remove_var("OPENAI_API_KEY");

    let config = ProviderConfig::default();
    let result = any_llm::providers::openai::OpenAIProvider::new(config);
    assert!(result.is_err());

    // Restore if it existed
    if let Some(key) = original {
        std::env::set_var("OPENAI_API_KEY", key);
    }
}

/// Test message creation helpers.
#[test]
fn test_message_system() {
    let msg = Message::system("You are a helpful assistant.");
    assert_eq!(msg.role, Role::System);
    assert!(matches!(&msg.content, Some(Content::Text(t)) if t == "You are a helpful assistant."));
}

/// Test user message creation.
#[test]
fn test_message_user() {
    let msg = Message::user("Hello!");
    assert_eq!(msg.role, Role::User);
    assert!(matches!(&msg.content, Some(Content::Text(t)) if t == "Hello!"));
}

/// Test user message with image.
#[test]
fn test_message_user_with_image() {
    let parts = vec![
        ContentPart::text("What's in this image?"),
        ContentPart::image_url("https://example.com/image.png"),
    ];
    let msg = Message::user_with_parts(parts);

    assert_eq!(msg.role, Role::User);
    match &msg.content {
        Some(Content::Parts(parts)) => {
            assert_eq!(parts.len(), 2);
            assert!(
                matches!(&parts[0], ContentPart::Text { text } if text == "What's in this image?")
            );
            assert!(matches!(&parts[1], ContentPart::ImageUrl { .. }));
        }
        _ => panic!("Expected Parts content"),
    }
}

/// Test user message with base64 image.
#[test]
fn test_message_user_with_base64_image() {
    let parts = vec![
        ContentPart::text("Describe this:"),
        ContentPart::image_base64("abc123data", "image/png"),
    ];
    let msg = Message::user_with_parts(parts);

    match &msg.content {
        Some(Content::Parts(parts)) => {
            assert_eq!(parts.len(), 2);
            if let ContentPart::ImageUrl { image_url } = &parts[1] {
                assert!(image_url.is_base64());
                let (media_type, data) = image_url.parse_base64().unwrap();
                assert_eq!(media_type, "image/png");
                assert_eq!(data, "abc123data");
            } else {
                panic!("Expected ImageUrl part");
            }
        }
        _ => panic!("Expected Parts content"),
    }
}

/// Test assistant message with tool calls.
#[test]
fn test_message_assistant_with_tool_calls() {
    let tool_calls = vec![ToolCall::new(
        "call_123",
        "get_weather",
        r#"{"location":"Paris"}"#,
    )];
    let msg = Message::assistant_with_tool_calls(tool_calls);

    assert_eq!(msg.role, Role::Assistant);
    assert!(msg.content.is_none());
    assert!(msg.tool_calls.is_some());

    let calls = msg.tool_calls.unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].id, "call_123");
    assert_eq!(calls[0].function.name, "get_weather");
}

/// Test tool message creation.
#[test]
fn test_message_tool() {
    let msg = Message::tool("call_123", r#"{"temperature": "22C"}"#);
    assert_eq!(msg.role, Role::Tool);
    assert_eq!(msg.tool_call_id, Some("call_123".to_string()));
}

/// Test tool choice auto.
#[test]
fn test_tool_choice_auto() {
    let choice = ToolChoice::auto();
    assert_eq!(choice.as_mode(), Some("auto"));
}

/// Test tool choice required.
#[test]
fn test_tool_choice_required() {
    let choice = ToolChoice::required();
    assert_eq!(choice.as_mode(), Some("required"));
}

/// Test tool choice specific function.
#[test]
fn test_tool_choice_specific_function() {
    let choice = ToolChoice::function("my_function");
    assert_eq!(choice.as_function(), Some("my_function"));
}

/// Test tool creation with builder.
#[test]
fn test_tool_creation() {
    let tool = Tool::function("get_weather", "Get weather for a location")
        .parameters(json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city name"
                }
            },
            "required": ["location"]
        }))
        .build();

    assert_eq!(tool.tool_type, "function");
    assert_eq!(tool.function.name, "get_weather");
    assert_eq!(
        tool.function.description,
        Some("Get weather for a location".to_string())
    );
    assert!(tool.function.parameters.is_some());
}

/// Test tool call argument parsing.
#[test]
fn test_tool_call_parse_arguments() {
    let tc = ToolCall::new("id", "func", r#"{"key": "value"}"#);

    #[derive(serde::Deserialize)]
    struct Args {
        key: String,
    }

    let args: Args = tc.parse_arguments().unwrap();
    assert_eq!(args.key, "value");
}

/// Test error conversion patterns.
#[test]
fn test_error_messages() {
    let err = any_llm::AnyLLMError::rate_limit("openai", "Too many requests");
    assert!(err.to_string().contains("Rate limit"));
    assert!(err.to_string().contains("openai"));

    let err = any_llm::AnyLLMError::authentication("openai", "Invalid API key");
    assert!(err.to_string().contains("Authentication"));

    let err = any_llm::AnyLLMError::model_not_found("openai", "gpt-5");
    assert!(err.to_string().contains("not found"));
    assert!(err.to_string().contains("gpt-5"));
}
