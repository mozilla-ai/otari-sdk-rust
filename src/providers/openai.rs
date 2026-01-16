//! OpenAI provider implementation.

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartText,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContent,
        ChatCompletionRequestUserMessageContentPart, ChatCompletionTool,
        ChatCompletionToolChoiceOption, ChatCompletionToolType, CreateChatCompletionRequestArgs,
        FunctionCall, FunctionObject, ImageDetail, ImageUrl as OpenAIImageUrl,
    },
    Client,
};
use async_trait::async_trait;
use futures::StreamExt;

use crate::error::{AnyLLMError, Result};
use crate::provider::{CompletionStream, Provider, ProviderConfig};
use crate::types::{
    ChatCompletion, ChatCompletionChunk, ChatCompletionMessage, Choice, ChoiceDelta, ChunkChoice,
    CompletionParams, CompletionUsage, Content, ContentPart, Function, FunctionDelta, Message,
    Reasoning, Role, ToolCall, ToolCallDelta, ToolChoice,
};

/// OpenAI provider using the async-openai SDK.
pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| AnyLLMError::MissingApiKey {
                provider: "openai".to_string(),
                env_var: "OPENAI_API_KEY".to_string(),
            })?;

        let mut openai_config = OpenAIConfig::new().with_api_key(api_key);

        if let Some(api_base) = config.api_base {
            openai_config = openai_config.with_api_base(api_base);
        }

        Ok(Self {
            client: Client::with_config(openai_config),
        })
    }

    /// Convert our Message type to OpenAI's ChatCompletionRequestMessage.
    fn convert_message(message: &Message) -> Result<ChatCompletionRequestMessage> {
        match message.role {
            Role::System => {
                let content = message
                    .content
                    .as_ref()
                    .map(|c| c.extract_text())
                    .unwrap_or_default();
                Ok(ChatCompletionRequestSystemMessageArgs::default()
                    .content(content)
                    .build()
                    .map_err(|e| AnyLLMError::invalid_request("openai", e.to_string()))?
                    .into())
            }
            Role::User => {
                let content = match &message.content {
                    Some(Content::Text(text)) => {
                        ChatCompletionRequestUserMessageContent::Text(text.clone())
                    }
                    Some(Content::Parts(parts)) => {
                        let openai_parts: Vec<ChatCompletionRequestUserMessageContentPart> = parts
                            .iter()
                            .map(|part| match part {
                                ContentPart::Text { text } => {
                                    ChatCompletionRequestUserMessageContentPart::Text(
                                        ChatCompletionRequestMessageContentPartText {
                                            text: text.clone(),
                                        },
                                    )
                                }
                                ContentPart::ImageUrl { image_url } => {
                                    let detail = image_url
                                        .detail
                                        .as_ref()
                                        .map(|d| match d.as_str() {
                                            "low" => ImageDetail::Low,
                                            "high" => ImageDetail::High,
                                            _ => ImageDetail::Auto,
                                        })
                                        .unwrap_or(ImageDetail::Auto);

                                    ChatCompletionRequestUserMessageContentPart::ImageUrl(
                                        async_openai::types::ChatCompletionRequestMessageContentPartImage {
                                            image_url: OpenAIImageUrl {
                                                url: image_url.url.clone(),
                                                detail: Some(detail),
                                            },
                                        },
                                    )
                                }
                            })
                            .collect();
                        ChatCompletionRequestUserMessageContent::Array(openai_parts)
                    }
                    None => ChatCompletionRequestUserMessageContent::Text(String::new()),
                };
                Ok(ChatCompletionRequestUserMessageArgs::default()
                    .content(content)
                    .build()
                    .map_err(|e| AnyLLMError::invalid_request("openai", e.to_string()))?
                    .into())
            }
            Role::Assistant => {
                let mut builder = ChatCompletionRequestAssistantMessageArgs::default();

                if let Some(content) = &message.content {
                    builder.content(content.extract_text());
                }

                if let Some(tool_calls) = &message.tool_calls {
                    let openai_calls: Vec<ChatCompletionMessageToolCall> = tool_calls
                        .iter()
                        .map(|tc| ChatCompletionMessageToolCall {
                            id: tc.id.clone(),
                            r#type: ChatCompletionToolType::Function,
                            function: FunctionCall {
                                name: tc.function.name.clone(),
                                arguments: tc.function.arguments.clone(),
                            },
                        })
                        .collect();
                    builder.tool_calls(openai_calls);
                }

                Ok(builder
                    .build()
                    .map_err(|e| AnyLLMError::invalid_request("openai", e.to_string()))?
                    .into())
            }
            Role::Tool => {
                let content = message
                    .content
                    .as_ref()
                    .map(|c| c.extract_text())
                    .unwrap_or_default();
                let tool_call_id = message.tool_call_id.clone().unwrap_or_default();
                Ok(ChatCompletionRequestToolMessageArgs::default()
                    .content(content)
                    .tool_call_id(tool_call_id)
                    .build()
                    .map_err(|e| AnyLLMError::invalid_request("openai", e.to_string()))?
                    .into())
            }
        }
    }

    /// Convert our Tool type to OpenAI's ChatCompletionTool.
    fn convert_tool(tool: &crate::types::Tool) -> ChatCompletionTool {
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
                name: tool.function.name.clone(),
                description: tool.function.description.clone(),
                parameters: tool.function.parameters.clone(),
                strict: None,
            },
        }
    }

    /// Convert our ToolChoice to OpenAI's ChatCompletionToolChoiceOption.
    fn convert_tool_choice(choice: &ToolChoice) -> ChatCompletionToolChoiceOption {
        match choice {
            ToolChoice::Mode(mode) => match mode.as_str() {
                "none" => ChatCompletionToolChoiceOption::None,
                "auto" => ChatCompletionToolChoiceOption::Auto,
                "required" => ChatCompletionToolChoiceOption::Required,
                _ => ChatCompletionToolChoiceOption::Auto,
            },
            ToolChoice::Function { function, .. } => ChatCompletionToolChoiceOption::Named(
                async_openai::types::ChatCompletionNamedToolChoice {
                    r#type: ChatCompletionToolType::Function,
                    function: async_openai::types::FunctionName {
                        name: function.name.clone(),
                    },
                },
            ),
        }
    }

    /// Convert OpenAI response to our ChatCompletion type.
    fn convert_response(
        response: async_openai::types::CreateChatCompletionResponse,
    ) -> ChatCompletion {
        let choices = response
            .choices
            .into_iter()
            .map(|choice| {
                let tool_calls = choice.message.tool_calls.clone().map(|tcs| {
                    tcs.into_iter()
                        .map(|tc| ToolCall {
                            id: tc.id,
                            tool_type: "function".to_string(),
                            function: Function {
                                name: tc.function.name,
                                arguments: tc.function.arguments,
                            },
                        })
                        .collect()
                });

                // Handle reasoning from various field names (currently not supported by SDK)
                let reasoning: Option<Reasoning> = None;

                Choice {
                    index: choice.index,
                    message: ChatCompletionMessage {
                        role: Role::Assistant,
                        content: choice.message.content,
                        tool_calls,
                        reasoning,
                        refusal: choice.message.refusal,
                    },
                    finish_reason: choice
                        .finish_reason
                        .map(|r| format!("{:?}", r).to_lowercase()),
                    logprobs: None,
                }
            })
            .collect();

        let usage = response.usage.map(|u| CompletionUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        ChatCompletion {
            id: response.id,
            object: "chat.completion".to_string(),
            created: response.created as i64,
            model: response.model,
            choices,
            usage,
            system_fingerprint: response.system_fingerprint,
        }
    }

    /// Convert OpenAI streaming chunk to our ChatCompletionChunk type.
    fn convert_chunk(
        chunk: async_openai::types::CreateChatCompletionStreamResponse,
    ) -> ChatCompletionChunk {
        let choices = chunk
            .choices
            .into_iter()
            .map(|choice| {
                let tool_calls = choice.delta.tool_calls.map(|tcs| {
                    tcs.into_iter()
                        .map(|tc| ToolCallDelta {
                            index: Some(tc.index as u32),
                            id: tc.id,
                            tool_type: tc.r#type.map(|_| "function".to_string()),
                            function: tc.function.map(|f| FunctionDelta {
                                name: f.name,
                                arguments: f.arguments,
                            }),
                        })
                        .collect()
                });

                let role = choice.delta.role.map(|_| Role::Assistant);

                ChunkChoice {
                    index: choice.index,
                    delta: ChoiceDelta {
                        role,
                        content: choice.delta.content,
                        tool_calls,
                        reasoning: None, // TODO: Add when SDK supports it
                        refusal: choice.delta.refusal,
                    },
                    finish_reason: choice
                        .finish_reason
                        .map(|r| format!("{:?}", r).to_lowercase()),
                    logprobs: None,
                }
            })
            .collect();

        let usage = chunk.usage.map(|u| CompletionUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        ChatCompletionChunk {
            id: chunk.id,
            object: "chat.completion.chunk".to_string(),
            created: chunk.created as i64,
            model: chunk.model,
            choices,
            usage,
            system_fingerprint: chunk.system_fingerprint,
        }
    }

    /// Convert OpenAI error to our error type.
    fn convert_error(error: async_openai::error::OpenAIError) -> AnyLLMError {
        let message = error.to_string();

        // Try to detect error type from message
        if message.contains("rate limit") || message.contains("429") {
            AnyLLMError::rate_limit("openai", message)
        } else if message.contains("authentication")
            || message.contains("401")
            || message.contains("invalid api key")
        {
            AnyLLMError::authentication("openai", message)
        } else if message.contains("not found") || message.contains("404") {
            AnyLLMError::ModelNotFound {
                provider: "openai".to_string(),
                model: "unknown".to_string(),
            }
        } else if message.contains("400") || message.contains("bad request") {
            AnyLLMError::invalid_request("openai", message)
        } else {
            AnyLLMError::provider_error("openai", message)
        }
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn supports_images(&self) -> bool {
        true
    }

    fn supports_reasoning(&self) -> bool {
        true // o1, o3 models support reasoning
    }

    async fn completion(&self, params: CompletionParams) -> Result<ChatCompletion> {
        // Convert messages
        let messages: Vec<ChatCompletionRequestMessage> = params
            .messages
            .iter()
            .map(Self::convert_message)
            .collect::<Result<Vec<_>>>()?;

        // Build request
        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(&params.model_id).messages(messages);

        // Add optional parameters
        if let Some(temperature) = params.temperature {
            request_builder.temperature(temperature);
        }
        if let Some(top_p) = params.top_p {
            request_builder.top_p(top_p);
        }
        if let Some(max_tokens) = params.max_tokens {
            request_builder.max_tokens(max_tokens);
        }
        #[allow(clippy::cast_possible_truncation)]
        if let Some(n) = params.n {
            request_builder.n(n as u8);
        }
        if let Some(presence_penalty) = params.presence_penalty {
            request_builder.presence_penalty(presence_penalty);
        }
        if let Some(frequency_penalty) = params.frequency_penalty {
            request_builder.frequency_penalty(frequency_penalty);
        }
        if let Some(seed) = params.seed {
            request_builder.seed(seed);
        }
        if let Some(user) = &params.user {
            request_builder.user(user);
        }
        if let Some(stop) = &params.stop {
            request_builder.stop(stop.to_vec());
        }

        // Add tools
        if let Some(tools) = &params.tools {
            let openai_tools: Vec<ChatCompletionTool> =
                tools.iter().map(Self::convert_tool).collect();
            request_builder.tools(openai_tools);
        }

        // Add tool choice
        if let Some(tool_choice) = &params.tool_choice {
            request_builder.tool_choice(Self::convert_tool_choice(tool_choice));
        }

        // Add parallel tool calls
        if let Some(parallel) = params.parallel_tool_calls {
            request_builder.parallel_tool_calls(parallel);
        }

        let request = request_builder
            .build()
            .map_err(|e| AnyLLMError::invalid_request("openai", e.to_string()))?;

        // Make the API call
        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(Self::convert_error)?;

        Ok(Self::convert_response(response))
    }

    async fn completion_stream(&self, params: CompletionParams) -> Result<CompletionStream> {
        // Convert messages
        let messages: Vec<ChatCompletionRequestMessage> = params
            .messages
            .iter()
            .map(Self::convert_message)
            .collect::<Result<Vec<_>>>()?;

        // Build request
        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(&params.model_id)
            .messages(messages)
            .stream(true);

        // Add optional parameters (same as completion)
        if let Some(temperature) = params.temperature {
            request_builder.temperature(temperature);
        }
        if let Some(top_p) = params.top_p {
            request_builder.top_p(top_p);
        }
        if let Some(max_tokens) = params.max_tokens {
            request_builder.max_tokens(max_tokens);
        }
        if let Some(presence_penalty) = params.presence_penalty {
            request_builder.presence_penalty(presence_penalty);
        }
        if let Some(frequency_penalty) = params.frequency_penalty {
            request_builder.frequency_penalty(frequency_penalty);
        }
        if let Some(seed) = params.seed {
            request_builder.seed(seed);
        }
        if let Some(user) = &params.user {
            request_builder.user(user);
        }
        if let Some(stop) = &params.stop {
            request_builder.stop(stop.to_vec());
        }

        // Add tools
        if let Some(tools) = &params.tools {
            let openai_tools: Vec<ChatCompletionTool> =
                tools.iter().map(Self::convert_tool).collect();
            request_builder.tools(openai_tools);
        }

        // Add tool choice
        if let Some(tool_choice) = &params.tool_choice {
            request_builder.tool_choice(Self::convert_tool_choice(tool_choice));
        }

        // Add parallel tool calls
        if let Some(parallel) = params.parallel_tool_calls {
            request_builder.parallel_tool_calls(parallel);
        }

        let request = request_builder
            .build()
            .map_err(|e| AnyLLMError::invalid_request("openai", e.to_string()))?;

        // Create the stream
        let stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(Self::convert_error)?;

        // Map the stream to our types
        let mapped_stream = stream.map(|result| match result {
            Ok(chunk) => Ok(Self::convert_chunk(chunk)),
            Err(e) => Err(Self::convert_error(e)),
        });

        Ok(Box::pin(mapped_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_message_system() {
        let msg = Message::system("You are a helpful assistant.");
        let result = OpenAIProvider::convert_message(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_message_user() {
        let msg = Message::user("Hello!");
        let result = OpenAIProvider::convert_message(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_message_user_with_image() {
        let parts = vec![
            ContentPart::text("What's in this image?"),
            ContentPart::image_url("https://example.com/image.png"),
        ];
        let msg = Message::user_with_parts(parts);
        let result = OpenAIProvider::convert_message(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_tool_choice_auto() {
        let choice = ToolChoice::auto();
        let result = OpenAIProvider::convert_tool_choice(&choice);
        assert!(matches!(result, ChatCompletionToolChoiceOption::Auto));
    }

    #[test]
    fn test_convert_tool_choice_required() {
        let choice = ToolChoice::required();
        let result = OpenAIProvider::convert_tool_choice(&choice);
        assert!(matches!(result, ChatCompletionToolChoiceOption::Required));
    }

    #[test]
    fn test_convert_tool_choice_specific() {
        let choice = ToolChoice::function("my_function");
        let result = OpenAIProvider::convert_tool_choice(&choice);
        assert!(matches!(result, ChatCompletionToolChoiceOption::Named(_)));
    }
}
