<p align="center">
  <img src="assets/otari-logo.svg" width="320" alt="otari logo"/>
</p>

<div align="center">

# otari (Rust)

[![Crates.io](https://img.shields.io/crates/v/otari.svg)](https://crates.io/crates/otari)
[![Documentation](https://docs.rs/otari/badge.svg)](https://docs.rs/otari)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org)
<a href="https://discord.gg/4gf3zXrQUc">
    <img src="https://img.shields.io/static/v1?label=Chat%20on&message=Discord&color=blue&logo=Discord&style=flat-square" alt="Discord">
</a>

**Communicate with any LLM provider through the Otari gateway.**

[Python SDK](https://github.com/mozilla-ai/otari-sdk-python) | [TypeScript SDK](https://github.com/mozilla-ai/otari-sdk-ts) | [Go SDK](https://github.com/mozilla-ai/otari-sdk-go) | [Documentation](https://mozilla-ai.github.io/otari/) | [Platform (Beta)](https://otari.ai/)

</div>

## Quickstart

Add to your `Cargo.toml`:

```toml
[dependencies]
otari = "0.1"  # From crates.io (once published)
tokio = { version = "1", features = ["full"] }
```

Or install from GitHub directly:

```toml
[dependencies]
otari = { git = "https://github.com/mozilla-ai/otari-sdk-rust" }
tokio = { version = "1", features = ["full"] }
```

The fastest way to start is the **hosted gateway**. Grab a platform token from
[otari.ai](https://otari.ai/), set it in your environment, and the SDK targets
`https://api.otari.ai` automatically — no API key or base URL needed in code.

```bash
export OTARI_AI_TOKEN="your-platform-token"
```

```rust
use otari::{completion, Message, CompletionOptions};

#[tokio::main]
async fn main() -> otari::Result<()> {
    let messages = vec![Message::user("Hello!")];

    // No api_key / api_base: platform mode uses OTARI_AI_TOKEN against the
    // hosted gateway (https://api.otari.ai).
    let response = completion(
        "openai:gpt-4o-mini",
        messages,
        CompletionOptions::default(),
    ).await?;

    println!("{}", response.content().unwrap_or_default());
    Ok(())
}
```

## Installation

### Requirements

- Rust 1.83 or newer
- Either a platform token for the hosted gateway, or a running
  [Otari gateway](https://github.com/mozilla-ai/otari) instance for self-hosting

### Authentication

There are two ways to authenticate, depending on where the gateway runs.

**Hosted platform (recommended)** — uses `Authorization: Bearer` platform-mode
auth against the hosted gateway. Set the platform token in your environment and
leave `api_key` / `api_base` unset:

```bash
export OTARI_AI_TOKEN="your-platform-token"
```

**Self-hosting the gateway** — point the SDK at your own gateway with an API key
(sent as the `Otari-Key` header) and an explicit base URL:

```rust
let options = CompletionOptions::with_api_key("your-gateway-key")
    .api_base("http://localhost:8000");
```

Or via environment variables (the SDK reads canonical names first, then the
legacy aliases):

```bash
export GATEWAY_API_KEY="your-gateway-key"   # legacy alias: OTARI_API_KEY
export GATEWAY_API_BASE="http://localhost:8000"  # legacy alias: OTARI_API_BASE
```

| Variable           | Purpose                                  | Legacy alias            |
|--------------------|------------------------------------------|-------------------------|
| `OTARI_AI_TOKEN`   | Platform token for the hosted gateway    | `OTARI_PLATFORM_TOKEN`  |
| `GATEWAY_API_KEY`  | API key for a self-hosted gateway        | `OTARI_API_KEY`         |
| `GATEWAY_API_BASE` | Gateway base URL                         | `OTARI_API_BASE`        |

## Otari Gateway

The [Otari gateway](https://github.com/mozilla-ai/otari) is a FastAPI-based proxy server that exposes an OpenAI-compatible API and routes requests to multiple upstream LLM providers. It adds enterprise-grade features:

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
  ghcr.io/mozilla-ai/otari/gateway:latest
```

> **Note:** You can use a specific release version instead of `latest` (e.g., `1.2.0`). See [available versions](https://github.com/orgs/mozilla-ai/packages/container/package/otari%2Fgateway).

The gateway needs at least one provider key configured (e.g. the `OPENAI_API_KEY` above, or via [otari.ai/organization-settings/provider-keys](https://otari.ai/organization-settings/provider-keys) on the hosted platform) so it can route requests upstream.

### Managed Platform (Beta)

Prefer a hosted experience? The [Otari platform](https://otari.ai/) provides a managed control plane for keys, usage tracking, and cost visibility across providers, while still building on the same interfaces.

## Usage

### Basic Completion

```rust
use otari::{completion, Message, CompletionOptions};

let messages = vec![
    Message::system("You are a helpful assistant."),
    Message::user("What is the capital of France?"),
];

let response = completion(
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
let response = completion(
    "openai:gpt-4o", messages.clone(), options.clone()
).await?;

// Anthropic via gateway
let response = completion(
    "anthropic:claude-3-5-sonnet-latest", messages, options
).await?;
```

### Streaming

```rust
use otari::{completion_stream, Message, CompletionOptions, ChunkAccumulator};
use futures::StreamExt;

let messages = vec![Message::user("Tell me a story")];

let mut stream = completion_stream(
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
use otari::{completion, Message, CompletionOptions, Tool, ToolChoice};
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

let response = completion("openai:gpt-4o-mini", messages, options).await?;

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
use otari::{completion, Message, CompletionOptions, ReasoningEffort};

let messages = vec![Message::user("Solve this step by step: What is 15% of 240?")];

let options = CompletionOptions::with_api_key("your-api-key")
    .api_base("http://localhost:8000")
    .reasoning_effort(ReasoningEffort::Medium)
    .max_tokens(16000);

let response = completion(
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

The `Otari` client exposes a `moderation` method that calls
`POST /v1/moderations` and returns an OpenAI-compatible response:

```rust,no_run
use otari::{Config, ModerationInput, ModerationParams, Otari, OtariError};

# async fn example() -> otari::Result<()> {
let client = Otari::from_config(Config::default())?;

let resp = client
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
return `OtariError::Unsupported { provider, operation: "moderation" }`
(or `"multimodal_moderation"` when the request used image parts).

### Batch Operations

```rust
use otari::{Config, CreateBatchParams, Message, Otari};

# async fn example() -> otari::Result<()> {
let client = Otari::from_config(Config::default())?;

let params = CreateBatchParams::new(
    "openai:gpt-4o-mini",
    vec![
        ("req-1", vec![Message::user("Hello")]),
        ("req-2", vec![Message::user("World")]),
    ],
);

let batch = client.create_batch(params).await?;
println!("Batch ID: {}", batch.id);
# Ok(())
# }
```

### Error Handling

```rust
use otari::{completion, OtariError};

match completion(model, messages, options).await {
    Ok(response) => println!("{}", response.content().unwrap_or_default()),
    Err(OtariError::RateLimit { provider, message }) => {
        eprintln!("Rate limited by {}: {}", provider, message);
    }
    Err(OtariError::Authentication { provider, message }) => {
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

## Why choose `otari`?

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

- **[Full Documentation](https://mozilla-ai.github.io/otari/)** - Complete guides and API reference
- **[Gateway Documentation](https://mozilla-ai.github.io/otari/gateway/overview/)** - Gateway setup and deployment
- **[Python SDK](https://github.com/mozilla-ai/otari-sdk-python)** - The Python SDK
- **[TypeScript SDK](https://github.com/mozilla-ai/otari-sdk-ts)** - The TypeScript SDK for Node.js applications
- **[Go SDK](https://github.com/mozilla-ai/otari-sdk-go)** - The Go SDK
- **[Otari Platform (Beta)](https://otari.ai/)** - Hosted control plane for key management, usage tracking, and cost visibility

## Contributing

We welcome contributions from developers of all skill levels! Please see our [Contributing Guide](CONTRIBUTING.md) or open an issue to discuss changes.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
