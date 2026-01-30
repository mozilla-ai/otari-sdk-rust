use crate::{
    provider::CompletionStream,
    types::{
        ChatCompletionChunk, ChoiceDelta, ChunkChoice, CompletionUsage, FunctionDelta, Reasoning,
        ToolCallDelta,
    },
    AnyLLMError,
};
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;

pub struct AnthropicStream {
    source: EventSource,
    model: String,
}

impl AnthropicStream {
    pub fn new(source: EventSource, model: String) -> Self {
        Self { source, model }
    }
}

impl TryInto<CompletionStream> for AnthropicStream {
    type Error = AnyLLMError;

    fn try_into(self) -> Result<CompletionStream, Self::Error> {
        // Use scan to track when we should stop, then filter_map to convert events
        // This properly terminates the stream on StreamEnded or errors
        let stream = self.source
            .map(move |event| {
                let model = self.model.clone();
                match event {
                    Ok(Event::Message(msg)) => {
                        // Parse the SSE data
                        if let Ok(stream_event) =
                            serde_json::from_str::<AnthropicStreamEvent>(&msg.data)
                        {
                            stream_event
                                .to_chat_completion_chunk(&model)
                                .map(|chunk| Some(Ok(chunk)))
                                .unwrap_or(Some(Ok(ChatCompletionChunk::empty(&model))))
                        } else {
                            // Skip unparseable events
                            Some(Ok(ChatCompletionChunk::empty(&model)))
                        }
                    }
                    Ok(Event::Open) => Some(Ok(ChatCompletionChunk::empty(&model))),
                    Err(reqwest_eventsource::Error::StreamEnded) => {
                        // Normal stream termination - signal end
                        None
                    }
                    Err(e) => Some(Err(AnyLLMError::Streaming {
                        provider: "anthropic".into(),
                        message: e.to_string().into(),
                    })),
                }
            })
            // Stop the stream when we get None (StreamEnded)
            .take_while(|item| std::future::ready(item.is_some()))
            // Unwrap the Option layer
            .filter_map(|item| std::future::ready(item))
            // Filter out empty chunks
            .filter(|result| {
                std::future::ready(match result {
                    Ok(chunk) => !chunk.choices.is_empty(),
                    Err(_) => true,
                })
            });

        Ok(Box::pin(stream))
    }
}

#[derive(Debug, Deserialize)]
pub struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    index: Option<u32>,
    content_block: Option<StreamContentBlock>,
    delta: Option<StreamDelta>,
    usage: Option<StreamUsage>,
}

impl AnthropicStreamEvent {
    /// Convert an Anthropic streaming event to a chunk.
    fn to_chat_completion_chunk(&self, model: &str) -> Option<ChatCompletionChunk> {
        let mut delta = ChoiceDelta::default();
        let mut finish_reason = None;
        let mut usage = None;

        match self.event_type.as_str() {
            "content_block_start" => {
                if let Some(content_block) = &self.content_block {
                    match content_block.block_type.as_str() {
                        "text" => {
                            delta.content = Some(String::new());
                        }
                        "tool_use" => {
                            if let (Some(id), Some(name)) = (&content_block.id, &content_block.name)
                            {
                                delta.tool_calls = Some(vec![ToolCallDelta {
                                    index: Some(0),
                                    id: Some(id.clone()),
                                    tool_type: Some("function".to_string()),
                                    function: Some(FunctionDelta {
                                        name: Some(name.clone()),
                                        arguments: Some(String::new()),
                                    }),
                                }]);
                            }
                        }
                        "thinking" => {
                            delta.reasoning = Some(Reasoning::new(""));
                        }
                        _ => return None,
                    }
                }
            }
            "content_block_delta" => {
                if let Some(delta_data) = &self.delta {
                    match delta_data.delta_type.as_deref() {
                        Some("text_delta") => {
                            delta.content = delta_data.text.clone();
                        }
                        Some("input_json_delta") => {
                            if let Some(partial_json) = &delta_data.partial_json {
                                delta.tool_calls = Some(vec![ToolCallDelta {
                                    index: Some(0),
                                    id: None,
                                    tool_type: None,
                                    function: Some(FunctionDelta {
                                        name: None,
                                        arguments: Some(partial_json.clone()),
                                    }),
                                }]);
                            }
                        }
                        Some("thinking_delta") => {
                            if let Some(thinking) = &delta_data.thinking {
                                delta.reasoning = Some(Reasoning::new(thinking));
                            }
                        }
                        _ => return None,
                    }
                }
            }
            "content_block_stop" => {
                // Just an indicator that a block is done
                return None;
            }
            "message_stop" => {
                finish_reason = Some("stop".to_string());
            }
            "message_delta" => {
                if let Some(delta_data) = &self.delta {
                    if let Some(stop_reason) = &delta_data.stop_reason {
                        finish_reason = Some(match stop_reason.as_str() {
                            "end_turn" => "stop".to_string(),
                            "max_tokens" => "length".to_string(),
                            "tool_use" => "tool_calls".to_string(),
                            other => other.to_string(),
                        });
                    }
                }
                if let Some(usage_data) = &self.usage {
                    usage = Some(CompletionUsage {
                        prompt_tokens: usage_data.input_tokens.unwrap_or(0),
                        completion_tokens: usage_data.output_tokens.unwrap_or(0),
                        total_tokens: usage_data.input_tokens.unwrap_or(0)
                            + usage_data.output_tokens.unwrap_or(0),
                    });
                }
            }
            _ => return None,
        }

        Some(ChatCompletionChunk {
            id: format!("chatcmpl-{}", self.index.unwrap_or(0)),
            object: "chat.completion.chunk".to_string(),
            created: 0,
            model: model.to_string(),
            choices: vec![ChunkChoice {
                index: 0,
                delta,
                finish_reason,
                logprobs: None,
            }],
            usage,
            system_fingerprint: None,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct StreamContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StreamDelta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
    partial_json: Option<String>,
    thinking: Option<String>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StreamUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}
