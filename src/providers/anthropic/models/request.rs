//! Anthropic (Claude) provider implementation.

use serde::Serialize;
use serde_json::{json, Value};

use crate::error::{AnyLLMError, Result};
use crate::providers::Anthropic;
use crate::types::{
    CompletionParams, Content, ContentPart, Message, ReasoningEffort, Role, ToolChoice,
};

use super::super::DEFAULT_MAX_TOKENS;

#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    model: String,
    messages: Vec<Value>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

impl TryFrom<CompletionParams> for AnthropicRequest {
    type Error = AnyLLMError;

    fn try_from(params: CompletionParams) -> std::result::Result<Self, Self::Error> {
        if params.response_format.is_some() {
            return Err(AnyLLMError::unsupported_parameter::<Anthropic>(
                "response_format",
                "See https://docs.anthropic.com/en/docs/test-and-evaluate/strengthen-guardrails/increase-consistency",
            ));
        }

        // Extract system messages
        let (system, filtered_messages) = extract_system_messages(&params.messages);

        // Convert messages
        let messages = convert_messages(filtered_messages)?;

        // Build request body
        let max_tokens = params.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        let tools = params.tools.map(|tools| convert_tools(&tools));

        let tool_choice = match (params.tool_choice, params.parallel_tool_calls) {
            (Some(tc), ptc) => Some(convert_tool_choice(&tc, ptc)),
            (None, Some(ptc)) => Some(json!({
                "type": "auto",
                "disable_parallel_tool_use": !ptc
            })),
            (None, None) => None,
        };

        let reasoning = match params.reasoning_effort {
            Some(ReasoningEffort::None) => Some(json!({"type": "disabled"})),
            Some(ReasoningEffort::Auto) => {
                // Don't set thinking - let Anthropic use default
                None
            }
            Some(effort) => reasoning_effort_to_budget(effort).map(|budget| {
                json!({
                    "type": "enabled",
                    "budget_tokens": budget
                })
            }),
            None => None,
        };

        Ok(Self {
            model: params.model_id.clone(),
            messages,
            max_tokens,
            system,
            temperature: params.temperature,
            top_p: params.top_p,
            stop_sequences: params.stop.map(|i| json!(i.to_vec())),
            tools,
            tool_choice,
            reasoning,
            stream: None,
        })
    }
}

impl AnthropicRequest {
    pub fn stream(mut self) -> Self {
        self.stream = Some(true);

        self
    }
}

fn extract_system_messages(messages: &[Message]) -> (Option<String>, Vec<&Message>) {
    let mut system_parts = Vec::new();
    let mut filtered = Vec::new();

    for msg in messages {
        if msg.role == Role::System {
            if let Some(content) = &msg.content {
                system_parts.push(content.extract_text());
            }
        } else {
            filtered.push(msg);
        }
    }

    let system = if system_parts.is_empty() {
        None
    } else {
        Some(system_parts.join("\n"))
    };

    (system, filtered)
}

fn convert_messages(messages: Vec<&Message>) -> Result<Vec<Value>> {
    let mut result = Vec::new();

    for msg in messages {
        match msg.role {
            Role::System => {
                // System messages should be extracted separately
                continue;
            }
            Role::User => {
                let content = convert_content(&msg.content)?;
                result.push(json!({
                    "role": "user",
                    "content": content
                }));
            }
            Role::Assistant => {
                if let Some(tool_calls) = &msg.tool_calls {
                    // Convert tool calls to tool_use blocks
                    let tool_use_blocks: Vec<Value> = tool_calls
                        .iter()
                        .map(|tc| {
                            let input: Value =
                                serde_json::from_str(&tc.function.arguments).unwrap_or(json!({}));
                            json!({
                                "type": "tool_use",
                                "id": tc.id,
                                "name": tc.function.name,
                                "input": input
                            })
                        })
                        .collect();
                    result.push(json!({
                        "role": "assistant",
                        "content": tool_use_blocks
                    }));
                } else {
                    let content = msg
                        .content
                        .as_ref()
                        .map(|c| c.extract_text())
                        .unwrap_or_default();
                    result.push(json!({
                        "role": "assistant",
                        "content": content
                    }));
                }
            }
            Role::Tool => {
                // Convert tool response to user message with tool_result block
                let tool_result = json!({
                    "type": "tool_result",
                    "tool_use_id": msg.tool_call_id.clone().unwrap_or_default(),
                    "content": msg.content.as_ref().map(|c| c.extract_text()).unwrap_or_default()
                });

                // Check if we can merge with previous user message containing tool_results
                if let Some(last) = result.last_mut() {
                    if last.get("role") == Some(&json!("user")) {
                        if let Some(content) = last.get_mut("content") {
                            if let Some(arr) = content.as_array_mut() {
                                if arr.first().and_then(|v| v.get("type"))
                                    == Some(&json!("tool_result"))
                                {
                                    arr.push(tool_result);
                                    continue;
                                }
                            }
                        }
                    }
                }

                // Create new user message with tool_result
                result.push(json!({
                    "role": "user",
                    "content": [tool_result]
                }));
            }
        }
    }

    Ok(result)
}

