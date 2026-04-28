//! Unit tests for core completion functionality.

use otari::{CompletionOptions, CompletionParams, Config, Message, Role, ToolChoice};

#[test]
fn test_message_creation() {
    let system = Message::system("You are helpful.");
    assert_eq!(system.role, Role::System);
    assert_eq!(system.text_content(), Some("You are helpful.".to_string()));

    let user = Message::user("Hello!");
    assert_eq!(user.role, Role::User);
    assert_eq!(user.text_content(), Some("Hello!".to_string()));

    let assistant = Message::assistant("Hi there!");
    assert_eq!(assistant.role, Role::Assistant);
    assert_eq!(assistant.text_content(), Some("Hi there!".to_string()));
}

#[test]
fn test_message_tool_response() {
    let tool_msg = Message::tool("call_123", "Result from tool");
    assert_eq!(tool_msg.role, Role::Tool);
    assert_eq!(tool_msg.tool_call_id, Some("call_123".to_string()));
    assert_eq!(
        tool_msg.text_content(),
        Some("Result from tool".to_string())
    );
}

#[test]
fn test_completion_params_builder() {
    let params = CompletionParams::new("gpt-4o-mini", vec![Message::user("Hello")])
        .with_temperature(0.7)
        .with_max_tokens(100)
        .with_stream(true);

    assert_eq!(params.model_id, "gpt-4o-mini");
    assert_eq!(params.temperature, Some(0.7));
    assert_eq!(params.max_tokens, Some(100));
    assert_eq!(params.stream, Some(true));
}

#[test]
fn test_completion_options_builder() {
    let options = CompletionOptions::with_api_key("test-key")
        .temperature(0.5)
        .max_tokens(200);

    assert_eq!(options.api_key, Some("test-key".to_string()));
    assert_eq!(options.temperature, Some(0.5));
    assert_eq!(options.max_tokens, Some(200));
}

#[test]
fn test_config_from_options() {
    let options = CompletionOptions::with_api_key("my-key").api_base("https://custom.api.com");

    let config: Config = options.into();

    assert_eq!(config.api_key, Some("my-key".to_string()));
    assert_eq!(config.api_base, Some("https://custom.api.com".to_string()));
}

#[test]
fn test_tool_choice_modes() {
    let none = ToolChoice::none();
    assert_eq!(none.as_mode(), Some("none"));

    let auto = ToolChoice::auto();
    assert_eq!(auto.as_mode(), Some("auto"));

    let required = ToolChoice::required();
    assert_eq!(required.as_mode(), Some("required"));
}

#[test]
fn test_tool_choice_specific_function() {
    let choice = ToolChoice::function("my_function");
    assert_eq!(choice.as_function(), Some("my_function"));
    assert!(choice.as_mode().is_none());
}
