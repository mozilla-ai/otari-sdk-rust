//! Multi-provider example - demonstrates switching between providers.
//!
//! Run with: `cargo run --example multi_provider`
//!
//! Make sure to set your API keys:
//! ```bash
//! export OPENAI_API_KEY="your-key-here"
//! export ANTHROPIC_API_KEY="your-key-here"
//! ```

use any_llm::{completion, CompletionOptions, Message};

#[tokio::main]
async fn main() -> any_llm::Result<()> {
    let question = "What is 2 + 2? Reply with just the number.";

    // Define providers to test
    let providers = vec![
        ("openai:gpt-4o-mini", "OpenAI GPT-4o-mini"),
        (
            "anthropic:claude-3-5-haiku-latest",
            "Anthropic Claude Haiku",
        ),
    ];

    println!("Asking multiple providers: \"{}\"\n", question);

    for (model, name) in providers {
        let messages = vec![Message::user(question)];

        print!("{}: ", name);

        match completion(model, messages, CompletionOptions::default().max_tokens(50)).await {
            Ok(response) => {
                println!("{}", response.content().unwrap_or("No response"));
            }
            Err(e) => {
                println!("Error - {} (is API key set?)", e);
            }
        }
    }

    println!("\nDone! Same code, different providers.");

    Ok(())
}
