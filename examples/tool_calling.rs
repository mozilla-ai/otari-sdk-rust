//! Tool calling example.
//!
//! Run with: `cargo run --example tool_calling`
//!
//! Make sure to set your API key:
//! ```bash
//! export OPENAI_API_KEY="your-key-here"
//! ```

use any_llm::{completion, providers::OpenAI, CompletionOptions, Message, Tool, ToolChoice};
use serde_json::json;

#[tokio::main]
async fn main() -> any_llm::Result<()> {
    // Define a weather tool
    let weather_tool = Tool::function("get_weather", "Get the current weather for a location")
        .parameters(json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "The temperature unit"
                }
            },
            "required": ["location"]
        }))
        .build();

    let messages = vec![Message::user("What's the weather like in Paris, France?")];

    let options = CompletionOptions::default()
        .tools(vec![weather_tool])
        .tool_choice(ToolChoice::auto());

    println!("Asking about weather with tool calling...\n");

    let response = completion::<OpenAI>("gpt-4o-mini", messages, options).await?;

    // Check if the model wants to call a tool
    if let Some(tool_calls) = &response.choices[0].message.tool_calls {
        println!("Model requested {} tool call(s):\n", tool_calls.len());

        for call in tool_calls {
            println!("  Tool: {}", call.function.name);
            println!("  Arguments: {}", call.function.arguments);
            println!("  Call ID: {}", call.id);

            // In a real application, you would:
            // 1. Parse the arguments
            // 2. Call your actual function
            // 3. Send the result back to the model
            println!("\n  (In production, you'd execute this function and return results)");
        }
    } else {
        // Model responded directly without calling a tool
        println!(
            "Direct response: {}",
            response.content().unwrap_or("No content")
        );
    }

    Ok(())
}
