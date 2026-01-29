//! Provider trait and factory for LLM providers.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::error::Result;
use crate::types::{ChatCompletion, ChatCompletionChunk, CompletionParams};

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
}

/// Configuration for creating a provider.
#[derive(Debug, Clone, Default)]
pub struct ProviderConfig {
    /// The API key (if not set, will try environment variable).
    pub api_key: Option<String>,

    /// The API base URL (for custom endpoints/proxies).
    pub api_base: Option<String>,

    /// Additional configuration (provider-specific).
    pub extra: std::collections::HashMap<String, String>,
}

impl ProviderConfig {
    /// Create a new provider config with an API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            ..Default::default()
        }
    }

    /// Set the API base URL.
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = Some(api_base.into());
        self
    }

    /// Add an extra configuration value.
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Trait for LLM providers.
///
/// Implement this trait to add support for a new LLM provider.
#[async_trait]
pub trait Provider: Sized + Send + Sync {
    const NAME: &'static str;
    const ENV_VAR: &'static str;
    const DOCS_URL: &'static str;

    const SUPPORTS_STREAMING: bool = true;
    const SUPPORTS_TOOLS: bool = true;
    const SUPPORTS_IMAGES: bool = false;
    const SUPPORTS_REASONING: bool = false;
    const SUPPORTS_PDF: bool = false;

    fn api_key(config: &ProviderConfig) -> Option<String> {
        config.api_key
            .clone()
            .or_else(|| std::env::var(Self::ENV_VAR).ok())
    }

    fn from_config(config: ProviderConfig) -> Result<Self>;

    /// Create a chat completion.
    async fn completion(&self, params: CompletionParams) -> Result<ChatCompletion>;

    /// Create a streaming chat completion.
    async fn completion_stream(&self, params: CompletionParams) -> Result<CompletionStream>;
}
