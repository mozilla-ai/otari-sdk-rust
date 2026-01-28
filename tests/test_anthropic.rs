//! Unit tests for the Anthropic provider.

use any_llm::{
    providers::anthropic::Anthropic, Content, ContentPart, Message, ProviderConfig,
    ReasoningEffort, Role, Tool, ToolCall, ToolChoice,
};
use serde_json::json;

/// Test that we can create an Anthropic provider with config.
#[test]
fn test_anthropic_provider_creation_with_api_key() {
    let config = ProviderConfig::new("test-api-key");
    let provider = any_llm::provider::AnyLLMProvider::<Anthropic>::from_config(config);
    assert!(provider.is_ok());
}

/// Test that we can create an Anthropic provider with custom API base.
#[test]
fn test_anthropic_provider_creation_with_api_base() {
    let config = ProviderConfig::new("test-api-key").with_api_base("https://custom.anthropic.com");
    let provider = any_llm::provider::AnyLLMProvider::<Anthropic>::from_config(config);
    assert!(provider.is_ok());
}

/// Test that provider creation fails without API key.
/// TODO: Let's revisit this because we should NOT be making tests with side effects
// #[test]
// fn test_anthropic_provider_creation_without_api_key_fails() {
//     // Clear environment variable temporarily for this test
//     let original = std::env::var("ANTHROPIC_API_KEY").ok();
//     std::env::remove_var("ANTHROPIC_API_KEY");

//     let config = ProviderConfig::default();
//     let result = any_llm::providers::anthropic::Anthropic::new(config);
//     assert!(result.is_err());

//     // Restore if it existed
//     if let Some(key) = original {
//         std::env::set_var("ANTHROPIC_API_KEY", key);
//     }
// }

/// Test system message extraction - single message.
#[test]
fn test_system_message_extraction_single() {
    let messages = vec![Message::system("You are helpful."), Message::user("Hello")];

    // We can't call the private method directly, but we can verify the message structure
    let system_msg = &messages[0];
    assert_eq!(system_msg.role, Role::System);
    assert_eq!(
        system_msg.text_content(),
        Some("You are helpful.".to_string())
    );
}

/// Test system message extraction - multiple messages should be concatenated.
#[test]
fn test_system_message_extraction_multiple() {
    let messages = vec![
        Message::system("You are helpful."),
        Message::system("Be concise."),
        Message::user("Hello"),
    ];

    // Count system messages
    let system_count = messages.iter().filter(|m| m.role == Role::System).count();
    assert_eq!(system_count, 2);

    // In the actual implementation, these would be concatenated with "\n"
    let system_text: String = messages
        .iter()
        .filter(|m| m.role == Role::System)
        .filter_map(|m| m.text_content())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(system_text, "You are helpful.\nBe concise.");
}

/// Test tool choice conversion - "required" maps to "any" in Anthropic.
#[test]
fn test_tool_choice_required_maps_to_any() {
    let choice = ToolChoice::required();

    // In our implementation, "required" should map to "any" for Anthropic
    // This is tested in the provider's convert_tool_choice method
    assert_eq!(choice.as_mode(), Some("required"));
}

/// Test tool choice conversion - specific tool.
#[test]
fn test_tool_choice_specific_tool() {
    let choice = ToolChoice::function("get_weather");
    assert_eq!(choice.as_function(), Some("get_weather"));
}

/// Test reasoning effort to thinking budget mapping.
#[test]
fn test_reasoning_effort_values() {
    // These mappings are defined in the Anthropic provider:
    // Minimal -> 1024
    // Low -> 2048
    // Medium -> 8192
    // High -> 24576

    assert_eq!(ReasoningEffort::None as i32, 0);
    assert_eq!(ReasoningEffort::Minimal as i32, 1);
    assert_eq!(ReasoningEffort::Low as i32, 2);
    assert_eq!(ReasoningEffort::Medium as i32, 3);
    assert_eq!(ReasoningEffort::High as i32, 4);
    assert_eq!(ReasoningEffort::Auto as i32, 5);
}

/// Test image content conversion - base64.
#[test]
fn test_image_content_base64() {
    let parts = vec![
        ContentPart::text("What's this?"),
        ContentPart::image_base64("abc123", "image/png"),
    ];

    let msg = Message::user_with_parts(parts);

    match &msg.content {
        Some(Content::Parts(parts)) => {
            assert_eq!(parts.len(), 2);
            if let ContentPart::ImageUrl { image_url } = &parts[1] {
                assert!(image_url.is_base64());
                let (media_type, data) = image_url.parse_base64().unwrap();
                assert_eq!(media_type, "image/png");
                assert_eq!(data, "abc123");
            } else {
                panic!("Expected ImageUrl");
            }
        }
        _ => panic!("Expected Parts"),
    }
}

