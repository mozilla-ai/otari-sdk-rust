//! Unit tests for core completion functionality.

use any_llm::{
    parse_model_string, AnyLLMError, CompletionOptions, CompletionParams, LLMProvider, Message,
    ProviderConfig, Role, ToolChoice,
};

#[test]
fn test_parse_model_string_valid_colon() {
    let (provider, model) = parse_model_string("openai:gpt-4o-mini").unwrap();
    assert_eq!(provider, LLMProvider::OpenAI);
    assert_eq!(model, "gpt-4o-mini");
}

#[test]
fn test_parse_model_string_valid_slash() {
    let (provider, model) = parse_model_string("anthropic/claude-3-5-sonnet-latest").unwrap();
    assert_eq!(provider, LLMProvider::Anthropic);
    assert_eq!(model, "claude-3-5-sonnet-latest");
}

#[test]
fn test_parse_model_string_with_colons_in_model() {
    // Fine-tuned models can have colons in their ID
    let (provider, model) = parse_model_string("openai:ft:gpt-4o:org:custom:id").unwrap();
    assert_eq!(provider, LLMProvider::OpenAI);
    assert_eq!(model, "ft:gpt-4o:org:custom:id");
}

#[test]
fn test_parse_model_string_invalid_no_separator() {
    let result = parse_model_string("gpt-4o-mini");
    assert!(result.is_err());
    if let Err(AnyLLMError::InvalidRequest { message, .. }) = result {
        assert!(message.contains("Invalid model format"));
    } else {
        panic!("Expected InvalidRequest error");
    }
}

#[test]
fn test_parse_model_string_invalid_empty_provider() {
    let result = parse_model_string(":gpt-4o-mini");
    assert!(result.is_err());
    if let Err(AnyLLMError::InvalidRequest { message, .. }) = result {
        assert!(message.contains("Empty provider"));
    } else {
        panic!("Expected InvalidRequest error for empty provider");
    }
}

#[test]
fn test_parse_model_string_invalid_empty_model() {
    let result = parse_model_string("openai:");
    assert!(result.is_err());
    if let Err(AnyLLMError::InvalidRequest { message, .. }) = result {
        assert!(message.contains("Empty model ID"));
    } else {
        panic!("Expected InvalidRequest error for empty model");
    }
}

#[test]
fn test_parse_model_string_unsupported_provider() {
    let result = parse_model_string("unsupported:model");
    assert!(result.is_err());
    if let Err(AnyLLMError::UnsupportedProvider { provider }) = result {
        assert_eq!(provider, "unsupported");
    } else {
        panic!("Expected UnsupportedProvider error");
    }
}

#[test]
fn test_llm_provider_from_str() {
    assert_eq!(
        LLMProvider::from_str("openai").unwrap(),
        LLMProvider::OpenAI
    );
    assert_eq!(
        LLMProvider::from_str("OPENAI").unwrap(),
        LLMProvider::OpenAI
    );
    assert_eq!(
        LLMProvider::from_str("OpenAI").unwrap(),
        LLMProvider::OpenAI
    );
    assert_eq!(
        LLMProvider::from_str("anthropic").unwrap(),
        LLMProvider::Anthropic
    );
}

#[test]
fn test_llm_provider_as_str() {
    assert_eq!(LLMProvider::OpenAI.as_str(), "openai");
    assert_eq!(LLMProvider::Anthropic.as_str(), "anthropic");
}

#[test]
fn test_llm_provider_env_var() {
    assert_eq!(LLMProvider::OpenAI.env_var(), "OPENAI_API_KEY");
    assert_eq!(LLMProvider::Anthropic.env_var(), "ANTHROPIC_API_KEY");
}

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
fn test_provider_config_from_options() {
    let options = CompletionOptions::with_api_key("my-key").api_base("https://custom.api.com");

    let config: ProviderConfig = options.into();

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

#[test]
fn test_all_providers_can_be_loaded() {
    // Test that we can create providers with mock config
    // Note: This will fail if API key is not provided, which is expected
    let providers = [LLMProvider::OpenAI, LLMProvider::Anthropic];

    for provider in providers {
        let config = ProviderConfig::new("test-api-key");
        let result = any_llm::create_provider(provider, config);
        assert!(result.is_ok(), "Failed to create provider: {:?}", provider);
    }
}
