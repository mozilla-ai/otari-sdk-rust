//! Completion request and response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::message::{Message, Role};
use super::tool::{Tool, ToolCall, ToolChoice};
use super::usage::CompletionUsage;

/// The level of reasoning effort for models that support extended thinking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    /// No reasoning - disable extended thinking.
    None,
    /// Minimal reasoning effort.
    Minimal,
    /// Low reasoning effort.
    Low,
    /// Medium reasoning effort.
    Medium,
    /// High reasoning effort.
    High,
    /// Auto - let the provider decide.
    #[default]
    Auto,
}

/// Stop sequence - can be a single string or multiple strings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StopSequence {
    /// A single stop sequence.
    Single(String),
    /// Multiple stop sequences.
    Multiple(Vec<String>),
}

impl StopSequence {
    /// Create a stop sequence from a single string.
    pub fn single(s: impl Into<String>) -> Self {
        StopSequence::Single(s.into())
    }

    /// Create stop sequences from multiple strings.
    pub fn multiple(sequences: Vec<String>) -> Self {
        StopSequence::Multiple(sequences)
    }

    /// Convert to a vector of strings.
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            StopSequence::Single(s) => vec![s.clone()],
            StopSequence::Multiple(v) => v.clone(),
        }
    }
}

/// Parameters for a completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionParams {
    /// The model identifier (without provider prefix).
    pub model_id: String,

    /// The messages in the conversation.
    pub messages: Vec<Message>,

    /// Tools available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// How the model should choose which tool to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Sampling temperature (0.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Nucleus sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Maximum tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Number of completions to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    /// Stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopSequence>,

    /// Presence penalty (-2.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Frequency penalty (-2.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Random seed for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// User identifier for abuse detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Whether to allow parallel tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// Whether to return log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Number of top log probabilities to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,

    /// Logit bias for specific tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<std::collections::HashMap<String, f32>>,

    /// Response format (e.g., JSON mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,

    /// Reasoning effort for models that support extended thinking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
}

impl Default for CompletionParams {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            messages: Vec::new(),
            tools: None,
            tool_choice: None,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: None,
            n: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            seed: None,
            user: None,
            parallel_tool_calls: None,
            logprobs: None,
            top_logprobs: None,
            logit_bias: None,
            response_format: None,
            reasoning_effort: None,
        }
    }
}

impl CompletionParams {
    /// Create new completion params with model and messages.
    pub fn new(model_id: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model_id: model_id.into(),
            messages,
            ..Default::default()
        }
    }

    /// Set the temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the max tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set streaming mode.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Set tools.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set tool choice.
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Set reasoning effort.
    pub fn with_reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }
}

/// Extended thinking/reasoning content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reasoning {
    /// The reasoning content.
    pub content: String,
}

impl Reasoning {
    /// Create new reasoning content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

/// A message in a completion response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCompletionMessage {
    /// The role of the message author.
    pub role: Role,

    /// The content of the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Tool calls made by the assistant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Extended thinking/reasoning content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,

    /// Optional refusal message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
}

impl Default for ChatCompletionMessage {
    fn default() -> Self {
        Self {
            role: Role::Assistant,
            content: None,
            tool_calls: None,
            reasoning: None,
            refusal: None,
        }
    }
}

/// A choice in a completion response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Choice {
    /// The index of this choice.
    pub index: u32,

    /// The generated message.
    pub message: ChatCompletionMessage,

    /// The reason the generation stopped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    /// Log probabilities, if requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Value>,
}

/// A chat completion response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCompletion {
    /// Unique identifier for this completion.
    pub id: String,

    /// The object type (always "chat.completion").
    pub object: String,

    /// Unix timestamp of when the completion was created.
    pub created: i64,

    /// The model used for the completion.
    pub model: String,

    /// The generated choices.
    pub choices: Vec<Choice>,

    /// Token usage statistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,

    /// System fingerprint for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

impl ChatCompletion {
    /// Get the first choice's message content.
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.message.content.as_deref())
    }

    /// Get the first choice's tool calls.
    pub fn tool_calls(&self) -> Option<&[ToolCall]> {
        self.choices
            .first()
            .and_then(|c| c.message.tool_calls.as_deref())
    }

    /// Get the first choice's reasoning content.
    pub fn reasoning(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.message.reasoning.as_ref())
            .map(|r| r.content.as_str())
    }

    /// Get the finish reason for the first choice.
    pub fn finish_reason(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.finish_reason.as_deref())
    }
}
