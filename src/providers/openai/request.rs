use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionTool, ChatCompletionToolChoiceOption,
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
};

use crate::{
    error::{AnyLLMError, Result},
    types::CompletionParams,
};

impl TryFrom<CompletionParams> for CreateChatCompletionRequest {
    type Error = AnyLLMError;

    fn try_from(params: CompletionParams) -> Result<Self> {
        // Convert messages
        let messages: Vec<ChatCompletionRequestMessage> = params
            .messages
            .into_iter()
            .map(|i| i.try_into())
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
        if let Some(tools) = params.tools {
            let openai_tools: Vec<ChatCompletionTool> =
                tools.into_iter().map(|i| i.into()).collect();
            request_builder.tools(openai_tools);
        }

        // Add tool choice
        if let Some(tool_choice) = params.tool_choice {
            let choice: ChatCompletionToolChoiceOption = tool_choice.into();
            request_builder.tool_choice(choice);
        }

        // Add parallel tool calls
        if let Some(parallel) = params.parallel_tool_calls {
            request_builder.parallel_tool_calls(parallel);
        }

        request_builder
            .build()
            .map_err(|e| AnyLLMError::invalid_request("openai", e.to_string()))
    }
}