fn convert_content(content: &Option<Content>) -> Result<Value> {
    match content {
        None => Ok(json!("")),
        Some(Content::Text(text)) => Ok(json!(text)),
        Some(Content::Parts(parts)) => {
            let converted: Vec<Value> = parts
                .iter()
                .map(|part| match part {
                    ContentPart::Text { text } => json!({
                        "type": "text",
                        "text": text
                    }),
                    ContentPart::ImageUrl { image_url } => {
                        if let Some((media_type, data)) = image_url.parse_base64() {
                            json!({
                                "type": "image",
                                "source": {
                                    "type": "base64",
                                    "media_type": media_type,
                                    "data": data
                                }
                            })
                        } else {
                            json!({
                                "type": "image",
                                "source": {
                                    "type": "url",
                                    "url": image_url.url
                                }
                            })
                        }
                    }
                })
                .collect();
            Ok(json!(converted))
        }
    }
}

/// Convert tools to Anthropic format.
fn convert_tools(tools: &[crate::types::Tool]) -> Vec<AnthropicTool> {
    tools
        .iter()
        .filter(|t| t.tool_type == "function")
        .map(|t| {
            let params = t.function.parameters.clone().unwrap_or(json!({}));
            AnthropicTool(json!({
                "name": t.function.name,
                "description": t.function.description.clone().unwrap_or_default(),
                "input_schema": {
                    "type": "object",
                    "properties": params.get("properties").cloned().unwrap_or(json!({})),
                    "required": params.get("required").cloned().unwrap_or(json!([]))
                }
            }))
        })
        .collect()
}

/// Convert tool choice to Anthropic format.
fn convert_tool_choice(choice: &ToolChoice, parallel_tool_calls: Option<bool>) -> Value {
    let disable_parallel = !parallel_tool_calls.unwrap_or(true);

    match choice {
        ToolChoice::Mode(mode) => {
            let tool_type = match mode.as_str() {
                "none" => "none",
                "auto" => "auto",
                "required" => "any", // Anthropic maps "required" to "any"
                _ => "auto",
            };
            json!({
                "type": tool_type,
                "disable_parallel_tool_use": disable_parallel
            })
        }
        ToolChoice::Function { function, .. } => {
            json!({
                "type": "tool",
                "name": function.name,
                "disable_parallel_tool_use": disable_parallel
            })
        }
    }
}

/// Reasoning effort to thinking budget mapping.
fn reasoning_effort_to_budget(effort: ReasoningEffort) -> Option<u32> {
    match effort {
        ReasoningEffort::None => None,
        ReasoningEffort::Minimal => Some(1024),
        ReasoningEffort::Low => Some(2048),
        ReasoningEffort::Medium => Some(8192),
        ReasoningEffort::High => Some(24576),
        ReasoningEffort::Auto => None, // Let Anthropic decide
    }
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct AnthropicTool(Value);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_system_messages() {
        let messages = vec![
            Message::system("You are helpful."),
            Message::system("Be concise."),
            Message::user("Hello"),
        ];

        let (system, filtered) = extract_system_messages(&messages);

        assert_eq!(system, Some("You are helpful.\nBe concise.".to_string()));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].role, Role::User);
    }

    #[test]
    fn test_extract_system_messages_none() {
        let messages = vec![Message::user("Hello")];

        let (system, filtered) = extract_system_messages(&messages);

        assert!(system.is_none());
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_convert_tool_choice_required() {
        let choice = ToolChoice::required();
        let result = convert_tool_choice(&choice, None);

        assert_eq!(result["type"], "any");
        assert_eq!(result["disable_parallel_tool_use"], false);
    }

    #[test]
    fn test_convert_tool_choice_specific() {
        let choice = ToolChoice::function("my_function");
        let result = convert_tool_choice(&choice, Some(false));

        assert_eq!(result["type"], "tool");
        assert_eq!(result["name"], "my_function");
        assert_eq!(result["disable_parallel_tool_use"], true);
    }

    #[test]
    fn test_reasoning_effort_to_budget() {
        assert_eq!(reasoning_effort_to_budget(ReasoningEffort::None), None);
        assert_eq!(
            reasoning_effort_to_budget(ReasoningEffort::Minimal),
            Some(1024)
        );
        assert_eq!(reasoning_effort_to_budget(ReasoningEffort::Low), Some(2048));
        assert_eq!(
            reasoning_effort_to_budget(ReasoningEffort::Medium),
            Some(8192)
        );
        assert_eq!(
            reasoning_effort_to_budget(ReasoningEffort::High),
            Some(24576)
        );
        assert_eq!(reasoning_effort_to_budget(ReasoningEffort::Auto), None);
    }

    #[test]
    fn test_convert_content_text() {
        let content = Some(Content::text("Hello"));
        let result = convert_content(&content).unwrap();
        assert_eq!(result, json!("Hello"));
    }

    #[test]
    fn test_convert_content_with_image() {
        let content = Some(Content::parts(vec![
            ContentPart::text("What's this?"),
            ContentPart::image_base64("abc123", "image/png"),
        ]));
        let result = convert_content(&content).unwrap();

        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["type"], "text");
        assert_eq!(arr[1]["type"], "image");
        assert_eq!(arr[1]["source"]["type"], "base64");
        assert_eq!(arr[1]["source"]["media_type"], "image/png");
    }
}
