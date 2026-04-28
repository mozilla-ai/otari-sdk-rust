//! # any-llm
//!
//! A unified Rust SDK for interacting with LLMs via the any-llm gateway.
//!
//! This library provides a single, consistent interface to interact with
//! the [any-llm gateway](https://github.com/mozilla-ai/any-llm), a FastAPI-based
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
//! use any_llm::{completion, Message, CompletionOptions, providers::Gateway};
//!
//! #[tokio::main]
//! async fn main() -> any_llm::Result<()> {
//!     let messages = vec![Message::user("Hello, how are you?")];
//!
//!     let response = completion::<Gateway>(
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
//! use any_llm::{completion_stream, Message, CompletionOptions, ChunkAccumulator, providers::Gateway};
//! use futures::StreamExt;
//!
//! # async fn example() -> any_llm::Result<()> {
//! let messages = vec![Message::user("Tell me a story")];
//!
//! let mut stream = completion_stream::<Gateway>(
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
//! use any_llm::{completion, Message, CompletionOptions, Tool, ToolChoice, providers::Gateway};
//! use serde_json::json;
//!
//! # async fn example() -> any_llm::Result<()> {
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
//! let response = completion::<Gateway>("openai:gpt-4o-mini", messages, options).await?;
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
//! - `ANY_LLM_API_KEY`: API key for the any-llm gateway
//! - `ANY_LLM_API_BASE`: Base URL of the any-llm gateway

pub mod api;
pub mod error;
pub mod provider;
pub mod providers;
pub mod types;

// Re-export main types for convenience
pub use api::{completion, completion_stream, rerank, CompletionOptions, RerankOptions};
pub use error::{AnyLLMError, Result};
pub use provider::{Provider, ProviderConfig};
pub use types::{
    Batch, BatchRequestCounts, BatchRequestItem, BatchResult, BatchResultError, BatchResultItem,
    BatchStatus, ChatCompletion, ChatCompletionChunk, ChatCompletionMessage, Choice, ChoiceDelta,
    ChunkAccumulator, ChunkChoice, CompletionParams, CompletionUsage, Content, ContentPart,
    CreateBatchParams, Function, ImageUrl, ListBatchesOptions, Message, ModerationContentPart,
    ModerationImageUrl, ModerationInput, ModerationParams, ModerationResponse, ModerationResult,
    Reasoning, ReasoningEffort, RerankMeta, RerankParams, RerankResponse, RerankResult,
    RerankUsage, Role, StopSequence, Tool, ToolCall, ToolCallDelta, ToolChoice, ToolFunction,
};
