//! Provider trait and factory for LLM providers.

use std::pin::Pin;

use futures::Stream;

use crate::{
    error::Result,
    types::{ChatCompletion, ChatCompletionChunk, CompletionParams, RerankParams, RerankResponse},
};

mod config;
mod error;
mod interface;

pub use config::ProviderConfig;
pub use interface::Provider;

/// A stream of completion chunks.
pub type CompletionStream =
    Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send + 'static>>;

#[derive(Debug)]
pub struct AnyLLMProvider<P: Provider>(P);

impl<P: Provider> AnyLLMProvider<P> {
    pub fn from_config(config: ProviderConfig) -> Result<Self> {
        P::from_config(config).map(Self)
    }

    pub async fn completion(&self, params: CompletionParams) -> Result<ChatCompletion> {
        self.0.completion(params).await
    }

    pub async fn completion_stream(&self, params: CompletionParams) -> Result<CompletionStream> {
        self.0.completion_stream(params).await
    }

    /// Rerank documents by relevance to a query.
    pub async fn rerank(&self, params: RerankParams) -> Result<RerankResponse> {
        self.0.rerank(params).await
    }
}
