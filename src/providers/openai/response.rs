use crate::types::{
    ChatCompletion, ChatCompletionMessage, Choice, CompletionUsage, Function, Reasoning, Role,
    ToolCall,
};
use async_openai::types::CreateChatCompletionResponse;

/// Convert OpenAI response to our ChatCompletion type.
impl From<CreateChatCompletionResponse> for ChatCompletion {
    fn from(response: CreateChatCompletionResponse) -> ChatCompletion {
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
}
