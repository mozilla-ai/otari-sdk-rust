//! Anthropic (Claude) provider implementation.

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::{AnyLLMError, Result};
use crate::provider::{CompletionStream, Provider, ProviderConfig};
use crate::types::{
    ChatCompletion, ChatCompletionChunk, ChatCompletionMessage, Choice, ChoiceDelta, ChunkChoice,
    CompletionParams, CompletionUsage, Content, ContentPart, Function, FunctionDelta, Message,
    Reasoning, ReasoningEffort, Role, ToolCall, ToolCallDelta, ToolChoice,
};

/// Default max tokens for Anthropic (required parameter).
const DEFAULT_MAX_TOKENS: u32 = 8192;

/// API version for Anthropic.
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Default API base URL.
const DEFAULT_API_BASE: &str = "https://api.anthropic.com";

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

/// Anthropic provider using the Messages API.
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    api_base: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| AnyLLMError::MissingApiKey {
                provider: "anthropic".to_string(),
                env_var: "ANTHROPIC_API_KEY".to_string(),
            })?;

        let api_base = config
            .api_base
            .unwrap_or_else(|| DEFAULT_API_BASE.to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
            api_base,
        })
    }

    /// Extract system messages and return (system_text, filtered_messages).
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

    /// Convert messages to Anthropic format.
    fn convert_messages(messages: Vec<&Message>) -> Result<Vec<Value>> {
        let mut result = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => {
                    // System messages should be extracted separately
                    continue;
                }
                Role::User => {
                    let content = Self::convert_content(&msg.content)?;
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
                                let input: Value = serde_json::from_str(&tc.function.arguments)
                                    .unwrap_or(json!({}));
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

    /// Convert content to Anthropic format.
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
    fn convert_tools(tools: &[crate::types::Tool]) -> Vec<Value> {
        tools
            .iter()
            .filter(|t| t.tool_type == "function")
            .map(|t| {
                let params = t.function.parameters.clone().unwrap_or(json!({}));
                json!({
                    "name": t.function.name,
                    "description": t.function.description.clone().unwrap_or_default(),
                    "input_schema": {
                        "type": "object",
                        "properties": params.get("properties").cloned().unwrap_or(json!({})),
                        "required": params.get("required").cloned().unwrap_or(json!([]))
                    }
                })
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

    /// Convert Anthropic response to our ChatCompletion type.
    fn convert_response(response: AnthropicResponse) -> ChatCompletion {
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
                _ => {}
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

    /// Convert an Anthropic streaming event to a chunk.
    fn convert_stream_event(
        event: &AnthropicStreamEvent,
        model: &str,
    ) -> Option<ChatCompletionChunk> {
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

    /// Convert HTTP error to our error type.
    fn convert_error(status: u16, body: &str) -> AnyLLMError {
        match status {
            429 => AnyLLMError::rate_limit("anthropic", body),
            401 => AnyLLMError::authentication("anthropic", body),
            400 => AnyLLMError::invalid_request("anthropic", body),
            404 => AnyLLMError::ModelNotFound {
                provider: "anthropic".to_string(),
                model: "unknown".to_string(),
            },
            _ => AnyLLMError::provider_error("anthropic", format!("Status {}: {}", status, body)),
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn supports_images(&self) -> bool {
        true
    }

    fn supports_reasoning(&self) -> bool {
        true
    }

    async fn completion(&self, params: CompletionParams) -> Result<ChatCompletion> {
        // Check for unsupported parameters
        if params.response_format.is_some() {
            return Err(AnyLLMError::unsupported_parameter(
                "anthropic",
                "response_format",
                "See https://docs.anthropic.com/en/docs/test-and-evaluate/strengthen-guardrails/increase-consistency",
            ));
        }

        // Extract system messages
        let (system, filtered_messages) = Self::extract_system_messages(&params.messages);

        // Convert messages
        let messages = Self::convert_messages(filtered_messages)?;

        // Build request body
        let max_tokens = params.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        let mut body = json!({
            "model": params.model_id,
            "messages": messages,
            "max_tokens": max_tokens,
        });

        // Add system message
        if let Some(system_text) = system {
            body["system"] = json!(system_text);
        }

        // Add optional parameters
        if let Some(temperature) = params.temperature {
            body["temperature"] = json!(temperature);
        }
        if let Some(top_p) = params.top_p {
            body["top_p"] = json!(top_p);
        }
        if let Some(stop) = &params.stop {
            body["stop_sequences"] = json!(stop.to_vec());
        }

        // Add tools
        if let Some(tools) = &params.tools {
            let anthropic_tools = Self::convert_tools(tools);
            if !anthropic_tools.is_empty() {
                body["tools"] = json!(anthropic_tools);
            }
        }

        // Add tool choice
        if let Some(tool_choice) = &params.tool_choice {
            body["tool_choice"] =
                Self::convert_tool_choice(tool_choice, params.parallel_tool_calls);
        } else if params.parallel_tool_calls.is_some() {
            // If parallel_tool_calls is set but no tool_choice, still need to configure it
            body["tool_choice"] = json!({
                "type": "auto",
                "disable_parallel_tool_use": !params.parallel_tool_calls.unwrap_or(true)
            });
        }

        // Add reasoning/thinking
        if let Some(effort) = params.reasoning_effort {
            match effort {
                ReasoningEffort::None => {
                    body["thinking"] = json!({"type": "disabled"});
                }
                ReasoningEffort::Auto => {
                    // Don't set thinking - let Anthropic use default
                }
                _ => {
                    if let Some(budget) = reasoning_effort_to_budget(effort) {
                        body["thinking"] = json!({
                            "type": "enabled",
                            "budget_tokens": budget
                        });
                    }
                }
            }
        }

        // Make the API call
        let response = self
            .client
            .post(format!("{}/v1/messages", self.api_base))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status().as_u16();
        if status != 200 {
            let body = response.text().await.unwrap_or_default();
            return Err(Self::convert_error(status, &body));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;
        Ok(Self::convert_response(anthropic_response))
    }

    async fn completion_stream(&self, params: CompletionParams) -> Result<CompletionStream> {
        // Check for unsupported parameters
        if params.response_format.is_some() {
            return Err(AnyLLMError::unsupported_parameter(
                "anthropic",
                "response_format",
                "See https://docs.anthropic.com/en/docs/test-and-evaluate/strengthen-guardrails/increase-consistency",
            ));
        }

        // Extract system messages
        let (system, filtered_messages) = Self::extract_system_messages(&params.messages);

        // Convert messages
        let messages = Self::convert_messages(filtered_messages)?;

        // Build request body (same as completion but with stream: true)
        let max_tokens = params.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        let mut body = json!({
            "model": params.model_id,
            "messages": messages,
            "max_tokens": max_tokens,
            "stream": true,
        });

        // Add system message
        if let Some(system_text) = system {
            body["system"] = json!(system_text);
        }

        // Add optional parameters
        if let Some(temperature) = params.temperature {
            body["temperature"] = json!(temperature);
        }
        if let Some(top_p) = params.top_p {
            body["top_p"] = json!(top_p);
        }
        if let Some(stop) = &params.stop {
            body["stop_sequences"] = json!(stop.to_vec());
        }

        // Add tools
        if let Some(tools) = &params.tools {
            let anthropic_tools = Self::convert_tools(tools);
            if !anthropic_tools.is_empty() {
                body["tools"] = json!(anthropic_tools);
            }
        }

        // Add tool choice
        if let Some(tool_choice) = &params.tool_choice {
            body["tool_choice"] =
                Self::convert_tool_choice(tool_choice, params.parallel_tool_calls);
        }

        // Add reasoning/thinking
        if let Some(effort) = params.reasoning_effort {
            match effort {
                ReasoningEffort::None => {
                    body["thinking"] = json!({"type": "disabled"});
                }
                ReasoningEffort::Auto => {}
                _ => {
                    if let Some(budget) = reasoning_effort_to_budget(effort) {
                        body["thinking"] = json!({
                            "type": "enabled",
                            "budget_tokens": budget
                        });
                    }
                }
            }
        }

        let model = params.model_id.clone();

        // Create request
        let request = self
            .client
            .post(format!("{}/v1/messages", self.api_base))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body);

        // Create SSE stream
        let es = EventSource::new(request).map_err(|e| AnyLLMError::Streaming {
            provider: "anthropic".to_string(),
            message: e.to_string(),
        })?;

        // Use scan to track when we should stop, then filter_map to convert events
        // This properly terminates the stream on StreamEnded or errors
        let stream = es
            .map(move |event| {
                let model = model.clone();
                match event {
                    Ok(Event::Message(msg)) => {
                        // Parse the SSE data
                        if let Ok(stream_event) =
                            serde_json::from_str::<AnthropicStreamEvent>(&msg.data)
                        {
                            Self::convert_stream_event(&stream_event, &model)
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
                        provider: "anthropic".to_string(),
                        message: e.to_string(),
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

// Anthropic API response types

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    usage: Usage,
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

// Streaming event types

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    index: Option<u32>,
    content_block: Option<StreamContentBlock>,
    delta: Option<StreamDelta>,
    usage: Option<StreamUsage>,
}

#[derive(Debug, Deserialize)]
struct StreamContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
    partial_json: Option<String>,
    thinking: Option<String>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

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

        let (system, filtered) = AnthropicProvider::extract_system_messages(&messages);

        assert_eq!(system, Some("You are helpful.\nBe concise.".to_string()));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].role, Role::User);
    }

    #[test]
    fn test_extract_system_messages_none() {
        let messages = vec![Message::user("Hello")];

        let (system, filtered) = AnthropicProvider::extract_system_messages(&messages);

        assert!(system.is_none());
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_convert_tool_choice_required() {
        let choice = ToolChoice::required();
        let result = AnthropicProvider::convert_tool_choice(&choice, None);

        assert_eq!(result["type"], "any");
        assert_eq!(result["disable_parallel_tool_use"], false);
    }

    #[test]
    fn test_convert_tool_choice_specific() {
        let choice = ToolChoice::function("my_function");
        let result = AnthropicProvider::convert_tool_choice(&choice, Some(false));

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
        let result = AnthropicProvider::convert_content(&content).unwrap();
        assert_eq!(result, json!("Hello"));
    }

    #[test]
    fn test_convert_content_with_image() {
        let content = Some(Content::parts(vec![
            ContentPart::text("What's this?"),
            ContentPart::image_base64("abc123", "image/png"),
        ]));
        let result = AnthropicProvider::convert_content(&content).unwrap();

        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["type"], "text");
        assert_eq!(arr[1]["type"], "image");
        assert_eq!(arr[1]["source"]["type"], "base64");
        assert_eq!(arr[1]["source"]["media_type"], "image/png");
    }
}
