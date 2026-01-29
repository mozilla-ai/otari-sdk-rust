//! Anthropic (Claude) provider implementation.

use async_trait::async_trait;
use reqwest::Client;
use reqwest_eventsource::EventSource;

use crate::error::{AnyLLMError, Result};
use crate::provider::{CompletionStream, Provider, ProviderConfig};
use crate::types::{ChatCompletion, CompletionParams};

mod models;

use models::request::AnthropicRequest;
use models::response::AnthropicResponse;
use models::stream::AnthropicStream;

/// Default max tokens for Anthropic (required parameter).
const DEFAULT_MAX_TOKENS: u32 = 8192;

/// API version for Anthropic.
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Default API base URL.
const DEFAULT_API_BASE: &str = "https://api.anthropic.com";

/// Anthropic provider using the Messages API.
pub struct Anthropic {
    client: Client,
    api_key: String,
    api_base: String,
}

#[async_trait]
impl Provider for Anthropic {
    const NAME: &'static str = "anthropic";
    const ENV_VAR: &'static str = "ANTHROPIC_API_KEY";
    const DOCS_URL: &'static str = "https://docs.anthropic.com/en/home";

    const SUPPORTS_IMAGES: bool = true;
    const SUPPORTS_REASONING: bool = true;

    /// Create a new Anthropic provider.
    fn from_config(config: ProviderConfig) -> Result<Self> {
        let api_key = Self::api_key(&config).ok_or_else(|| AnyLLMError::MissingApiKey {
            provider: Self::NAME.to_string(),
            env_var: Self::ENV_VAR.to_string(),
        })?;

        let api_base = config
            .api_base
            .unwrap_or_else(|| DEFAULT_API_BASE.to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
            api_base,
        })
    }

    async fn completion_fn(&self, params: CompletionParams) -> Result<impl Into<ChatCompletion>> {
        let body: AnthropicRequest = params.try_into()?;

        // Make the API call
        let response = self
            .client
            .post(format!("{}/v1/messages", self.api_base))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status().as_u16();
        if status != 200 {
            let body = response.text().await.unwrap_or_default();
            return Err(convert_error(status, &body));
        }

        Ok(response.json::<AnthropicResponse>().await?)
    }

    async fn completion_stream_fn(&self, params: CompletionParams) -> Result<CompletionStream> {
        let model = params.model_id.clone();

        let body = TryInto::<AnthropicRequest>::try_into(params)?.stream();

        // Create request
        let request = self
            .client
            .post(format!("{}/v1/messages", self.api_base))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body);

        // Create SSE stream
        let es = EventSource::new(request).map_err(|e| AnyLLMError::Streaming {
            provider: "anthropic".to_string(),
            message: e.to_string(),
        })?;

        let stream = AnthropicStream::new(es, model);

        stream.try_into()
    }
}

/// Convert Anthropic HTTP error to any-llm-rust error type.
fn convert_error(status: u16, body: &str) -> AnyLLMError {
    match status {
        429 => AnyLLMError::rate_limit("anthropic", body),
        401 => AnyLLMError::authentication("anthropic", body),
        400 => AnyLLMError::invalid_request("anthropic", body),
        404 => AnyLLMError::ModelNotFound {
            provider: "anthropic".to_string(),
            model: "unknown".to_string(),
        },
        _ => AnyLLMError::provider_error("anthropic", format!("Status {}: {}", status, body)),
    }
}
