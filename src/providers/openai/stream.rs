use async_openai::types::CreateChatCompletionStreamResponse;

use crate::types::{
    ChatCompletionChunk, ChoiceDelta, ChunkChoice, CompletionUsage, FunctionDelta, Role,
    ToolCallDelta,
};

impl From<CreateChatCompletionStreamResponse> for ChatCompletionChunk {
    fn from(value: CreateChatCompletionStreamResponse) -> Self {
        let choices = value
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

        let usage = value.usage.map(|u| CompletionUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        ChatCompletionChunk {
            id: value.id,
            object: "chat.completion.chunk".to_string(),
            created: value.created as i64,
            model: value.model,
            choices,
            usage,
            system_fingerprint: value.system_fingerprint,
        }
    }
}
