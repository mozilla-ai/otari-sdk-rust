//! Gateway response types and conversion to shared `ChatCompletion`.

use serde::Deserialize;
use serde_json::Value;

use crate::types::{
    ChatCompletion, ChatCompletionMessage, Choice, CompletionUsage, Function, Reasoning, Role,
    ToolCall,
};

/// Field names that providers use for reasoning/thinking content.
const REASONING_FIELD_NAMES: &[&str] = &["reasoning", "reasoning_content", "thinking", "think"];

/// Raw response from the gateway (OpenAI format).
#[derive(Debug, Deserialize)]
pub struct GatewayResponse {
    pub id: String,
    #[serde(default = "default_object")]
    pub object: String,
    #[serde(default)]
    pub created: i64,
    pub model: String,
    pub choices: Vec<GatewayChoice>,
    pub usage: Option<GatewayUsage>,
    pub system_fingerprint: Option<String>,
}

fn default_object() -> String {
    "chat.completion".to_string()
}

#[derive(Debug, Deserialize)]
pub struct GatewayChoice {
    pub index: u32,
    pub message: GatewayMessage,
    pub finish_reason: Option<String>,
    pub logprobs: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct GatewayMessage {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<GatewayToolCall>>,
    pub refusal: Option<String>,
    /// Catch additional fields (e.g. reasoning_content, thinking).
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct GatewayToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: GatewayFunction,
}

#[derive(Debug, Deserialize)]
pub struct GatewayFunction {
    pub name: String,
    pub arguments: String,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Deserialize)]
pub struct GatewayUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl From<GatewayResponse> for ChatCompletion {
    fn from(resp: GatewayResponse) -> Self {
        let choices = resp
            .choices
            .into_iter()
            .map(|c| {
                let reasoning = extract_reasoning(&c.message);
                let tool_calls = c.message.tool_calls.map(|tcs| {
                    tcs.into_iter()
                        .map(|tc| ToolCall {
                            id: tc.id,
                            tool_type: tc.tool_type,
                            function: Function {
                                name: tc.function.name,
                                arguments: tc.function.arguments,
                            },
                        })
                        .collect()
                });

                Choice {
                    index: c.index,
                    message: ChatCompletionMessage {
                        role: Role::Assistant,
                        content: c.message.content,
                        tool_calls,
                        reasoning,
                        refusal: c.message.refusal,
                    },
                    finish_reason: c.finish_reason,
                    logprobs: c.logprobs,
                }
            })
            .collect();

        let usage = resp.usage.map(|u| CompletionUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        ChatCompletion {
            id: resp.id,
            object: resp.object,
            created: resp.created,
            model: resp.model,
            choices,
            usage,
            system_fingerprint: resp.system_fingerprint,
        }
    }
}

/// Extract reasoning content from the message's extra fields.
///
/// Different upstream providers use different field names for
/// reasoning/thinking content. This checks all known field names
/// (mirroring the Python SDK's `REASONING_FIELD_NAMES` list).
fn extract_reasoning(msg: &GatewayMessage) -> Option<Reasoning> {
    for field in REASONING_FIELD_NAMES {
        if let Some(val) = msg.extra.get(*field) {
            // Could be a string directly, or {"content": "..."}.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_response_conversion() {
        let json = serde_json::json!({
            "id": "chatcmpl-abc",
            "object": "chat.completion",
            "created": 1_700_000_000_i64,
            "model": "openai:gpt-4o-mini",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "Hello!"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
        });
        let resp: GatewayResponse = serde_json::from_value(json).unwrap();
        let completion: ChatCompletion = resp.into();

        assert_eq!(completion.id, "chatcmpl-abc");
        assert_eq!(completion.content(), Some("Hello!"));
        assert_eq!(completion.finish_reason(), Some("stop"));
        assert_eq!(completion.usage.unwrap().total_tokens, 15);
    }

    #[test]
    fn test_reasoning_content_extraction() {
        let json = serde_json::json!({
            "id": "id",
            "model": "model",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "result",
                    "reasoning_content": "I thought about it"
                },
                "finish_reason": "stop"
            }]
        });
        let resp: GatewayResponse = serde_json::from_value(json).unwrap();
        let completion: ChatCompletion = resp.into();
        assert_eq!(completion.reasoning(), Some("I thought about it"));
    }

    #[test]
    fn test_thinking_field_extraction() {
        let json = serde_json::json!({
            "id": "id",
            "model": "model",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "result",
                    "thinking": "deep thoughts"
                },
                "finish_reason": "stop"
            }]
        });
        let resp: GatewayResponse = serde_json::from_value(json).unwrap();
        let completion: ChatCompletion = resp.into();
        assert_eq!(completion.reasoning(), Some("deep thoughts"));
    }

    #[test]
    fn test_tool_calls_conversion() {
        let json = serde_json::json!({
            "id": "id",
            "model": "model",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"city\":\"Paris\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        });
        let resp: GatewayResponse = serde_json::from_value(json).unwrap();
        let completion: ChatCompletion = resp.into();
        let tc = completion.tool_calls().unwrap();
        assert_eq!(tc.len(), 1);
        assert_eq!(tc[0].function.name, "get_weather");
    }
}
