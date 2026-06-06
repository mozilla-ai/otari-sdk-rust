//! Gateway SSE stream handling.
//!
//! The gateway emits OpenAI-format Server-Sent Events. Each event is a
//! `data: {json}` line, and the stream terminates with `data: [DONE]`.

use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use serde_json::Value;

use crate::types::{
    ChatCompletionChunk, ChoiceDelta, ChunkChoice, CompletionStream, CompletionUsage,
    FunctionDelta, Reasoning, ToolCallDelta,
};
use crate::OtariError;

/// A stream of raw SSE event payloads parsed as JSON `Value`s.
///
/// Used for the responses (`/responses`) and messages (`/messages`) event
/// streams, which (mirroring the Python reference) have no single typed chunk
/// model: callers get the raw parsed event. Terminates on `[DONE]` and on
/// end-of-stream.
pub type RawValueStream =
    std::pin::Pin<Box<dyn futures::Stream<Item = Result<Value, OtariError>> + Send + 'static>>;

/// Wrap a `reqwest-eventsource` source into a stream of raw JSON values,
/// stopping on the `[DONE]` sentinel (matching the chat shim's semantics).
///
/// Each emitted item is either a parsed event (`Ok`), or `None` for frames
/// that carry no chunk data (the connection-open frame, mirroring the Python
/// shim's skipping of non-`data:` framing lines). The `take_while` stops at
/// the `[DONE]` sentinel / end-of-stream; `filter_map` drops the `None`s.
pub fn raw_value_stream(source: EventSource) -> RawValueStream {
    let stream = source
        .map(|event| match event {
            Ok(Event::Message(msg)) => {
                // The [DONE] sentinel terminates the stream (outer `None`).
                if msg.data.trim() == "[DONE]" {
                    return None;
                }
                match serde_json::from_str::<Value>(&msg.data) {
                    Ok(value) => Some(Some(Ok(value))),
                    Err(e) => Some(Some(Err(OtariError::Streaming {
                        provider: "otari".into(),
                        message: format!("Failed to parse event: {e}").into(),
                    }))),
                }
            }
            // Connection-open frame carries no payload; emit a skippable item.
            Ok(Event::Open) => Some(None),
            Err(reqwest_eventsource::Error::StreamEnded) => None,
            Err(e) => Some(Some(Err(OtariError::Streaming {
                provider: "otari".into(),
                message: e.to_string().into(),
            }))),
        })
        // Stop on the [DONE] sentinel (outer `None`) and on end-of-stream.
        .take_while(|outer| std::future::ready(outer.is_some()))
        // Drop the connection-open skip (inner `None`); keep real items.
        .filter_map(|outer| std::future::ready(outer.flatten()));

    Box::pin(stream)
}

/// Field names that providers use for reasoning/thinking content in deltas.
const REASONING_FIELD_NAMES: &[&str] = &["reasoning", "reasoning_content", "thinking", "think"];

pub struct GatewayStream {
    source: EventSource,
    model: String,
}

impl GatewayStream {
    pub fn new(source: EventSource, model: String) -> Self {
        Self { source, model }
    }
}

impl TryInto<CompletionStream> for GatewayStream {
    type Error = OtariError;

    fn try_into(self) -> Result<CompletionStream, Self::Error> {
        let GatewayStream { source, model } = self;

        let stream = source
            .map(move |event| {
                let model = model.clone();
                match event {
                    Ok(Event::Message(msg)) => {
                        // The [DONE] sentinel terminates the stream.
                        if msg.data.trim() == "[DONE]" {
                            return None;
                        }

                        match serde_json::from_str::<GatewayChunkRaw>(&msg.data) {
                            Ok(raw) => Some(Ok(raw.into_chunk(&model))),
                            Err(e) => Some(Err(OtariError::Streaming {
                                provider: "otari".into(),
                                message: format!("Failed to parse chunk: {e}").into(),
                            })),
                        }
                    }
                    Ok(Event::Open) => Some(Ok(ChatCompletionChunk::empty(&model))),
                    Err(reqwest_eventsource::Error::StreamEnded) => None,
                    Err(e) => Some(Err(OtariError::Streaming {
                        provider: "otari".into(),
                        message: e.to_string().into(),
                    })),
                }
            })
            .take_while(|item| std::future::ready(item.is_some()))
            .filter_map(|item| std::future::ready(item))
            .filter(|result| {
                std::future::ready(match result {
                    Ok(chunk) => !chunk.choices.is_empty(),
                    Err(_) => true,
                })
            });

        Ok(Box::pin(stream))
    }
}

