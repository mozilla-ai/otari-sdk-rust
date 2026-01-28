use crate::types::{
    ChatCompletionChunk, ChoiceDelta, ChunkChoice, CompletionUsage, FunctionDelta, Reasoning,
    ToolCallDelta,
};
use serde::Deserialize;

#[derive(Debug)]
pub struct AnthropicStream {
    event: AnthropicStreamEvent,
    model: String,
}

impl AnthropicStream {
    pub fn new(event: AnthropicStreamEvent, model: String) -> Self {
        Self { event, model }
    }
}

impl From<AnthropicStream> for Option<ChatCompletionChunk> {
    /// Convert an Anthropic streaming event to a chunk.
    fn from(value: AnthropicStream) -> Option<ChatCompletionChunk> {
        let event = value.event;
        let model = value.model.as_str();
        let mut delta = ChoiceDelta::default();
        let mut finish_reason = None;
        let mut usage = None;

        match event.event_type.as_str() {
            "content_block_start" => {
                if let Some(content_block) = &event.content_block {
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
                if let Some(delta_data) = &event.delta {
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
                if let Some(delta_data) = &event.delta {
                    if let Some(stop_reason) = &delta_data.stop_reason {
                        finish_reason = Some(match stop_reason.as_str() {
                            "end_turn" => "stop".to_string(),
                            "max_tokens" => "length".to_string(),
                            "tool_use" => "tool_calls".to_string(),
                            other => other.to_string(),
                        });
                    }
                }
                if let Some(usage_data) = &event.usage {
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
            id: format!("chatcmpl-{}", event.index.unwrap_or(0)),
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
pub struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    index: Option<u32>,
    content_block: Option<StreamContentBlock>,
    delta: Option<StreamDelta>,
    usage: Option<StreamUsage>,
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
