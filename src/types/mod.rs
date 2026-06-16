//! Type definitions for otari.
//!
//! This module contains all the core types used throughout the library,
//! designed to be compatible with the OpenAI API format while supporting
//! extensions for other providers.

use std::pin::Pin;

use futures::Stream;

use crate::error::Result;

mod batch;
mod chunk;
mod completion;
mod media;
mod message;
mod moderation;
mod rerank;
mod tool;
mod usage;

pub use batch::*;
pub use chunk::*;
pub use completion::*;
pub use media::*;
pub use message::*;
pub use moderation::*;
pub use rerank::*;
pub use tool::*;
pub use usage::*;

/// A stream of completion chunks.
pub type CompletionStream =
    Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send + 'static>>;

/// A stream of raw SSE event payloads parsed as JSON values, used by the
/// responses / messages streaming endpoints (which have no single typed chunk
/// model). Re-exported from the SSE shim.
pub use crate::client::models::stream::RawValueStream;
