//! High-level API functions for easy LLM access.
//!
//! This module provides simple, stateless functions for making LLM calls.
//! For more control or connection reuse, use the `Provider` trait directly.

use crate::error::Result;
use crate::provider::{AnyLLMProvider, CompletionStream, ProviderConfig};
use crate::types::{
    ChatCompletion, CompletionParams, Message, ReasoningEffort, RerankParams, RerankResponse,
    StopSequence, Tool, ToolChoice,
};
use crate::Provider;
use serde_json::Value;

/// Options for completion requests.
#[derive(Debug, Clone, Default)]
pub struct CompletionOptions {
    /// API key (if not set, uses environment variable).
    pub api_key: Option<String>,

    /// API base URL (for custom endpoints/proxies).
    pub api_base: Option<String>,

    /// Tools available to the model.
    pub tools: Option<Vec<Tool>>,

    /// How the model should choose which tool to use.
    pub tool_choice: Option<ToolChoice>,

    /// Sampling temperature (0.0 to 2.0).
    pub temperature: Option<f32>,

    /// Nucleus sampling parameter.
    pub top_p: Option<f32>,

    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,

    /// Number of completions to generate.
    pub n: Option<u32>,

    /// Stop sequences.
    pub stop: Option<StopSequence>,

    /// Presence penalty (-2.0 to 2.0).
    pub presence_penalty: Option<f32>,

    /// Frequency penalty (-2.0 to 2.0).
    pub frequency_penalty: Option<f32>,

    /// Random seed for reproducibility.
    pub seed: Option<i64>,

    /// User identifier for abuse detection.
    pub user: Option<String>,

    /// Whether to allow parallel tool calls.
    pub parallel_tool_calls: Option<bool>,

    /// Whether to return log probabilities.
    pub logprobs: Option<bool>,

    /// Number of top log probabilities to return.
    pub top_logprobs: Option<u32>,

    /// Logit bias for specific tokens.
    pub logit_bias: Option<std::collections::HashMap<String, f32>>,

    /// Response format (e.g., JSON mode).
    pub response_format: Option<Value>,

    /// Reasoning effort for models that support extended thinking.
    pub reasoning_effort: Option<ReasoningEffort>,
}

impl CompletionOptions {
    /// Create new options with an API key.
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            ..Default::default()
        }
    }

    /// Set the API base URL.
    pub fn api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = Some(api_base.into());
        self
    }

    /// Set the temperature.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the max tokens.
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set the tools.
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set the tool choice.
    pub fn tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Set the reasoning effort.
    pub fn reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }
}

impl From<CompletionOptions> for ProviderConfig {
    fn from(options: CompletionOptions) -> Self {
        ProviderConfig {
            api_key: options.api_key,
            api_base: options.api_base,
            extra: Default::default(),
        }
    }
}

/// Create a chat completion.
///
/// # Arguments
///
/// * `model` - Model ID string
/// * `messages` - The conversation messages
/// * `options` - Optional configuration (API key, temperature, etc.)
///
/// # Examples
///
/// ```rust,no_run
/// use any_llm::{completion, Message, CompletionOptions, providers::Gateway};
///
/// #[tokio::main]
/// async fn main() -> any_llm::Result<()> {
///     let messages = vec![
///         Message::system("You are a helpful assistant."),
///         Message::user("What is the capital of France?"),
///     ];
///
///     let response = completion::<Gateway>(
///         "openai:gpt-4o-mini",
///         messages,
///         CompletionOptions::with_api_key("your-api-key")
///             .api_base("http://localhost:8000"),
///     ).await?;
///
///     println!("{}", response.content().unwrap_or("No response"));
///     Ok(())
/// }
/// ```
pub async fn completion<P: Provider>(
    model: &str,
    messages: Vec<Message>,
    options: CompletionOptions,
) -> Result<ChatCompletion> {
    let model_id = model.to_string();
    let provider = AnyLLMProvider::<P>::from_config(options.clone().into())?;

    let params = CompletionParams {
        model_id,
        messages,
        tools: options.tools,
        tool_choice: options.tool_choice,
        temperature: options.temperature,
        top_p: options.top_p,
        max_tokens: options.max_tokens,
        stream: Some(false),
        n: options.n,
        stop: options.stop,
        presence_penalty: options.presence_penalty,
        frequency_penalty: options.frequency_penalty,
        seed: options.seed,
        user: options.user,
        parallel_tool_calls: options.parallel_tool_calls,
        logprobs: options.logprobs,
        top_logprobs: options.top_logprobs,
        logit_bias: options.logit_bias,
        response_format: options.response_format,
        reasoning_effort: options.reasoning_effort,
    };

    provider.completion(params).await
}

