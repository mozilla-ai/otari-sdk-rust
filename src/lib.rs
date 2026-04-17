//! # any-llm
//!
//! A unified Rust SDK for multiple LLM providers.
//!
//! This library provides a single, consistent interface to interact with
//! multiple LLM providers (OpenAI, Anthropic, and more). Switch providers
//! without changing your code.
//!
//! ## Features
//!
//! - **Unified API**: Single interface for all providers
//! - **Streaming support**: Real-time token streaming with async streams
//! - **Tool calling**: Function/tool calling with automatic format conversion
//! - **Image support**: Send images to vision-capable models
//! - **Extended thinking**: Support for Anthropic's thinking/reasoning feature
//! - **Type-safe**: Strong Rust types with serde serialization
//!
//! ## Cargo Features
//!
//! - `openai` (default): Enable OpenAI provider support
//! - `anthropic` (default): Enable Anthropic provider support
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use any_llm::{completion, Message, CompletionOptions, providers::OpenAI};
//!
//! #[tokio::main]
//! async fn main() -> any_llm::Result<()> {
//!     let messages = vec![Message::user("Hello, how are you?")];
//!
//!     let response = completion::<OpenAI>(
//!         "gpt-4o-mini",
//!         messages,
//!         CompletionOptions::default(),
//!     ).await?;
//!
//!     println!("{}", response.content().unwrap_or_default());
//!     Ok(())
//! }
//! ```
//!
//! ## Switching Providers
//!
//! Simply change the model string to switch providers:
//!
//! ```rust,no_run
//! # use any_llm::{completion, Message, CompletionOptions, providers::{OpenAI, Anthropic}};
//! # async fn example() -> any_llm::Result<()> {
//! # let messages = vec![Message::user("Hello")];
//! // OpenAI
//! let response = completion::<OpenAI>(":gpt-4o", messages.clone(), CompletionOptions::default()).await?;
//!
//! // Anthropic
//! let response = completion::<Anthropic>("claude-3-5-sonnet-latest", messages, CompletionOptions::default()).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Streaming
//!
//! ```rust,no_run
//! use any_llm::{completion_stream, Message, CompletionOptions, ChunkAccumulator, providers::OpenAI};
//! use futures::StreamExt;
//!
//! # async fn example() -> any_llm::Result<()> {
//! let messages = vec![Message::user("Tell me a story")];
//!
//! let mut stream = completion_stream::<OpenAI>(
//!     "gpt-4o-mini",
//!     messages,
//!     CompletionOptions::default(),
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
//! use any_llm::{completion, Message, CompletionOptions, Tool, ToolChoice, providers::OpenAI};
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
//! let options = CompletionOptions::default()
//!     .tools(vec![weather_tool])
//!     .tool_choice(ToolChoice::auto());
//!
//! let response = completion::<OpenAI>("gpt-4o-mini", messages, options).await?;
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
//! Set your API keys via environment variables:
//!
//! - `OPENAI_API_KEY`: OpenAI API key
//! - `ANTHROPIC_API_KEY`: Anthropic API key

pub mod api;
pub mod error;
pub mod provider;
pub mod providers;
pub mod types;

// Re-export main types for convenience
pub use api::{completion, completion_stream, CompletionOptions};
pub use error::{AnyLLMError, Result};
pub use provider::{Provider, ProviderConfig};
pub use types::{
    Batch, BatchRequestCounts, BatchRequestItem, BatchResult, BatchResultError, BatchResultItem,
    BatchStatus, ChatCompletion, ChatCompletionChunk, ChatCompletionMessage, Choice, ChoiceDelta,
    ChunkAccumulator, ChunkChoice, CompletionParams, CompletionUsage, Content, ContentPart,
    CreateBatchParams, Function, ImageUrl, ListBatchesOptions, Message, Reasoning, ReasoningEffort,
    Role, StopSequence, Tool, ToolCall, ToolCallDelta, ToolChoice, ToolFunction,
};
