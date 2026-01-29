//! Provider trait and factory for LLM providers.

use std::pin::Pin;

use futures::{Future, Stream};

use crate::{
    error::{AnyLLMError, Result},
    types::Content,
    types::{ChatCompletion, ChatCompletionChunk, CompletionParams},
};

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
        config
            .api_key
            .clone()
            .or_else(|| std::env::var(Self::ENV_VAR).ok())
    }

    fn from_config(config: ProviderConfig) -> Result<Self>;

    fn validate_completion_params(params: &CompletionParams) -> Result<()> {
        // check tools
        if !Self::SUPPORTS_TOOLS && (params.tools.is_some() || params.tool_choice.is_some()) {
            return Err(AnyLLMError::provider_error(
                Self::NAME,
                "Provider does not support tools",
            ));
        }

        // check images
        if !Self::SUPPORTS_IMAGES {
            for message in &params.messages {
                if let Some(Content::Parts(parts)) = &message.content {
                    for part in parts {
                        match part {
                            crate::types::ContentPart::ImageUrl { .. } => {
                                return Err(AnyLLMError::provider_error(
                                    Self::NAME,
                                    "Provider does not support images",
                                ));
                            }
                            crate::types::ContentPart::Text { .. } => {}
                        }
                    }
                }
            }
        }

        // validate reasoning effort
        if !Self::SUPPORTS_REASONING && params.reasoning_effort.is_some() {
            return Err(AnyLLMError::provider_error(
                Self::NAME.to_string(),
                "Provider does not support reasoning",
            ));
        }

        Ok(())
    }

    fn completion_fn(
        &self,
        params: CompletionParams,
    ) -> impl Future<Output = Result<ChatCompletion>> + Send;

    /// Create a chat completion.
    fn completion(
        &self,
        params: CompletionParams,
    ) -> impl Future<Output = Result<ChatCompletion>> + Send {
        async move {
            Self::validate_completion_params(&params)?;

            self.completion_fn(params).await
        }
    }

    fn completion_stream_fn(
        &self,
        params: CompletionParams,
    ) -> impl Future<Output = Result<CompletionStream>> + Send;

    /// Create a streaming chat completion.
    fn completion_stream(
        &self,
        params: CompletionParams,
    ) -> impl Future<Output = Result<CompletionStream>> + Send {
        async move {
            if !Self::SUPPORTS_STREAMING {
                return Err(AnyLLMError::provider_error(
                    Self::NAME,
                    "Provider does not support streaming",
                ));
            }

            Self::validate_completion_params(&params)?;

            self.completion_stream_fn(params).await
        }
    }
}
