use crate::{types::Tool, ToolChoice};
use async_openai::types::{
    ChatCompletionTool, ChatCompletionToolChoiceOption, ChatCompletionToolType, FunctionObject,
};

impl From<Tool> for ChatCompletionTool {
    fn from(tool: crate::types::Tool) -> ChatCompletionTool {
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
                name: tool.function.name,
                description: tool.function.description,
                parameters: tool.function.parameters,
                strict: None,
            },
        }
    }
}

impl From<ToolChoice> for ChatCompletionToolChoiceOption {
    fn from(choice: ToolChoice) -> ChatCompletionToolChoiceOption {
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
                        name: function.name,
                    },
                },
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_tool_choice_auto() {
        let choice = ToolChoice::auto();
        let result: ChatCompletionToolChoiceOption = choice.into();
        assert!(matches!(result, ChatCompletionToolChoiceOption::Auto));
    }

    #[test]
    fn test_convert_tool_choice_required() {
        let choice = ToolChoice::required();
        let result: ChatCompletionToolChoiceOption = choice.into();
        assert!(matches!(result, ChatCompletionToolChoiceOption::Required));
    }

    #[test]
    fn test_convert_tool_choice_specific() {
        let choice = ToolChoice::function("my_function");
        let result: ChatCompletionToolChoiceOption = choice.into();
        assert!(matches!(result, ChatCompletionToolChoiceOption::Named(_)));
    }
}
