//! Moderation types.
//!
//! These mirror the OpenAI `/v1/moderations` request/response shape and
//! are used by the gateway provider's inherent `moderation` method.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Input to a moderation request.
///
/// Serialized as untagged JSON so the body matches the OpenAI-compatible
/// shape: either a single string, an array of strings, or an array of
/// multimodal content parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModerationInput {
    /// A single text string.
    Text(String),
    /// A batch of text strings.
    Batch(Vec<String>),
    /// Multimodal content parts (OpenAI `omni-moderation-*` only).
    Parts(Vec<ModerationContentPart>),
}

impl Default for ModerationInput {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

/// A single multimodal content part for moderation input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModerationContentPart {
    /// A text part.
    Text {
        /// The text content.
        text: String,
    },
    /// An image referenced by URL.
    ImageUrl {
        /// The image URL descriptor.
        image_url: ModerationImageUrl,
    },
}

/// An image URL reference for a moderation content part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationImageUrl {
    /// The URL of the image (HTTP(S) or `data:` URI).
    pub url: String,
}

/// Parameters for a moderation request.
///
/// `include_raw` is intentionally not serialized into the HTTP body; it
/// is emitted as the `?include_raw=true` query string by the gateway
/// client when `true`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModerationParams {
    /// Namespaced model identifier (e.g. `openai:omni-moderation-latest`).
    pub model: String,

    /// The input to moderate.
    pub input: ModerationInput,

    /// Optional end-user identifier for abuse monitoring.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// When `true`, the gateway should echo the upstream raw response
    /// alongside its normalized output. Serialized as a query parameter
    /// (`?include_raw=true`), not in the body.
    #[serde(skip_serializing)]
    pub include_raw: bool,
}

impl ModerationParams {
    /// Create a new moderation request for the given model and input.
    pub fn new(model: impl Into<String>, input: ModerationInput) -> Self {
        Self {
            model: model.into(),
            input,
            user: None,
            include_raw: false,
        }
    }

    /// Attach an end-user identifier.
    #[must_use]
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Ask the gateway to include the upstream provider's raw response.
    #[must_use]
    pub fn with_include_raw(mut self, include_raw: bool) -> Self {
        self.include_raw = include_raw;
        self
    }
}

/// A single moderation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationResult {
    /// Whether the input was flagged by any category.
    pub flagged: bool,

    /// Map of category name to whether it tripped.
    #[serde(default)]
    pub categories: HashMap<String, bool>,

    /// Map of category name to model confidence score.
    #[serde(default)]
    pub category_scores: HashMap<String, f64>,

    /// Which input modalities contributed to each category (OpenAI
    /// omni-moderation only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_applied_input_types: Option<HashMap<String, Vec<String>>>,

    /// The upstream provider's raw payload when `include_raw=true` was
    /// requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_raw: Option<serde_json::Value>,
}

/// Top-level moderation response (OpenAI-compatible shape).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationResponse {
    /// Opaque response identifier.
    pub id: String,
    /// Echoed model identifier.
    pub model: String,
    /// One result per input item.
    pub results: Vec<ModerationResult>,
}
