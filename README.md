<p align="center">
  <img src="assets/otari-logo.svg" width="320" alt="otari logo"/>
</p>

<div align="center">

# Otari Rust Client SDK

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org)
<a href="https://discord.gg/4gf3zXrQUc">
    <img src="https://img.shields.io/static/v1?label=Chat%20on&message=Discord&color=blue&logo=Discord&style=flat-square" alt="Discord">
</a>

**Rust client for [otari](https://github.com/mozilla-ai/otari), the open-source core that powers [otari.ai](https://otari.ai).**
Communicate with any LLM provider through otari using a single, typed interface.

[Python SDK](https://github.com/mozilla-ai/otari-sdk-python) | [TypeScript SDK](https://github.com/mozilla-ai/otari-sdk-ts) | [Go SDK](https://github.com/mozilla-ai/otari-sdk-go) | [Documentation](https://mozilla-ai.github.io/otari/) | [Platform (Beta)](https://otari.ai/)

</div>

> New to otari? The [otari repo](https://github.com/mozilla-ai/otari) explains what it is and why you’d use it.

## Quickstart

The fastest way to start is the **hosted gateway**. Grab a platform token from
[otari.ai](https://otari.ai/), set it in your environment, and the SDK targets
`https://api.otari.ai` automatically, no API key or base URL needed in code.

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
  [otari](https://github.com/mozilla-ai/otari) instance for self-hosting

### Install

This crate is not yet published to crates.io (publishing is tracked
separately). Until then, depend on it directly from git:

```toml
[dependencies]
otari = { git = "https://github.com/mozilla-ai/otari-sdk-rust" }
```

## Authentication

There are two ways to authenticate, depending on where the gateway runs.

**Platform mode (recommended)**: uses `Authorization: Bearer` platform-mode
auth against the hosted gateway. Set the platform token in your environment and
leave `api_key` / `api_base` unset; the base URL defaults to
`https://api.otari.ai`:

```bash
export OTARI_AI_TOKEN="your-platform-token"
```

**Self-hosted gateway**: point the SDK at your own gateway with an API key
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

## Usage

The high-level free functions (`completion`, `completion_stream`, `rerank`)
build a client per call from `CompletionOptions` / `RerankOptions`. For the
remaining endpoints, build an `Otari` client once with `Otari::from_config` and
call its methods. In every example below, credentials come from the environment
(`Config::default()` / `CompletionOptions::default()`); swap in
`CompletionOptions::with_api_key(...).api_base(...)` or an explicit `Config` for
a self-hosted gateway.

### Chat completions

```rust
use otari::{completion, Message, CompletionOptions};

let messages = vec![
    Message::system("You are a helpful assistant."),
    Message::user("What is the capital of France?"),
];

let response = completion(
    "openai:gpt-4o-mini",
    messages,
    CompletionOptions::default(),
).await?;

println!("{}", response.content().unwrap_or_default());
```

### Streaming

```rust
use otari::{completion_stream, Message, CompletionOptions, ChunkAccumulator};
use futures::StreamExt;

let messages = vec![Message::user("Tell me a story")];

let mut stream = completion_stream(
    "openai:gpt-4o-mini",
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

### Responses API

The OpenAI-style Responses API is available on the `Otari` client via
`client.response(...)`. The gateway's responses payload has no single typed
model, so this returns the raw `serde_json::Value`. Use
`client.response_stream(...)` for the streamed form.

```rust
use otari::{Config, Otari};
use serde_json::json;

let client = Otari::from_config(Config::default())?;

let resp = client
    .response(json!({
        "model": "openai:gpt-4o-mini",
        "input": "Write a haiku about the sea.",
    }))
    .await?;

println!("{}", resp["id"]);
```

### Messages API

The Anthropic-style `/messages` endpoint is available via
`client.message(...)`. The request must include `max_tokens`, and the response
is returned as a raw `serde_json::Value`. Use `client.message_stream(...)` for
streaming.

```rust
use otari::{Config, Otari};
use serde_json::json;

let client = Otari::from_config(Config::default())?;

let resp = client
    .message(json!({
        "model": "anthropic:claude-3-5-sonnet",
        "messages": [{"role": "user", "content": "Hello!"}],
        "max_tokens": 256,
    }))
    .await?;

println!("{}", resp["id"]);
```

### Embeddings

Create embeddings via `client.embedding(...)`, which returns the generated
typed `CreateEmbeddingResponse`.

```rust
use otari::{Config, Otari};
use serde_json::json;

let client = Otari::from_config(Config::default())?;

let resp = client
    .embedding(json!({
        "model": "openai:text-embedding-3-small",
        "input": "The quick brown fox",
    }))
    .await?;

println!("vector length: {}", resp.data[0].embedding.len());
```

### Listing models

List the models the gateway can route to with `client.list_models(...)`. Pass
`Some(provider)` to scope the list to one provider, or `None` for all.

```rust
use otari::{Config, Otari};

let client = Otari::from_config(Config::default())?;

let models = client.list_models(None).await?;
for model in models {
    println!("{}", model.id);
}
```

### Moderation

The `Otari` client exposes a `moderation` method that calls
`POST /v1/moderations` and returns an OpenAI-compatible response:

```rust
use otari::{Config, ModerationInput, ModerationParams, Otari};

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
```

Only upstream providers with moderation support will succeed; others
return `OtariError::Unsupported { provider, operation: "moderation" }`
(or `"multimodal_moderation"` when the request used image parts).

### Reranking

Rerank documents by relevance to a query with the `rerank` free function.
Results come back sorted by `relevance_score` descending.

```rust
use otari::{rerank, RerankOptions};

let documents = vec![
    "The capital of France is Paris.".to_string(),
    "Bananas are a good source of potassium.".to_string(),
];

let response = rerank(
    "cohere:rerank-v3.5",
    "What is the capital of France?",
    documents,
    RerankOptions::default(),
).await?;

for result in &response.results {
    println!("doc {} scored {}", result.index, result.relevance_score);
}
```

### Image generation

Generate images with `client.image_generation(...)`, which calls
`POST /v1/images/generations` and returns the gateway's OpenAI-compatible
payload as a raw `serde_json::Value` (the generated core models this response as
an opaque object):

```rust
use otari::{Config, ImageGenerationParams, Otari};

let client = Otari::from_config(Config::default())?;

let result = client
    .image_generation(
        ImageGenerationParams::new("openai:dall-e-3", "a red bicycle")
            .with_size("1024x1024"),
    )
    .await?;

println!("{}", result["data"][0]["url"]);
```

### Audio (speech and transcription)

`client.speech(...)` synthesizes text to speech (`POST /v1/audio/speech`) and
returns the raw audio as `bytes::Bytes`, since the gateway responds with binary
audio rather than JSON:

```rust
use otari::{Config, Otari, SpeechParams};

let client = Otari::from_config(Config::default())?;

let audio = client
    .speech(SpeechParams::new("openai:tts-1", "Hello there!", "alloy"))
    .await?;
std::fs::write("speech.mp3", &audio)?;
```

`client.transcription(...)` transcribes audio (`POST /v1/audio/transcriptions`).
The audio bytes are uploaded as multipart form data, and the result is returned
as a `serde_json::Value`: the parsed JSON for JSON formats, or a JSON string for
the `text` / `srt` / `vtt` formats.

```rust
use otari::{Config, Otari, TranscriptionParams};

let client = Otari::from_config(Config::default())?;
let audio = std::fs::read("speech.mp3")?;

let result = client
    .transcription(
        TranscriptionParams::new("openai:whisper-1", audio).with_filename("speech.mp3"),
    )
    .await?;

println!("{}", result["text"]);
```

### Batch operations

```rust
use otari::{BatchRequestItem, BatchStatus, Config, CreateBatchParams, Otari};
use serde_json::json;

let client = Otari::from_config(Config::default())?;

let params = CreateBatchParams::new(
    "openai:gpt-4o-mini",
    vec![
        BatchRequestItem {
            custom_id: "req-1".to_string(),
            body: json!({
                "messages": [{"role": "user", "content": "Hello"}],
                "max_tokens": 50,
            }),
        },
    ],
)
.completion_window("24h");

let batch = client.create_batch(params).await?;
println!("Batch ID: {}", batch.id);

// Poll status, fetch results, or cancel (provider scopes the lookup):
let provider = batch.provider.as_deref().unwrap_or("openai");
let batch = client.retrieve_batch(&batch.id, provider).await?;
if batch.status == BatchStatus::Completed {
    let results = client.retrieve_batch_results(&batch.id, provider).await?;
    println!("results: {}", results.results.len());
}
```

### Error handling

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

### Tool calling

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
let options = CompletionOptions::default()
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

### Extended thinking

For models that support extended thinking (reasoning):

```rust
use otari::{completion, Message, CompletionOptions, ReasoningEffort};

let messages = vec![Message::user("Solve this step by step: What is 15% of 240?")];

let options = CompletionOptions::default()
    .reasoning_effort(ReasoningEffort::Medium)
    .max_tokens(16000);

let response = completion(
    "anthropic:claude-3-5-sonnet",
    messages,
    options,
).await?;

// Access reasoning content
if let Some(reasoning) = &response.choices[0].message.reasoning {
    println!("Thinking: {}", reasoning.content);
}
println!("Answer: {}", response.content().unwrap_or_default());
```

### Switching models

Change the model string to route to different upstream providers through the gateway:

```rust
// OpenAI via gateway
let response = completion(
    "openai:gpt-4o", messages.clone(), options.clone()
).await?;

// Anthropic via gateway
let response = completion(
    "anthropic:claude-3-5-sonnet", messages, options
).await?;
```

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
