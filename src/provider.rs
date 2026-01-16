//! Provider trait and factory for LLM providers.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::error::{AnyLLMError, Result};
use crate::types::{ChatCompletion, ChatCompletionChunk, CompletionParams};

/// A stream of completion chunks.
pub type CompletionStream =
    Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send + 'static>>;

/// Supported LLM providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LLMProvider {
    /// OpenAI (GPT models).
    OpenAI,
    /// Anthropic (Claude models).
    Anthropic,
}

impl LLMProvider {
    /// Parse a provider from a string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Self::OpenAI),
            "anthropic" => Ok(Self::Anthropic),
            _ => Err(AnyLLMError::UnsupportedProvider {
                provider: s.to_string(),
            }),
        }
    }

    /// Get the provider name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
        }
    }

    /// Get the environment variable name for the API key.
    pub fn env_var(&self) -> &'static str {
        match self {
            Self::OpenAI => "OPENAI_API_KEY",
            Self::Anthropic => "ANTHROPIC_API_KEY",
        }
    }

    /// Get the documentation URL for the provider.
    pub fn documentation_url(&self) -> &'static str {
        match self {
            Self::OpenAI => "https://platform.openai.com/docs/api-reference",
            Self::Anthropic => "https://docs.anthropic.com/en/home",
        }
    }
}

impl std::fmt::Display for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
pub trait Provider: Send + Sync {
    /// Get the provider name.
    fn name(&self) -> &'static str;

    /// Check if the provider supports streaming.
    fn supports_streaming(&self) -> bool {
        true
    }

    /// Check if the provider supports tool/function calling.
    fn supports_tools(&self) -> bool {
        true
    }

    /// Check if the provider supports image inputs.
    fn supports_images(&self) -> bool {
        false
    }

    /// Check if the provider supports extended reasoning/thinking.
    fn supports_reasoning(&self) -> bool {
        false
    }

    /// Check if the provider supports PDF inputs.
    fn supports_pdf(&self) -> bool {
        false
    }

    /// Create a chat completion.
    async fn completion(&self, params: CompletionParams) -> Result<ChatCompletion>;

    /// Create a streaming chat completion.
    async fn completion_stream(&self, params: CompletionParams) -> Result<CompletionStream>;
}

/// Create a provider instance.
pub fn create_provider(provider: LLMProvider, config: ProviderConfig) -> Result<Box<dyn Provider>> {
    match provider {
        LLMProvider::OpenAI => {
            let p = crate::providers::openai::OpenAIProvider::new(config)?;
            Ok(Box::new(p))
        }
        LLMProvider::Anthropic => {
            let p = crate::providers::anthropic::AnthropicProvider::new(config)?;
            Ok(Box::new(p))
        }
    }
}
