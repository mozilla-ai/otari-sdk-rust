//! Streaming chunk types for chat completions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::completion::Reasoning;
use super::message::Role;
use super::tool::ToolCallDelta;
use super::usage::CompletionUsage;

/// A delta (partial update) in a streaming response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChoiceDelta {
    /// The role of the message author (only in first chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,

    /// Partial content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Partial tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,

    /// Partial reasoning content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,

    /// Optional refusal message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
}

/// A choice in a streaming chunk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkChoice {
    /// The index of this choice.
    pub index: u32,

    /// The delta (partial update).
    pub delta: ChoiceDelta,

    /// The reason the generation stopped (only in final chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    /// Log probabilities, if requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Value>,
}

/// A streaming chunk of a chat completion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCompletionChunk {
    /// Unique identifier for this completion.
    pub id: String,

    /// The object type (always "chat.completion.chunk").
    pub object: String,

    /// Unix timestamp of when the chunk was created.
    pub created: i64,

    /// The model used for the completion.
    pub model: String,

    /// The partial choices.
    pub choices: Vec<ChunkChoice>,

    /// Token usage (only in final chunk if stream_options.include_usage is true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,

    /// System fingerprint for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

impl ChatCompletionChunk {
    /// Create an empty chunk (used internally for skippable events).
    pub(crate) fn empty(model: &str) -> Self {
        Self {
            id: String::new(),
            object: "chat.completion.chunk".to_string(),
            created: 0,
            model: model.to_string(),
            choices: vec![],
            usage: None,
            system_fingerprint: None,
        }
    }

    /// Get the content delta from the first choice.
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.delta.content.as_deref())
    }

    /// Get the reasoning delta from the first choice.
    pub fn reasoning(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.delta.reasoning.as_ref())
            .map(|r| r.content.as_str())
    }

    /// Get the tool call deltas from the first choice.
    pub fn tool_calls(&self) -> Option<&[ToolCallDelta]> {
        self.choices
            .first()
            .and_then(|c| c.delta.tool_calls.as_deref())
    }

    /// Check if this is the final chunk.
    pub fn is_final(&self) -> bool {
        self.choices
            .first()
            .map(|c| c.finish_reason.is_some())
            .unwrap_or(false)
    }

    /// Get the finish reason if this is the final chunk.
    pub fn finish_reason(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.finish_reason.as_deref())
    }
}

/// Accumulator for streaming chunks into a complete response.
#[derive(Debug, Default)]
pub struct ChunkAccumulator {
    /// Accumulated content.
    pub content: String,

    /// Accumulated reasoning.
    pub reasoning: String,

    /// Accumulated tool calls (by index).
    pub tool_calls: Vec<AccumulatedToolCall>,

    /// The finish reason from the final chunk.
    pub finish_reason: Option<String>,

    /// The model name.
    pub model: Option<String>,

    /// The completion ID.
    pub id: Option<String>,

    /// Token usage from the final chunk.
    pub usage: Option<CompletionUsage>,
}

/// An accumulated tool call from streaming.
#[derive(Debug, Clone, Default)]
pub struct AccumulatedToolCall {
    /// The tool call ID.
    pub id: String,
    /// The tool type.
    pub tool_type: String,
    /// The function name.
    pub name: String,
    /// Accumulated arguments string.
    pub arguments: String,
}

impl ChunkAccumulator {
    /// Create a new accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a chunk to the accumulator.
    pub fn add(&mut self, chunk: &ChatCompletionChunk) {
        // Store metadata from first chunk
        if self.id.is_none() {
            self.id = Some(chunk.id.clone());
            self.model = Some(chunk.model.clone());
        }

        // Process each choice
        for choice in &chunk.choices {
            // Accumulate content
            if let Some(content) = &choice.delta.content {
                self.content.push_str(content);
            }

            // Accumulate reasoning
            if let Some(reasoning) = &choice.delta.reasoning {
                self.reasoning.push_str(&reasoning.content);
            }

            // Accumulate tool calls
            if let Some(tool_calls) = &choice.delta.tool_calls {
                for tc in tool_calls {
                    let index = tc.index.unwrap_or(0) as usize;

                    // Extend vector if needed
                    while self.tool_calls.len() <= index {
                        self.tool_calls.push(AccumulatedToolCall::default());
                    }

                    let accumulated = &mut self.tool_calls[index];

                    if let Some(id) = &tc.id {
                        accumulated.id = id.clone();
                    }
                    if let Some(tool_type) = &tc.tool_type {
                        accumulated.tool_type = tool_type.clone();
                    }
                    if let Some(function) = &tc.function {
                        if let Some(name) = &function.name {
                            accumulated.name = name.clone();
                        }
                        if let Some(args) = &function.arguments {
                            accumulated.arguments.push_str(args);
                        }
                    }
                }
            }

            // Store finish reason
            if let Some(reason) = &choice.finish_reason {
                self.finish_reason = Some(reason.clone());
            }
        }

        // Store usage from final chunk
        if let Some(usage) = &chunk.usage {
            self.usage = Some(usage.clone());
        }
    }

    /// Check if we have any content accumulated.
    pub fn has_content(&self) -> bool {
        !self.content.is_empty()
    }

    /// Check if we have any reasoning accumulated.
    pub fn has_reasoning(&self) -> bool {
        !self.reasoning.is_empty()
    }

    /// Check if we have any tool calls accumulated.
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}
