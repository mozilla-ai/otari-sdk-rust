//! Example: Chat completion via the any-llm gateway.
//!
//! # Setup
//!
//! Set environment variables:
//! ```sh
//! export GATEWAY_API_BASE=http://localhost:8000
//! export GATEWAY_PLATFORM_TOKEN=tk_your_token_here
//! ```
//!
//! # Run
//!
//! ```sh
//! cargo run --example gateway_completion
//! ```

use any_llm::{
    completion, completion_stream, providers::Gateway, ChunkAccumulator, CompletionOptions, Message,
};
use futures::StreamExt;

#[tokio::main]
async fn main() -> any_llm::Result<()> {
    let messages = vec![Message::user("Say hello in three languages.")];
    let options = CompletionOptions::default();

    // --- Non-streaming ---
    println!("--- Non-streaming ---");
    let response =
        completion::<Gateway>("openai:gpt-4o-mini", messages.clone(), options.clone()).await?;
    println!("{}", response.content().unwrap_or("(no content)"));

    if let Some(usage) = &response.usage {
        println!(
            "Tokens: {} prompt + {} completion = {} total",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    // --- Streaming ---
    println!("\n--- Streaming ---");
    let mut stream = completion_stream::<Gateway>("openai:gpt-4o-mini", messages, options).await?;

    let mut acc = ChunkAccumulator::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(content) = chunk.content() {
            print!("{content}");
        }
        acc.add(&chunk);
    }
    println!();

    if let Some(usage) = &acc.usage {
        println!(
            "Tokens: {} prompt + {} completion = {} total",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    Ok(())
}
