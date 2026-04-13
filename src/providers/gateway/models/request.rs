//! Gateway request types and conversion from `CompletionParams`.

use serde::Serialize;
use serde_json::{json, Value};

use crate::error::{AnyLLMError, Result};
use crate::providers::Gateway;
use crate::types::{CompletionParams, Content, ContentPart, Message};

/// Request body for the gateway's `/v1/chat/completions` endpoint.
///
/// The gateway speaks the OpenAI wire format, so this maps `CompletionParams`
/// to an OpenAI-compatible JSON body.
#[derive(Debug, Serialize)]
pub struct GatewayRequest {
    pub model: String,
    pub messages: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<Value>,
}

impl GatewayRequest {
    /// Enable streaming and request usage in the final chunk.
    pub fn stream(mut self) -> Self {
        self.stream = Some(true);
        self.stream_options = Some(json!({"include_usage": true}));
        self
    }
}

impl TryFrom<CompletionParams> for GatewayRequest {
    type Error = AnyLLMError;

    fn try_from(params: CompletionParams) -> Result<Self> {
        let messages = params
            .messages
            .iter()
            .map(convert_message)
            .collect::<Result<Vec<_>>>()?;

        let tools = params
            .tools
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| {
                AnyLLMError::invalid_request::<Gateway>(format!("failed to serialize tools: {e}"))
            })?;
        let tool_choice = params
            .tool_choice
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| {
                AnyLLMError::invalid_request::<Gateway>(format!(
                    "failed to serialize tool_choice: {e}"
                ))
            })?;
        let logit_bias = params
            .logit_bias
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| {
                AnyLLMError::invalid_request::<Gateway>(format!(
                    "failed to serialize logit_bias: {e}"
                ))
            })?;
        let stop = params.stop.as_ref().map(|s| json!(s.to_vec()));
        let reasoning_effort = params
            .reasoning_effort
            .map(|r| format!("{r:?}").to_lowercase());

        Ok(Self {
            model: params.model_id,
            messages,
            temperature: params.temperature,
            top_p: params.top_p,
            max_tokens: params.max_tokens,
            n: params.n,
            stop,
            presence_penalty: params.presence_penalty,
            frequency_penalty: params.frequency_penalty,
            seed: params.seed,
            user: params.user,
            tools,
            tool_choice,
            parallel_tool_calls: params.parallel_tool_calls,
            logprobs: params.logprobs,
            top_logprobs: params.top_logprobs,
            logit_bias,
            response_format: params.response_format,
            reasoning_effort,
            stream: None,
            stream_options: None,
        })
    }
}

/// Convert an any-llm `Message` to an OpenAI-format JSON value.
fn convert_message(msg: &Message) -> Result<Value> {
    let role = msg.role.as_str();

    let mut obj = json!({"role": role});

    // Content
    match &msg.content {
        Some(Content::Text(text)) => {
            obj["content"] = json!(text);
        }
        Some(Content::Parts(parts)) => {
            let converted: Vec<Value> = parts
                .iter()
                .map(|part| match part {
                    ContentPart::Text { text } => json!({"type": "text", "text": text}),
                    ContentPart::ImageUrl { image_url } => {
                        let mut image_url_obj = json!({"url": image_url.url});
                        if let Some(detail) = &image_url.detail {
                            image_url_obj["detail"] = json!(detail);
                        }
                        json!({
                            "type": "image_url",
                            "image_url": image_url_obj
                        })
                    }
                })
                .collect();
            obj["content"] = json!(converted);
        }
        None => {
            obj["content"] = Value::Null;
        }
    }

    // Tool calls (assistant messages)
    if let Some(tool_calls) = &msg.tool_calls {
        obj["tool_calls"] = serde_json::to_value(tool_calls).map_err(|e| {
            AnyLLMError::provider_error::<Gateway>(format!("Failed to serialize tool calls: {e}"))
        })?;
    }

    // Tool call ID (tool messages)
    if let Some(id) = &msg.tool_call_id {
        obj["tool_call_id"] = json!(id);
    }

    // Name
    if let Some(name) = &msg.name {
        obj["name"] = json!(name);
    }

    Ok(obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CompletionParams, Message, ReasoningEffort, ToolChoice};

    #[test]
    fn test_basic_conversion() {
        let params = CompletionParams::new("openai:gpt-4o-mini", vec![Message::user("hello")]);
        let req: GatewayRequest = params.try_into().unwrap();
        assert_eq!(req.model, "openai:gpt-4o-mini");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0]["role"], "user");
        assert_eq!(req.messages[0]["content"], "hello");
        assert!(req.stream.is_none());
    }

    #[test]
    fn test_system_and_assistant_messages() {
        let params = CompletionParams::new(
            "model",
            vec![
                Message::system("You are helpful."),
                Message::user("Hi"),
                Message::assistant("Hello!"),
            ],
        );
        let req: GatewayRequest = params.try_into().unwrap();
        assert_eq!(req.messages.len(), 3);
        assert_eq!(req.messages[0]["role"], "system");
        assert_eq!(req.messages[1]["role"], "user");
        assert_eq!(req.messages[2]["role"], "assistant");
    }

    #[test]
    fn test_optional_params_forwarded() {
        let params = CompletionParams::new("model", vec![Message::user("hi")])
            .with_temperature(0.5)
            .with_max_tokens(100)
            .with_reasoning_effort(ReasoningEffort::High)
            .with_tool_choice(ToolChoice::auto());

        let req: GatewayRequest = params.try_into().unwrap();
        assert_eq!(req.temperature, Some(0.5));
        assert_eq!(req.max_tokens, Some(100));
        assert_eq!(req.reasoning_effort.as_deref(), Some("high"));
        assert!(req.tool_choice.is_some());
    }

    #[test]
    fn test_stream_method() {
        let params = CompletionParams::new("model", vec![Message::user("hi")]);
        let req: GatewayRequest = params.try_into().unwrap();
        let req = req.stream();
        assert_eq!(req.stream, Some(true));
        assert!(req.stream_options.is_some());
    }
}
