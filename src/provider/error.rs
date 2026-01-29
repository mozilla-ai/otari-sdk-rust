use super::interface::Provider;
use crate::error::{AnyLLMError, ErrorStr};

impl AnyLLMError {
    /// Create a new rate limit error.
    pub fn rate_limit<P: Provider>(message: impl Into<ErrorStr>) -> Self {
        Self::RateLimit {
            provider: P::NAME.into(),
            message: message.into(),
        }
    }

    /// Create a new authentication error.
    pub fn authentication<P: Provider>(message: impl Into<ErrorStr>) -> Self {
        Self::Authentication {
            provider: P::NAME.into(),
            message: message.into(),
        }
    }

    /// Create a new invalid request error.
    pub fn invalid_request<P: Provider>(message: impl Into<ErrorStr>) -> Self {
        Self::InvalidRequest {
            provider: P::NAME.into(),
            message: message.into(),
        }
    }

    /// Create a new provider error.
    pub fn provider_error<P: Provider>(message: impl Into<ErrorStr>) -> Self {
        Self::Provider {
            provider: P::NAME.into(),
            message: message.into(),
        }
    }

    /// Create a new model not found error.
    pub fn model_not_found<P: Provider>(model: impl Into<ErrorStr>) -> Self {
        Self::ModelNotFound {
            provider: P::NAME.into(),
            model: model.into(),
        }
    }

    /// Create a new unsupported parameter error.
    pub fn unsupported_parameter<P: Provider>(
        param: impl Into<ErrorStr>,
        hint: impl Into<ErrorStr>,
    ) -> Self {
        Self::UnsupportedParameter {
            provider: P::NAME.into(),
            param: param.into(),
            hint: hint.into(),
        }
    }
}
