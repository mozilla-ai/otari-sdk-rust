//! Streaming completion example.
//!
//! Run with: `cargo run --example streaming`
//!
//! Make sure to set your API key:
//! ```bash
//! export OPENAI_API_KEY="your-key-here"
//! ```

use any_llm::{completion_stream, providers::OpenAI, ChunkAccumulator, CompletionOptions, Message};
use futures::StreamExt;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> any_llm::Result<()> {
    let messages = vec![Message::user(
        "Write a short poem about Rust programming language (4 lines max).",
    )];

    println!("Streaming response from OpenAI...\n");

    let mut stream = completion_stream::<OpenAI>(
        "gpt-4o-mini",
        messages,
        CompletionOptions::default().max_tokens(200),
    )
    .await?;

    let mut accumulator = ChunkAccumulator::new();

    // Stream and print each chunk as it arrives
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;

        // Print content as it streams
        if let Some(content) = chunk.content() {
            print!("{}", content);
            io::stdout().flush().unwrap();
        }

        // Accumulate for final stats
        accumulator.add(&chunk);
    }

    println!("\n\n--- Stream Complete ---");
    println!(
        "Total content length: {} characters",
        accumulator.content.len()
    );

    if let Some(usage) = &accumulator.usage {
        println!(
            "Tokens - Prompt: {}, Completion: {}, Total: {}",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    Ok(())
}
