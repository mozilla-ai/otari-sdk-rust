<p align="center">
  <picture>
    <img src="https://raw.githubusercontent.com/mozilla-ai/any-llm/refs/heads/main/docs/images/any-llm-logo-mark.png" width="20%" alt="any-llm logo"/>
  </picture>
</p>

<div align="center">

# any-llm

[![Crates.io](https://img.shields.io/crates/v/any-llm.svg)](https://crates.io/crates/any-llm)
[![Documentation](https://docs.rs/any-llm/badge.svg)](https://docs.rs/any-llm)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org)

**Communicate with any LLM provider using a single, unified interface.**

Switch between OpenAI, Anthropic, and more without changing your code.

[Documentation](https://docs.rs/any-llm) | [Examples](./examples) | [Contributing](CONTRIBUTING.md)

</div>

## Quickstart

Add to your `Cargo.toml`:

```toml
[dependencies]
any-llm = "0.1"  # From crates.io (once published)
tokio = { version = "1", features = ["full"] }
```

Or install from GitHub directly:

```toml
[dependencies]
any-llm = { git = "https://github.com/mozilla-ai/any-llm-rust" }
tokio = { version = "1", features = ["full"] }
```

```rust
use any_llm::{completion, Message, CompletionOptions, providers::OpenAI};

#[tokio::main]
async fn main() -> any_llm::Result<()> {
    // Set OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable

    let messages = vec![Message::user("Hello!")];

    let response = completion::<OpenAI>(
        "gpt-4o-mini",
        messages,
        CompletionOptions::default(),
    ).await?;

    println!("{}", response.content().unwrap_or_default());
    Ok(())
}
```

**That's it!** Change the model string to switch between providers.

## Installation

### Requirements

- Rust 1.83 or newer
- API keys for your chosen LLM providers

### Feature Flags

```toml
[dependencies]
# Both providers enabled by default
any-llm = "0.1"

# Or select specific providers
any-llm = { version = "0.1", default-features = false, features = ["openai"] }
any-llm = { version = "0.1", default-features = false, features = ["anthropic"] }

# From GitHub with specific features
any-llm = { git = "https://github.com/mozilla-ai/any-llm-rust", features = ["openai"] }
```

### Setting Up API Keys

Set environment variables for your chosen providers:

```bash
export OPENAI_API_KEY="your-key-here"
export ANTHROPIC_API_KEY="your-key-here"
```

Or pass API keys directly in code:

```rust
let options = CompletionOptions::with_api_key("your-api-key");
```

## Why choose `any-llm`?

- **Simple, unified interface** - Single function for all providers, switch models with just a string change
- **Type-safe** - Full Rust type safety with serde serialization
- **Leverages official SDKs** - Uses `async-openai` for OpenAI, ensuring maximum compatibility
- **Async-first** - Built on Tokio for high-performance async I/O
- **Streaming support** - Real-time token streaming with async streams
- **Tool calling** - Function/tool calling with automatic format conversion
- **Extended thinking** - Support for Anthropic's thinking/reasoning feature

## Usage

### Basic Completion

```rust
use any_llm::{completion, Message, CompletionOptions, providers::OpenAI};

let messages = vec![
    Message::system("You are a helpful assistant."),
    Message::user("What is the capital of France?"),
];

let response = completion::<OpenAI>(
    "gpt-4o-mini",
    messages,
    CompletionOptions::default(),
).await?;

println!("{}", response.content().unwrap_or_default());
```

### Switching Providers

Simply change the function generic type:

```rust
// OpenAI
let response = completion::<OpenAI>("gpt-4o", messages.clone(), options.clone()).await?;

// Anthropic
let response = completion::<Anthropic>("claude-3-5-sonnet-latest", messages, options).await?;
```

### Streaming

```rust
use any_llm::{completion_stream, Message, CompletionOptions, ChunkAccumulator, providers::OpenAI};
use futures::StreamExt;

let messages = vec![Message::user("Tell me a story")];

let mut stream = completion_stream::<OpenAI>(
    "gpt-4o-mini",
    messages,
    CompletionOptions::default(),
).await?;

let mut accumulator = ChunkAccumulator::new();
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(content) = chunk.content() {
        print!("{}", content);
    }
    accumulator.add(&chunk);
}

println!("\nTotal tokens: {:?}", accumulator.usage);
```

### Tool Calling

```rust
use any_llm::{completion, Message, CompletionOptions, Tool, ToolChoice, providers::OpenAI};
use serde_json::json;

let weather_tool = Tool::function("get_weather", "Get the current weather")
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

let messages = vec![Message::user("What's the weather in Paris?")];
let options = CompletionOptions::default()
    .tools(vec![weather_tool])
    .tool_choice(ToolChoice::auto());

let response = completion::<OpenAI>("gpt-4o-mini", messages, options).await?;

if let Some(tool_calls) = &response.choices[0].message.tool_calls {
    for call in tool_calls {
        println!("Function: {}", call.function.name);
        println!("Arguments: {}", call.function.arguments);
    }
}
```

### Extended Thinking (Anthropic)

```rust
use any_llm::{completion, Message, CompletionOptions, ReasoningEffort, providers::Anthropic};

let messages = vec![Message::user("Solve this step by step: What is 15% of 240?")];

let options = CompletionOptions::default()
    .reasoning_effort(ReasoningEffort::Medium)
    .max_tokens(16000);

let response = completion::<Anthropic>(
    "claude-sonnet-4-20250514",
    messages,
    options,
).await?;

// Access reasoning content
if let Some(reasoning) = &response.choices[0].message.reasoning {
    println!("Thinking: {}", reasoning.content);
}
println!("Answer: {}", response.content().unwrap_or_default());
```

### Using the Provider Directly

For more control or connection reuse:

```rust
use any_llm::{create_provider, LLMProvider, ProviderConfig, CompletionParams, Message};

let config = ProviderConfig::new("your-api-key");
let provider = create_provider(LLMProvider::OpenAI, config)?;

let params = CompletionParams {
    model_id: "gpt-4o-mini".to_string(),
    messages: vec![Message::user("Hello!")],
    ..Default::default()
};

let response = provider.completion(params).await?;
```

## Supported Providers

| Provider | Completion | Streaming | Tools | Images | Reasoning |
|----------|------------|-----------|-------|--------|-----------|
| OpenAI | ✅ | ✅ | ✅ | ✅ | ❌ |
| Anthropic | ✅ | ✅ | ✅ | ✅ | ✅ |

## Error Handling

```rust
use any_llm::{completion, AnyLLMError};

match completion(model, messages, options).await {
    Ok(response) => println!("{}", response.content().unwrap_or_default()),
    Err(AnyLLMError::RateLimit { provider, message }) => {
        eprintln!("Rate limited by {}: {}", provider, message);
    }
    Err(AnyLLMError::Authentication { provider, message }) => {
        eprintln!("Auth failed for {}: {}", provider, message);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Examples

See the [examples](./examples) directory for complete working examples:

- [`basic_completion.rs`](./examples/basic_completion.rs) - Simple chat completion
- [`streaming.rs`](./examples/streaming.rs) - Streaming responses
- [`tool_calling.rs`](./examples/tool_calling.rs) - Function/tool calling
- [`multi_provider.rs`](./examples/multi_provider.rs) - Switching between providers

Run an example:

```bash
OPENAI_API_KEY=your-key cargo run --example basic_completion
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## Related Projects

- [any-llm (Python)](https://github.com/mozilla-ai/any-llm) - The original Python implementation
- [async-openai](https://github.com/64bit/async-openai) - Rust OpenAI SDK we build on

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
