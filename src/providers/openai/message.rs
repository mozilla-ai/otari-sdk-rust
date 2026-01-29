use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartText,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs,
    ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestUserMessageContentPart, ChatCompletionToolType, FunctionCall, ImageDetail,
    ImageUrl as OpenAIImageUrl,
};

use crate::error::AnyLLMError;
use crate::providers::OpenAI;
use crate::types::{Content, ContentPart, Message, Role};

impl TryFrom<Message> for ChatCompletionRequestMessage {
    type Error = AnyLLMError;

    fn try_from(message: Message) -> Result<ChatCompletionRequestMessage, AnyLLMError> {
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
                    .map_err(|e| AnyLLMError::invalid_request::<OpenAI>(e.to_string()))?
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
                    .map_err(|e| AnyLLMError::invalid_request::<OpenAI>(e.to_string()))?
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
                    .map_err(|e| AnyLLMError::invalid_request::<OpenAI>(e.to_string()))?
                    .into())
            }
            Role::Tool => {
                let content = message
                    .content
                    .as_ref()
                    .map(|c| c.extract_text())
                    .unwrap_or_default();
                let tool_call_id = message.tool_call_id.unwrap_or_default();
                Ok(ChatCompletionRequestToolMessageArgs::default()
                    .content(content)
                    .tool_call_id(tool_call_id)
                    .build()
                    .map_err(|e| AnyLLMError::invalid_request::<OpenAI>(e.to_string()))?
                    .into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[test]
    fn test_convert_message_system() {
        let msg = Message::system("You are a helpful assistant.");
        let result: Result<ChatCompletionRequestMessage> = msg.try_into();
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_message_user() {
        let msg = Message::user("Hello!");
        let result: Result<ChatCompletionRequestMessage> = msg.try_into();
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_message_user_with_image() {
        let parts = vec![
            ContentPart::text("What's in this image?"),
            ContentPart::image_url("https://example.com/image.png"),
        ];
        let msg = Message::user_with_parts(parts);
        let result: Result<ChatCompletionRequestMessage> = msg.try_into();
        assert!(result.is_ok());
    }
}
