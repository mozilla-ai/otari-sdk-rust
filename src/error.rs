//! Error types for otari.
//!
//! This module provides a unified error hierarchy that maps gateway
//! errors to common error types.

use std::borrow::Cow;
use thiserror::Error;

pub type ErrorStr = Cow<'static, str>;

/// The main error type for otari operations.
#[derive(Error, Debug)]
pub enum OtariError {
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
    #[error("Unsupported provider: '{provider}'.")]
    UnsupportedProvider { provider: ErrorStr },

    /// Unsupported operation - the provider does not support this operation.
    #[error("Operation '{operation}' is not supported by {provider}")]
    Unsupported {
        provider: ErrorStr,
        operation: ErrorStr,
    },

    /// Unsupported parameter - a parameter is not supported by the provider.
    #[error("Parameter '{param}' is not supported by {provider}. {hint}")]
    UnsupportedParameter {
        param: ErrorStr,
        provider: ErrorStr,
        hint: ErrorStr,
    },

    /// Batch not yet complete — results cannot be retrieved.
    #[error("Batch '{batch_id}' is not yet complete (status: {status}). Check status with retrieve_batch().")]
    BatchNotComplete {
        batch_id: ErrorStr,
        status: ErrorStr,
        provider: ErrorStr,
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

/// A specialized Result type for otari operations.
pub type Result<T> = std::result::Result<T, OtariError>;

// ── Error helper constructors ───────────────────────────────────────

const PROVIDER_NAME: &str = "otari";

impl OtariError {
    /// Create a new rate limit error.
    pub fn rate_limit(message: impl Into<ErrorStr>) -> Self {
        Self::RateLimit {
            provider: PROVIDER_NAME.into(),
            message: message.into(),
        }
    }

    /// Create a new authentication error.
    pub fn authentication(message: impl Into<ErrorStr>) -> Self {
        Self::Authentication {
            provider: PROVIDER_NAME.into(),
            message: message.into(),
        }
    }

    /// Create a new invalid request error.
    pub fn invalid_request(message: impl Into<ErrorStr>) -> Self {
        Self::InvalidRequest {
            provider: PROVIDER_NAME.into(),
            message: message.into(),
        }
    }

    /// Create a new provider error.
    pub fn provider_error(message: impl Into<ErrorStr>) -> Self {
        Self::Provider {
            provider: PROVIDER_NAME.into(),
            message: message.into(),
        }
    }

    /// Create a new model not found error.
    pub fn model_not_found(model: impl Into<ErrorStr>) -> Self {
        Self::ModelNotFound {
            provider: PROVIDER_NAME.into(),
            model: model.into(),
        }
    }

    /// Create a new unsupported parameter error.
    pub fn unsupported_parameter(param: impl Into<ErrorStr>, hint: impl Into<ErrorStr>) -> Self {
        Self::UnsupportedParameter {
            provider: PROVIDER_NAME.into(),
            param: param.into(),
            hint: hint.into(),
        }
    }

    /// Create a new unsupported operation error.
    pub fn unsupported(operation: impl Into<ErrorStr>) -> Self {
        Self::Unsupported {
            provider: PROVIDER_NAME.into(),
            operation: operation.into(),
        }
    }

    /// Create a new unsupported operation error when the provider name is
    /// only known at runtime (e.g. parsed from an HTTP response body).
    pub fn unsupported_dynamic(
        provider: impl Into<ErrorStr>,
        operation: impl Into<ErrorStr>,
    ) -> Self {
        Self::Unsupported {
            provider: provider.into(),
            operation: operation.into(),
        }
    }
}