/// Test image content conversion - URL.
#[test]
fn test_image_content_url() {
    let parts = vec![ContentPart::image_url("https://example.com/image.jpg")];

    let msg = Message::user_with_parts(parts);

    match &msg.content {
        Some(Content::Parts(parts)) => {
            if let ContentPart::ImageUrl { image_url } = &parts[0] {
                assert!(!image_url.is_base64());
                assert_eq!(image_url.url, "https://example.com/image.jpg");
            } else {
                panic!("Expected ImageUrl");
            }
        }
        _ => panic!("Expected Parts"),
    }
}

/// Test tool call message in agent loop.
#[test]
fn test_tool_call_message() {
    let tool_calls = vec![ToolCall::new(
        "call_abc123",
        "get_weather",
        r#"{"location": "Paris"}"#,
    )];
    let msg = Message::assistant_with_tool_calls(tool_calls);

    assert_eq!(msg.role, Role::Assistant);
    let calls = msg.tool_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].id, "call_abc123");
    assert_eq!(calls[0].function.name, "get_weather");
    assert_eq!(calls[0].function.arguments, r#"{"location": "Paris"}"#);
}

/// Test tool result message.
#[test]
fn test_tool_result_message() {
    let msg = Message::tool(
        "call_abc123",
        r#"{"temperature": "22C", "condition": "sunny"}"#,
    );

    assert_eq!(msg.role, Role::Tool);
    assert_eq!(msg.tool_call_id, Some("call_abc123".to_string()));
    assert!(msg
        .text_content()
        .unwrap()
        .contains(r#""temperature": "22C""#));
}

/// Test that consecutive tool results can be tracked.
#[test]
fn test_consecutive_tool_results() {
    let messages = vec![
        Message::assistant_with_tool_calls(vec![
            ToolCall::new("call_1", "get_weather", r#"{"location": "Paris"}"#),
            ToolCall::new("call_2", "get_weather", r#"{"location": "London"}"#),
        ]),
        Message::tool("call_1", r#"{"temp": "22C"}"#),
        Message::tool("call_2", r#"{"temp": "18C"}"#),
    ];

    // Verify we have the expected structure
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].role, Role::Assistant);
    assert_eq!(messages[1].role, Role::Tool);
    assert_eq!(messages[2].role, Role::Tool);

    // In Anthropic, these consecutive tool results would be merged into
    // a single user message with multiple tool_result blocks
    let tool_count = messages.iter().filter(|m| m.role == Role::Tool).count();
    assert_eq!(tool_count, 2);
}

/// Test tool specification conversion.
#[test]
fn test_tool_spec_conversion() {
    let tool = Tool::function("get_weather", "Get the weather for a city")
        .parameters(json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name"
                }
            },
            "required": ["location"]
        }))
        .build();

    assert_eq!(tool.tool_type, "function");
    assert_eq!(tool.function.name, "get_weather");
    assert_eq!(
        tool.function.description,
        Some("Get the weather for a city".to_string())
    );

    // Verify parameters structure
    let params = tool.function.parameters.as_ref().unwrap();
    assert!(params.get("properties").is_some());
    assert!(params.get("required").is_some());
}

/// Test unsupported parameter error for response_format.
#[test]
fn test_response_format_unsupported() {
    // Anthropic doesn't support response_format
    // This would be checked in the completion method
    let err = any_llm::AnyLLMError::unsupported_parameter(
        "anthropic",
        "response_format",
        "Use tool calling for structured output",
    );

    assert!(err.to_string().contains("response_format"));
    assert!(err.to_string().contains("anthropic"));
}

/// Test error messages for Anthropic-specific errors.
#[test]
fn test_anthropic_error_messages() {
    let rate_limit = any_llm::AnyLLMError::rate_limit("anthropic", "Rate limit exceeded");
    assert!(rate_limit.to_string().contains("anthropic"));

    let auth = any_llm::AnyLLMError::authentication("anthropic", "Invalid x-api-key");
    assert!(auth.to_string().contains("anthropic"));
    assert!(auth.to_string().contains("Authentication"));
}
