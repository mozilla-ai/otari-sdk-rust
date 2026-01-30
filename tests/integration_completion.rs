//! Integration tests for chat completions.
//!
//! These tests require API keys to run. They are ignored by default.
//! Run with: `cargo test --test integration_completion -- --ignored`
//!
//! Set environment variables:
//! - OPENAI_API_KEY
//! - ANTHROPIC_API_KEY

use any_llm::{
    completion,
    providers::{Anthropic, OpenAI},
    CompletionOptions, Message,
};

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_openai_completion() {
    let messages = vec![
        Message::system("You are a helpful assistant. Be brief."),
        Message::user("What is 2 + 2? Reply with just the number."),
    ];

    let result = completion::<OpenAI>("gpt-4o-mini", messages, CompletionOptions::default()).await;

    match result {
        Ok(response) => {
            assert!(!response.choices.is_empty());
            let content = response.content().expect("Expected content");
            assert!(
                content.contains('4'),
                "Expected '4' in response: {}",
                content
            );
        }
        Err(e) => panic!("Completion failed: {}", e),
    }
}

#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_anthropic_completion() {
    let messages = vec![Message::user(
        "What is the capital of France? Reply with just the city name.",
    )];

    let result = completion::<Anthropic>(
        "claude-3-5-haiku-latest",
        messages,
        CompletionOptions::default().max_tokens(100),
    )
    .await;

    match result {
        Ok(response) => {
            assert!(!response.choices.is_empty());
            let content = response.content().expect("Expected content");
            assert!(
                content.to_lowercase().contains("paris"),
                "Expected 'Paris' in response: {}",
                content
            );
        }
        Err(e) => panic!("Completion failed: {}", e),
    }
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_openai_completion_with_temperature() {
    let messages = vec![Message::user("Say 'hello' and nothing else.")];

    let result = completion::<OpenAI>(
        "gpt-4o-mini",
        messages,
        CompletionOptions::default().temperature(0.0).max_tokens(10),
    )
    .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    let content = response.content().unwrap().to_lowercase();
    assert!(content.contains("hello"));
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_completion_returns_usage() {
    let messages = vec![Message::user("Hi")];
    let result = completion::<OpenAI>("gpt-4o-mini", messages, CompletionOptions::default()).await;

    match result {
        Ok(response) => {
            assert!(response.usage.is_some(), "Expected usage statistics");
            let usage = response.usage.unwrap();
            assert!(usage.prompt_tokens > 0);
            assert!(usage.completion_tokens > 0);
            assert_eq!(
                usage.total_tokens,
                usage.prompt_tokens + usage.completion_tokens
            );
        }
        Err(e) => panic!("Completion failed: {}", e),
    }
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_completion_invalid_model() {
    let messages = vec![Message::user("Hello")];
    let result = completion::<OpenAI>(
        "nonexistent-model-xyz",
        messages,
        CompletionOptions::default(),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_concurrent_completions() {
    let messages1 = vec![Message::user(
        "What is the capital of France? Reply with just the city name.",
    )];
    let messages2 = vec![Message::user(
        "What is the capital of Germany? Reply with just the city name.",
    )];

    let (result1, result2) = tokio::join!(
        completion::<OpenAI>(
            "gpt-4o-mini",
            messages1,
            CompletionOptions::default().max_tokens(20)
        ),
        completion::<OpenAI>(
            "gpt-4o-mini",
            messages2,
            CompletionOptions::default().max_tokens(20)
        )
    );

    match (result1, result2) {
        (Ok(r1), Ok(r2)) => {
            let content1 = r1.content().unwrap().to_lowercase();
            let content2 = r2.content().unwrap().to_lowercase();
            assert!(content1.contains("paris"));
            assert!(content2.contains("berlin"));
        }
        (Err(e), _) | (_, Err(e)) => panic!("Concurrent completion failed: {}", e),
    }
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_chat_completion_response_structure() {
    let messages = vec![Message::user("Hello")];
    let result = completion::<OpenAI>("gpt-4o-mini", messages, CompletionOptions::default()).await;

    match result {
        Ok(response) => {
            // Verify response structure
            assert!(!response.id.is_empty());
            assert_eq!(response.object, "chat.completion");
            assert!(response.created > 0);
            assert!(response.model.contains("gpt"));
            assert!(!response.choices.is_empty());

            let choice = &response.choices[0];
            assert_eq!(choice.index, 0);
            assert!(choice.finish_reason.is_some());
            assert!(choice.message.content.is_some());
        }
        Err(e) => panic!("Completion failed: {}", e),
    }
}
