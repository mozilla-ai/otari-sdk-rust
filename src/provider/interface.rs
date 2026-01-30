use futures::Future;

use crate::{
    error::{AnyLLMError, Result},
    types::Content,
    types::{ChatCompletion, CompletionParams},
};

use super::{config::ProviderConfig, CompletionStream};

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
            return Err(AnyLLMError::provider_error::<Self>(
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
                                return Err(AnyLLMError::provider_error::<Self>(
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
            return Err(AnyLLMError::provider_error::<Self>(
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
                return Err(AnyLLMError::provider_error::<Self>(
                    "Provider does not support streaming",
                ));
            }

            Self::validate_completion_params(&params)?;

            self.completion_stream_fn(params).await
        }
    }
}
