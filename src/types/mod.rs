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
mod message;
mod moderation;
mod rerank;
mod tool;
mod usage;

pub use batch::*;
pub use chunk::*;
pub use completion::*;
pub use message::*;
pub use moderation::*;
pub use rerank::*;
pub use tool::*;
pub use usage::*;

/// A stream of completion chunks.
pub type CompletionStream =
    Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send + 'static>>;
