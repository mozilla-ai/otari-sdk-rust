//! OpenAI provider implementation.

use async_openai::{config::OpenAIConfig, Client};
use async_trait::async_trait;
use futures::StreamExt;

use crate::error::{AnyLLMError, Result};
use crate::provider::{CompletionStream, Provider, ProviderConfig};
use crate::types::{ChatCompletion, CompletionParams};

mod message;
mod request;
mod response;
mod stream;
mod tool;

/// OpenAI provider using the async-openai SDK.
pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| AnyLLMError::MissingApiKey {
                provider: "openai".to_string(),
                env_var: "OPENAI_API_KEY".to_string(),
            })?;

        let mut openai_config = OpenAIConfig::new().with_api_key(api_key);

        if let Some(api_base) = config.api_base {
            openai_config = openai_config.with_api_base(api_base);
        }

        Ok(Self {
            client: Client::with_config(openai_config),
        })
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn supports_images(&self) -> bool {
        true
    }

    fn supports_reasoning(&self) -> bool {
        true // o1, o3 models support reasoning
    }

    async fn completion(&self, params: CompletionParams) -> Result<ChatCompletion> {
        let request = params.try_into()?;

        // Make the API call
        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(convert_error)?;

        Ok(response.into())
    }

    async fn completion_stream(&self, params: CompletionParams) -> Result<CompletionStream> {
        let request = params.try_into()?;

        // Create the stream
        let stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(convert_error)?;

        // Map the stream to our types
        let mapped_stream = stream.map(|result| match result {
            Ok(chunk) => Ok(chunk.into()),
            Err(e) => Err(convert_error(e)),
        });

        Ok(Box::pin(mapped_stream))
    }
}

/// Convert OpenAI error to our error type.
fn convert_error(error: async_openai::error::OpenAIError) -> AnyLLMError {
    let message = error.to_string();

    // Try to detect error type from message
    if message.contains("rate limit") || message.contains("429") {
        AnyLLMError::rate_limit("openai", message)
    } else if message.contains("authentication")
        || message.contains("401")
        || message.contains("invalid api key")
    {
        AnyLLMError::authentication("openai", message)
    } else if message.contains("not found") || message.contains("404") {
        AnyLLMError::ModelNotFound {
            provider: "openai".to_string(),
            model: "unknown".to_string(),
        }
    } else if message.contains("400") || message.contains("bad request") {
        AnyLLMError::invalid_request("openai", message)
    } else {
        AnyLLMError::provider_error("openai", message)
    }
}
