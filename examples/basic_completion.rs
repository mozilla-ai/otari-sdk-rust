//! Basic chat completion example.
//!
//! Run with: `cargo run --example basic_completion`
//!
//! Make sure to set your API key:
//! ```bash
//! export OPENAI_API_KEY="your-key-here"
//! ```

use any_llm::{completion, providers::OpenAI, CompletionOptions, Message};

#[tokio::main]
async fn main() -> any_llm::Result<()> {
    // Create messages
    let messages = vec![
        Message::system("You are a helpful assistant. Be concise."),
        Message::user("What is Rust programming language in one sentence?"),
    ];

    // Make completion request
    println!("Sending request to OpenAI...\n");

    let response = completion::<OpenAI>(
        "gpt-4o-mini",
        messages,
        CompletionOptions::default().max_tokens(100),
    )
    .await?;

    // Print response
    println!("Response: {}", response.content().unwrap_or("No content"));

    // Print usage statistics
    if let Some(usage) = &response.usage {
        println!(
            "\nTokens - Prompt: {}, Completion: {}, Total: {}",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    Ok(())
}
