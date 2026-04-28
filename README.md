<p align="center">
  <picture>
    <img src="https://raw.githubusercontent.com/mozilla-ai/any-llm/refs/heads/main/docs/public/images/any-llm-logo-mark.png" width="20%" alt="Project logo"/>
  </picture>
</p>

<div align="center">

# any-llm (Rust)

[![Crates.io](https://img.shields.io/crates/v/any-llm.svg)](https://crates.io/crates/any-llm)
[![Documentation](https://docs.rs/any-llm/badge.svg)](https://docs.rs/any-llm)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org)
<a href="https://discord.gg/4gf3zXrQUc">
    <img src="https://img.shields.io/static/v1?label=Chat%20on&message=Discord&color=blue&logo=Discord&style=flat-square" alt="Discord">
</a>

**Communicate with any LLM provider through the any-llm gateway.**

[Python SDK](https://github.com/mozilla-ai/any-llm) | [Documentation](https://mozilla-ai.github.io/any-llm/) | [Platform (Beta)](https://any-llm.ai/)

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
use any_llm::{completion, Message, CompletionOptions, providers::Gateway};

#[tokio::main]
async fn main() -> any_llm::Result<()> {
    let messages = vec![Message::user("Hello!")];

    let response = completion::<Gateway>(
        "openai:gpt-4o-mini",
        messages,
        CompletionOptions::with_api_key("your-api-key")
            .api_base("http://localhost:8000"),
    ).await?;

    println!("{}", response.content().unwrap_or_default());
    Ok(())
}
```

## Installation

### Requirements

- Rust 1.83 or newer
- A running [any-llm gateway](https://github.com/mozilla-ai/any-llm) instance

### Setting Up API Keys

Set environment variables:

```bash
export ANY_LLM_API_KEY="your-key-here"
export ANY_LLM_API_BASE="http://localhost:8000"
```

Alternatively, pass the API key and base URL directly in your code:

```rust
let options = CompletionOptions::with_api_key("your-api-key")
    .api_base("http://localhost:8000");
```

## any-llm Gateway

The [any-llm gateway](https://github.com/mozilla-ai/any-llm) is a FastAPI-based proxy server that exposes an OpenAI-compatible API and routes requests to multiple upstream LLM providers. It adds enterprise-grade features:

- **Budget Management** - Enforce spending limits with automatic daily, weekly, or monthly resets
- **API Key Management** - Issue, revoke, and monitor virtual API keys without exposing provider credentials
- **Usage Analytics** - Track every request with full token counts, costs, and metadata
- **Multi-tenant Support** - Manage access and budgets across users and teams

### Quick Start

```bash
docker run \
  -e GATEWAY_MASTER_KEY="your-secure-master-key" \
  -e OPENAI_API_KEY="your-api-key" \
  -p 8000:8000 \
  ghcr.io/mozilla-ai/any-llm/gateway:latest
```

> **Note:** You can use a specific release version instead of `latest` (e.g., `1.2.0`). See [available versions](https://github.com/orgs/mozilla-ai/packages/container/package/any-llm%2Fgateway).

### Managed Platform (Beta)

Prefer a hosted experience? The [any-llm platform](https://any-llm.ai/) provides a managed control plane for keys, usage tracking, and cost visibility across providers, while still building on the same `any-llm` interfaces.

## Usage

### Basic Completion

```rust
use any_llm::{completion, Message, CompletionOptions, providers::Gateway};

let messages = vec![
    Message::system("You are a helpful assistant."),
    Message::user("What is the capital of France?"),
];

let response = completion::<Gateway>(
    "openai:gpt-4o-mini",
    messages,
    CompletionOptions::with_api_key("your-api-key")
        .api_base("http://localhost:8000"),
).await?;

println!("{}", response.content().unwrap_or_default());
```

### Switching Models

Change the model string to route to different upstream providers through the gateway:

```rust
// OpenAI via gateway
let response = completion::<Gateway>(
    "openai:gpt-4o", messages.clone(), options.clone()
).await?;

// Anthropic via gateway
let response = completion::<Gateway>(
    "anthropic:claude-3-5-sonnet-latest", messages, options
).await?;
```

### Streaming

```rust
use any_llm::{completion_stream, Message, CompletionOptions, ChunkAccumulator, providers::Gateway};
use futures::StreamExt;

let messages = vec![Message::user("Tell me a story")];

let mut stream = completion_stream::<Gateway>(
    "openai:gpt-4o-mini",
    messages,
    CompletionOptions::with_api_key("your-api-key")
        .api_base("http://localhost:8000"),
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
use any_llm::{completion, Message, CompletionOptions, Tool, ToolChoice, providers::Gateway};
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
let options = CompletionOptions::with_api_key("your-api-key")
    .api_base("http://localhost:8000")
    .tools(vec![weather_tool])
    .tool_choice(ToolChoice::auto());

let response = completion::<Gateway>("openai:gpt-4o-mini", messages, options).await?;

if let Some(tool_calls) = &response.choices[0].message.tool_calls {
    for call in tool_calls {
        println!("Function: {}", call.function.name);
        println!("Arguments: {}", call.function.arguments);
    }
}
```

### Extended Thinking (Reasoning)

For models that support extended thinking:

```rust
use any_llm::{completion, Message, CompletionOptions, ReasoningEffort, providers::Gateway};

let messages = vec![Message::user("Solve this step by step: What is 15% of 240?")];

let options = CompletionOptions::with_api_key("your-api-key")
    .api_base("http://localhost:8000")
    .reasoning_effort(ReasoningEffort::Medium)
    .max_tokens(16000);

let response = completion::<Gateway>(
    "anthropic:claude-sonnet-4-20250514",
    messages,
    options,
).await?;

// Access reasoning content
if let Some(reasoning) = &response.choices[0].message.reasoning {
    println!("Thinking: {}", reasoning.content);
}
println!("Answer: {}", response.content().unwrap_or_default());
```

### Moderation

The gateway provider exposes an inherent `moderation` method that calls
`POST /v1/moderations` and returns an OpenAI-compatible response:

```rust,no_run
use any_llm::providers::Gateway;
use any_llm::{AnyLLMError, ModerationInput, ModerationParams, Provider, ProviderConfig};

# async fn example() -> any_llm::Result<()> {
let gw = Gateway::from_config(ProviderConfig::default())?;

let resp = gw
    .moderation(
        ModerationParams::new(
            "openai:omni-moderation-latest",
            ModerationInput::Text("hurt someone".into()),
        )
        .with_user("user_123"),
    )
    .await?;

if resp.results[0].flagged {
    println!("unsafe input");
}
# Ok(())
# }
```

Only upstream providers with moderation support will succeed; others
return `AnyLLMError::Unsupported { provider, operation: "moderation" }`
(or `"multimodal_moderation"` when the request used image parts).

### Batch Operations

```rust
use any_llm::providers::Gateway;
use any_llm::{CreateBatchParams, Message, Provider, ProviderConfig};

# async fn example() -> any_llm::Result<()> {
let gw = Gateway::from_config(ProviderConfig::default())?;

let params = CreateBatchParams::new(
    "openai:gpt-4o-mini",
    vec![
        ("req-1", vec![Message::user("Hello")]),
        ("req-2", vec![Message::user("World")]),
    ],
);

let batch = gw.create_batch(params).await?;
println!("Batch ID: {}", batch.id);
# Ok(())
# }
```

### Error Handling

```rust
use any_llm::{completion, AnyLLMError};

match completion::<Gateway>(model, messages, options).await {
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

## Gateway Capabilities

The gateway supports all features through upstream providers:

| Feature     | Supported |
|-------------|:---------:|
| Completion  |     ✅     |
| Streaming   |     ✅     |
| Tools       |     ✅     |
| Images      |     ✅     |
| Reasoning   |     ✅     |
| PDF         |     ✅     |
| Reranking   |     ✅     |
| Batch       |     ✅     |
| Moderation  |     ✅     |

## Why choose `any-llm`?

- **Simple, unified interface** - Single function for all models, switch providers by changing the model string
- **Developer friendly** - Full Rust type safety with serde serialization and clear, actionable error messages
- **Gateway-powered** - Route to any upstream provider through a single gateway endpoint
- **Async-first** - Built on Tokio for high-performance async I/O
- **Streaming support** - Real-time token streaming with async streams
- **Battle-tested** - Based on the proven [any-llm](https://github.com/mozilla-ai/any-llm) Python library

## Development

```bash
# Build
cargo build --all-features

# Run all checks
cargo fmt --check && cargo clippy --all-features -- -D warnings

# Run tests
cargo test --all-features

# Run the gateway example
cargo run --example gateway_completion

# Build docs
cargo doc --all-features --no-deps --open
```

## Documentation

- **[Full Documentation](https://mozilla-ai.github.io/any-llm/)** - Complete guides and API reference
- **[Gateway Documentation](https://mozilla-ai.github.io/any-llm/gateway/overview/)** - Gateway setup and deployment
- **[Python SDK](https://github.com/mozilla-ai/any-llm)** - The full Python SDK with direct provider access
- **[any-llm Platform (Beta)](https://any-llm.ai/)** - Hosted control plane for key management, usage tracking, and cost visibility

## Contributing

We welcome contributions from developers of all skill levels! Please see our [Contributing Guide](CONTRIBUTING.md) or open an issue to discuss changes.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