/// Raw streaming chunk from the gateway (OpenAI format with extra fields).
#[derive(Debug, Deserialize)]
struct GatewayChunkRaw {
    id: Option<String>,
    object: Option<String>,
    created: Option<i64>,
    model: Option<String>,
    choices: Option<Vec<RawChunkChoice>>,
    usage: Option<RawUsage>,
    system_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawChunkChoice {
    index: Option<u32>,
    delta: Option<RawDelta>,
    finish_reason: Option<String>,
    logprobs: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct RawDelta {
    role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<RawToolCallDelta>>,
    refusal: Option<String>,
    /// Catch reasoning fields.
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

#[derive(Debug, Deserialize)]
struct RawToolCallDelta {
    index: Option<u32>,
    id: Option<String>,
    #[serde(rename = "type")]
    tool_type: Option<String>,
    function: Option<RawFunctionDelta>,
}

#[derive(Debug, Deserialize)]
struct RawFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Deserialize)]
struct RawUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

impl GatewayChunkRaw {
    fn into_chunk(self, fallback_model: &str) -> ChatCompletionChunk {
        let choices = self
            .choices
            .unwrap_or_default()
            .into_iter()
            .map(|c| {
                let delta = c.delta.map(|d| {
                    let reasoning = extract_delta_reasoning(&d.extra);
                    let tool_calls = d.tool_calls.map(|tcs| {
                        tcs.into_iter()
                            .map(|tc| ToolCallDelta {
                                index: tc.index,
                                id: tc.id,
                                tool_type: tc.tool_type,
                                function: tc.function.map(|f| FunctionDelta {
                                    name: f.name,
                                    arguments: f.arguments,
                                }),
                            })
                            .collect()
                    });

                    let role = d.role.and_then(|r| match r.as_str() {
                        "assistant" => Some(crate::types::Role::Assistant),
                        "system" => Some(crate::types::Role::System),
                        "user" => Some(crate::types::Role::User),
                        "tool" => Some(crate::types::Role::Tool),
                        _ => None,
                    });

                    ChoiceDelta {
                        role,
                        content: d.content,
                        tool_calls,
                        reasoning,
                        refusal: d.refusal,
                    }
                });

                ChunkChoice {
                    index: c.index.unwrap_or(0),
                    delta: delta.unwrap_or_default(),
                    finish_reason: c.finish_reason,
                    logprobs: c.logprobs,
                }
            })
            .collect();

        let usage = self.usage.map(|u| {
            let prompt = u.prompt_tokens.unwrap_or(0);
            let completion = u.completion_tokens.unwrap_or(0);
            let total = u.total_tokens.unwrap_or(prompt + completion);
            CompletionUsage {
                prompt_tokens: prompt,
                completion_tokens: completion,
                total_tokens: total,
            }
        });

        ChatCompletionChunk {
            id: self.id.unwrap_or_default(),
            object: self
                .object
                .unwrap_or_else(|| "chat.completion.chunk".to_string()),
            created: self.created.unwrap_or(0),
            model: self.model.unwrap_or_else(|| fallback_model.to_string()),
            choices,
            usage,
            system_fingerprint: self.system_fingerprint,
        }
    }
}

/// Extract reasoning content from a delta's extra fields.
fn extract_delta_reasoning(extra: &serde_json::Map<String, Value>) -> Option<Reasoning> {
    for field in REASONING_FIELD_NAMES {
        if let Some(val) = extra.get(*field) {
            if let Some(text) = val.as_str() {
                if !text.is_empty() {
                    return Some(Reasoning::new(text));
                }
            } else if let Some(content) = val.get("content").and_then(|c| c.as_str()) {
                if !content.is_empty() {
                    return Some(Reasoning::new(content));
                }
            }
        }
    }
    None
}
