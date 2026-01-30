//! Integration tests for streaming completions.
//!
//! These tests require API keys to run. They are ignored by default.
//! Run with: `cargo test --test integration_streaming -- --ignored`
//!
//! Set environment variables:
//! - OPENAI_API_KEY
//! - ANTHROPIC_API_KEY

use any_llm::{
    completion_stream,
    providers::{Anthropic, OpenAI},
    ChunkAccumulator, CompletionOptions, Message,
};
use futures::StreamExt;

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_openai_streaming() {
    let messages = vec![Message::user("Count from 1 to 5, one number per line.")];

    let result = completion_stream::<OpenAI>(
        "gpt-4o-mini",
        messages,
        CompletionOptions::default().max_tokens(50),
    )
    .await;

    match result {
        Ok(mut stream) => {
            let mut accumulator = ChunkAccumulator::new();
            let mut chunk_count = 0;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        chunk_count += 1;
                        accumulator.add(&chunk);
                    }
                    Err(e) => panic!("Stream error: {}", e),
                }
            }

            assert!(chunk_count >= 1, "Expected at least 1 chunk");
            assert!(accumulator.has_content(), "Expected accumulated content");

            let content = &accumulator.content;
            assert!(
                content.contains('1') && content.contains('5'),
                "Expected numbers 1-5 in: {}",
                content
            );
        }
        Err(e) => panic!("Failed to create stream: {}", e),
    }
}

#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_anthropic_streaming() {
    let messages = vec![Message::user("Say 'hello world' and nothing else.")];

    let result = completion_stream::<Anthropic>(
        "claude-3-5-haiku-latest",
        messages,
        CompletionOptions::default().max_tokens(20),
    )
    .await;

    match result {
        Ok(mut stream) => {
            let mut accumulator = ChunkAccumulator::new();
            let mut chunk_count = 0;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        chunk_count += 1;
                        accumulator.add(&chunk);
                    }
                    Err(e) => panic!("Stream error: {}", e),
                }
            }

            assert!(chunk_count >= 1, "Expected at least 1 chunk");
            assert!(accumulator.has_content(), "Expected accumulated content");

            let content = accumulator.content.to_lowercase();
            assert!(
                content.contains("hello"),
                "Expected 'hello' in: {}",
                content
            );
        }
        Err(e) => panic!("Failed to create stream: {}", e),
    }
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_streaming_chunk_structure() {
    let messages = vec![Message::user("Hi")];

    let result = completion_stream::<OpenAI>(
        "gpt-4o-mini",
        messages,
        CompletionOptions::default().max_tokens(10),
    )
    .await;

    match result {
        Ok(mut stream) => {
            let mut found_content = false;
            let mut found_finish = false;

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.expect("Stream should not error");

                // Verify chunk structure
                assert_eq!(chunk.object, "chat.completion.chunk");
                assert!(!chunk.choices.is_empty());

                let choice = &chunk.choices[0];
                assert_eq!(choice.index, 0);

                // Track what we've seen
                if choice.delta.content.is_some() {
                    found_content = true;
                }
                if choice.finish_reason.is_some() {
                    found_finish = true;
                }
            }

            assert!(found_content, "Should have received content chunks");
            assert!(found_finish, "Should have received finish_reason");
        }
        Err(e) => panic!("Failed to create stream: {}", e),
    }
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_chunk_accumulator() {
    let messages = vec![Message::user("Say 'test' exactly.")];

    let result = completion_stream::<OpenAI>(
        "gpt-4o-mini",
        messages,
        CompletionOptions::default().max_tokens(10).temperature(0.0),
    )
    .await;

    match result {
        Ok(mut stream) => {
            let mut accumulator = ChunkAccumulator::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.expect("Stream should not error");
                accumulator.add(&chunk);
            }

            // Verify accumulator state
            assert!(accumulator.id.is_some());
            assert!(accumulator.model.is_some());
            assert!(accumulator.has_content());
            assert!(accumulator.finish_reason.is_some());
        }
        Err(e) => panic!("Failed to create stream: {}", e),
    }
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_streaming_early_termination() {
    let messages = vec![Message::user("Write a long story about a dragon.")];

    let result = completion_stream::<OpenAI>(
        "gpt-4o-mini",
        messages,
        CompletionOptions::default().max_tokens(500), // Request many tokens
    )
    .await;

    match result {
        Ok(mut stream) => {
            let mut chunk_count = 0;
            let mut content = String::new();

            // Only consume first 5 chunks (early termination)
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.expect("Stream should not error");
                if let Some(c) = chunk.content() {
                    content.push_str(c);
                }
                chunk_count += 1;
                if chunk_count >= 5 {
                    break; // Early termination
                }
            }

            // Should have received some content
            assert!(!content.is_empty(), "Should have received some content");
            assert!(chunk_count <= 5, "Should have stopped after 5 chunks");

            // Stream is dropped here - should clean up properly
        }
        Err(e) => panic!("Failed to create stream: {}", e),
    }
}
