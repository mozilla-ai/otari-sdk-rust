//! Error types for any-llm.
//!
//! This module provides a unified error hierarchy that maps provider-specific
//! errors to common error types.

use std::borrow::Cow;
use thiserror::Error;

type ErrorStr = Cow<'static, str>;

/// The main error type for any-llm operations.
#[derive(Error, Debug)]
pub enum AnyLLMError {
    /// Rate limit exceeded - the provider is throttling requests.
    #[error("Rate limit exceeded for {provider}: {message}")]
    RateLimit {
        message: ErrorStr,
        provider: ErrorStr,
    },

    /// Authentication failed - invalid or missing API key.
    #[error("Authentication failed for {provider}: {message}")]
    Authentication {
        message: ErrorStr,
        provider: ErrorStr,
    },

    /// Invalid request - the request parameters were invalid.
    #[error("Invalid request for {provider}: {message}")]
    InvalidRequest {
        message: ErrorStr,
        provider: ErrorStr,
    },

    /// Provider error - a general error from the provider.
    #[error("Provider error from {provider}: {message}")]
    Provider {
        message: ErrorStr,
        provider: ErrorStr,
    },

    /// Model not found - the requested model doesn't exist.
    #[error("Model '{model}' not found for {provider}")]
    ModelNotFound { model: ErrorStr, provider: ErrorStr },

    /// Context length exceeded - the input was too long.
    #[error("Context length exceeded for {provider}: {message}")]
    ContextLengthExceeded {
        message: ErrorStr,
        provider: ErrorStr,
    },

    /// Content filter triggered - the content was blocked.
    #[error("Content filtered by {provider}: {message}")]
    ContentFilter {
        message: ErrorStr,
        provider: ErrorStr,
    },

    /// Missing API key - no API key was provided or found in environment.
    #[error("Missing API key for {provider}. Set the {env_var} environment variable or pass api_key in config.")]
    MissingApiKey {
        provider: ErrorStr,
        env_var: ErrorStr,
    },

    /// Unsupported provider - the provider is not supported.
    #[error("Unsupported provider: '{provider}'. Supported providers: openai, anthropic")]
    UnsupportedProvider { provider: ErrorStr },

    /// Unsupported parameter - a parameter is not supported by the provider.
    #[error("Parameter '{param}' is not supported by {provider}. {hint}")]
    UnsupportedParameter {
        param: ErrorStr,
        provider: ErrorStr,
        hint: ErrorStr,
    },

    /// Streaming error - an error occurred during streaming.
    #[error("Streaming error from {provider}: {message}")]
    Streaming {
        message: ErrorStr,
        provider: ErrorStr,
    },

    /// Serialization error - failed to serialize/deserialize data.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP error - a network or HTTP error occurred.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

/// A specialized Result type for any-llm operations.
pub type Result<T> = std::result::Result<T, AnyLLMError>;

impl AnyLLMError {
    /// Create a new rate limit error.
    pub fn rate_limit(provider: impl Into<ErrorStr>, message: impl Into<ErrorStr>) -> Self {
        Self::RateLimit {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a new authentication error.
    pub fn authentication(provider: impl Into<ErrorStr>, message: impl Into<ErrorStr>) -> Self {
        Self::Authentication {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a new invalid request error.
    pub fn invalid_request(provider: impl Into<ErrorStr>, message: impl Into<ErrorStr>) -> Self {
        Self::InvalidRequest {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a new provider error.
    pub fn provider_error(provider: impl Into<ErrorStr>, message: impl Into<ErrorStr>) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a new model not found error.
    pub fn model_not_found(provider: impl Into<ErrorStr>, model: impl Into<ErrorStr>) -> Self {
        Self::ModelNotFound {
            provider: provider.into(),
            model: model.into(),
        }
    }

    /// Create a new unsupported parameter error.
    pub fn unsupported_parameter(
        provider: impl Into<ErrorStr>,
        param: impl Into<ErrorStr>,
        hint: impl Into<ErrorStr>,
    ) -> Self {
        Self::UnsupportedParameter {
            provider: provider.into(),
            param: param.into(),
            hint: hint.into(),
        }
    }

    /// Get the provider name associated with this error, if any.
    pub fn provider(&self) -> Option<&str> {
        match self {
            Self::RateLimit { provider, .. }
            | Self::Authentication { provider, .. }
            | Self::InvalidRequest { provider, .. }
            | Self::Provider { provider, .. }
            | Self::ModelNotFound { provider, .. }
            | Self::ContextLengthExceeded { provider, .. }
            | Self::ContentFilter { provider, .. }
            | Self::MissingApiKey { provider, .. }
            | Self::UnsupportedProvider { provider, .. }
            | Self::UnsupportedParameter { provider, .. }
            | Self::Streaming { provider, .. } => Some(provider),
            Self::Serialization(_) | Self::Http(_) => None,
        }
    }
}