/// Create a streaming chat completion.
///
/// # Arguments
///
/// * `model` - Model ID string (e.g., "gpt-4o-mini")
/// * `messages` - The conversation messages
/// * `options` - Optional configuration (API key, temperature, etc.)
///
/// # Examples
///
/// ```rust,no_run
/// use any_llm::{completion_stream, Message, CompletionOptions, providers::Gateway};
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> any_llm::Result<()> {
///     let messages = vec![Message::user("Tell me a story")];
///
///     let mut stream = completion_stream::<Gateway>(
///         "openai:gpt-4o-mini",
///         messages,
///         CompletionOptions::with_api_key("your-api-key")
///             .api_base("http://localhost:8000"),
///     ).await?;
///
///     while let Some(chunk) = stream.next().await {
///         let chunk = chunk?;
///         if let Some(content) = chunk.content() {
///             print!("{}", content);
///         }
///     }
///     Ok(())
/// }
/// ```
pub async fn completion_stream<P: Provider>(
    model: &str,
    messages: Vec<Message>,
    options: CompletionOptions,
) -> Result<CompletionStream> {
    let model_id = model.to_string();
    let provider = AnyLLMProvider::<P>::from_config(options.clone().into())?;

    let params = CompletionParams {
        model_id,
        messages,
        tools: options.tools,
        tool_choice: options.tool_choice,
        temperature: options.temperature,
        top_p: options.top_p,
        max_tokens: options.max_tokens,
        stream: Some(true),
        n: options.n,
        stop: options.stop,
        presence_penalty: options.presence_penalty,
        frequency_penalty: options.frequency_penalty,
        seed: options.seed,
        user: options.user,
        parallel_tool_calls: options.parallel_tool_calls,
        logprobs: options.logprobs,
        top_logprobs: options.top_logprobs,
        logit_bias: options.logit_bias,
        response_format: options.response_format,
        reasoning_effort: options.reasoning_effort,
    };

    provider.completion_stream(params).await
}

/// Options for a rerank request.
#[derive(Debug, Clone, Default)]
pub struct RerankOptions {
    /// API key (if not set, uses environment variable).
    pub api_key: Option<String>,

    /// API base URL (for custom endpoints/proxies).
    pub api_base: Option<String>,

    /// Maximum number of results to return.
    pub top_n: Option<u32>,

    /// Maximum tokens per document for truncation.
    pub max_tokens_per_doc: Option<u32>,

    /// User identifier for abuse detection.
    pub user: Option<String>,
}

impl From<RerankOptions> for ProviderConfig {
    fn from(options: RerankOptions) -> Self {
        ProviderConfig {
            api_key: options.api_key,
            api_base: options.api_base,
            extra: Default::default(),
        }
    }
}

/// Rerank documents by relevance to a query.
///
/// # Arguments
/// * `model` - Model identifier (e.g., "cohere:rerank-v3.5")
/// * `query` - The search query
/// * `documents` - Documents to rerank
/// * `options` - Additional options (API key, base URL, top_n, etc.)
///
/// # Returns
/// A `RerankResponse` with results sorted by `relevance_score` descending.
///
/// # Errors
/// Returns `AnyLLMError` if the provider does not support reranking or the request fails.
pub async fn rerank<P: Provider>(
    model: &str,
    query: &str,
    documents: Vec<String>,
    options: RerankOptions,
) -> Result<RerankResponse> {
    let provider = AnyLLMProvider::<P>::from_config(options.clone().into())?;
    let params = RerankParams {
        model_id: model.to_string(),
        query: query.to_string(),
        documents,
        top_n: options.top_n,
        max_tokens_per_doc: options.max_tokens_per_doc,
        user: options.user,
    };
    provider.rerank(params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_options_builder() {
        let options = CompletionOptions::with_api_key("test-key")
            .temperature(0.7)
            .max_tokens(100);

        assert_eq!(options.api_key, Some("test-key".to_string()));
        assert_eq!(options.temperature, Some(0.7));
        assert_eq!(options.max_tokens, Some(100));
    }
}
