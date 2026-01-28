use serde::Deserialize;
use serde_json::Value;

use crate::types::{
    ChatCompletion, ChatCompletionMessage, Choice, CompletionUsage, Function, Reasoning, Role,
    ToolCall,
};

#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    usage: Usage,
}

impl From<AnthropicResponse> for ChatCompletion {
    fn from(response: AnthropicResponse) -> ChatCompletion {
        let mut content_parts = Vec::new();
        let mut tool_calls = Vec::new();
        let mut reasoning_content: Option<String> = None;

        for block in response.content {
            match block.block_type.as_str() {
                "text" => {
                    if let Some(text) = block.text {
                        content_parts.push(text);
                    }
                }
                "tool_use" => {
                    if let (Some(id), Some(name), Some(input)) = (block.id, block.name, block.input)
                    {
                        tool_calls.push(ToolCall {
                            id,
                            tool_type: "function".to_string(),
                            function: Function {
                                name,
                                arguments: serde_json::to_string(&input).unwrap_or_default(),
                            },
                        });
                    }
                }
                "thinking" => {
                    if let Some(thinking) = block.thinking {
                        match &mut reasoning_content {
                            Some(existing) => existing.push_str(&thinking),
                            None => reasoning_content = Some(thinking),
                        }
                    }
                }
                _ => {
                    // TODO: internal unrecognized value error should be thrown here. or a warning.
                }
            }
        }

        // Map finish reason
        let finish_reason = match response.stop_reason.as_deref() {
            Some("end_turn") => Some("stop".to_string()),
            Some("max_tokens") => Some("length".to_string()),
            Some("tool_use") => Some("tool_calls".to_string()),
            other => other.map(|s| s.to_string()),
        };

        let message = ChatCompletionMessage {
            role: Role::Assistant,
            content: if content_parts.is_empty() {
                None
            } else {
                Some(content_parts.join(""))
            },
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
            reasoning: reasoning_content.map(|c| Reasoning::new(c)),
            refusal: None,
        };

        let usage = CompletionUsage {
            prompt_tokens: response.usage.input_tokens,
            completion_tokens: response.usage.output_tokens,
            total_tokens: response.usage.input_tokens + response.usage.output_tokens,
        };

        ChatCompletion {
            id: response.id,
            object: "chat.completion".to_string(),
            created: 0, // Anthropic doesn't provide this
            model: response.model,
            choices: vec![Choice {
                index: 0,
                message,
                finish_reason,
                logprobs: None,
            }],
            usage: Some(usage),
            system_fingerprint: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
    id: Option<String>,
    name: Option<String>,
    input: Option<Value>,
    thinking: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}
