//! # otari
//!
//! A unified Rust SDK for interacting with LLMs via the Otari gateway.
//!
//! This library provides a single, consistent interface to interact with
//! the [Otari gateway](https://github.com/mozilla-ai/otari-sdk-rust), a FastAPI-based
//! proxy that exposes an OpenAI-compatible API and routes requests to multiple
//! upstream LLM providers.
//!
//! ## Features
//!
//! - **Unified API**: Single interface for all models through the gateway
//! - **Streaming support**: Real-time token streaming with async streams
//! - **Tool calling**: Function/tool calling with automatic format conversion
//! - **Image support**: Send images to vision-capable models
//! - **Extended thinking**: Support for reasoning features
//! - **Reranking**: Document reranking support
//! - **Batch operations**: Create, retrieve, cancel, and list batch jobs
//! - **Moderation**: Content moderation via the gateway
//! - **Type-safe**: Strong Rust types with serde serialization
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use otari::{completion, Message, CompletionOptions};
//!
//! #[tokio::main]
//! async fn main() -> otari::Result<()> {
//!     let messages = vec![Message::user("Hello, how are you?")];
//!
//!     let response = completion(
//!         "openai:gpt-4o-mini",
//!         messages,
//!         CompletionOptions::with_api_key("your-api-key")
//!             .api_base("http://localhost:8000"),
//!     ).await?;
//!
//!     println!("{}", response.content().unwrap_or_default());
//!     Ok(())
//! }
//! ```
//!
//! ## Streaming
//!
//! ```rust,no_run
//! use otari::{completion_stream, Message, CompletionOptions, ChunkAccumulator};
//! use futures::StreamExt;
//!
//! # async fn example() -> otari::Result<()> {
//! let messages = vec![Message::user("Tell me a story")];
//!
//! let mut stream = completion_stream(
//!     "openai:gpt-4o-mini",
//!     messages,
//!     CompletionOptions::with_api_key("your-api-key")
//!         .api_base("http://localhost:8000"),
//! ).await?;
//!
//! let mut accumulator = ChunkAccumulator::new();
//! while let Some(chunk) = stream.next().await {
//!     let chunk = chunk?;
//!     if let Some(content) = chunk.content() {
//!         print!("{}", content);
//!     }
//!     accumulator.add(&chunk);
//! }
//!
//! // Access accumulated data
//! println!("\nFull response: {}", accumulator.content);
//! # Ok(())
//! # }
//! ```
//!
//! ## Tool Calling
//!
//! ```rust,no_run
//! use otari::{completion, Message, CompletionOptions, Tool, ToolChoice};
//! use serde_json::json;
//!
//! # async fn example() -> otari::Result<()> {
//! let weather_tool = Tool::function("get_weather", "Get the current weather")
//!     .parameters(json!({
//!         "type": "object",
//!         "properties": {
//!             "location": {
//!                 "type": "string",
//!                 "description": "City name"
//!             }
//!         },
//!         "required": ["location"]
//!     }))
//!     .build();
//!
//! let messages = vec![Message::user("What's the weather in Paris?")];
//! let options = CompletionOptions::with_api_key("your-api-key")
//!     .api_base("http://localhost:8000")
//!     .tools(vec![weather_tool])
//!     .tool_choice(ToolChoice::auto());
//!
//! let response = completion("openai:gpt-4o-mini", messages, options).await?;
//!
//! if let Some(tool_calls) = &response.choices[0].message.tool_calls {
//!     for call in tool_calls {
//!         println!("Function: {}, Args: {}", call.function.name, call.function.arguments);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Environment Variables
//!
//! Canonical names are read first, with the legacy alias as a fallback:
//!
//! - `OTARI_AI_TOKEN`: platform token for the hosted gateway (enables
//!   `Authorization: Bearer` platform mode). Legacy alias: `OTARI_PLATFORM_TOKEN`.
//! - `GATEWAY_API_KEY`: API key for a self-hosted gateway (`Otari-Key` header).
//!   Legacy alias: `OTARI_API_KEY`.
//! - `GATEWAY_API_BASE`: Base URL of the gateway. Legacy alias: `OTARI_API_BASE`.
//!   In platform mode this defaults to `https://api.otari.ai` when unset.

// OpenAPI-generated typed core (every endpoint + control plane), produced by
// the otari codegen pipeline and inlined as a private module so the crate
// publishes to crates.io as a single unit (no path dependency). The generated
// code refers to its own modules as `crate::models` / `crate::apis`, so those
// are re-exported at the crate root below.
mod _client;
pub(crate) use _client::{apis, models};

pub mod api;
pub mod client;
pub mod config;
pub mod control_plane;
mod core;
pub mod error;
pub mod types;

// Re-export main types for convenience
pub use api::{completion, completion_stream, rerank, CompletionOptions, RerankOptions};
pub use client::Otari;
pub use config::Config;
pub use error::{OtariError, Result};
pub use types::{
    Batch, BatchRequestCounts, BatchRequestItem, BatchResult, BatchResultError, BatchResultItem,
    BatchStatus, ChatCompletion, ChatCompletionChunk, ChatCompletionMessage, Choice, ChoiceDelta,
    ChunkAccumulator, ChunkChoice, CompletionParams, CompletionStream, CompletionUsage, Content,
    ContentPart, CreateBatchParams, Function, ImageUrl, ListBatchesOptions, Message,
    ModerationContentPart, ModerationImageUrl, ModerationInput, ModerationParams,
    ModerationResponse, ModerationResult, Reasoning, ReasoningEffort, RerankMeta, RerankParams,
    RerankResponse, RerankResult, RerankUsage, Role, StopSequence, Tool, ToolCall, ToolCallDelta,
    ToolChoice, ToolFunction,
};
