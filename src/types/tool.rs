//! Tool and function calling types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A tool that can be called by the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    /// The type of the tool (always "function" for now).
    #[serde(rename = "type")]
    pub tool_type: String,

    /// The function definition.
    pub function: ToolFunction,
}

impl Tool {
    /// Create a new function tool.
    pub fn function(name: impl Into<String>, description: impl Into<String>) -> ToolBuilder {
        ToolBuilder {
            name: name.into(),
            description: description.into(),
            parameters: None,
        }
    }
}

/// Builder for creating tools.
pub struct ToolBuilder {
    name: String,
    description: String,
    parameters: Option<Value>,
}

impl ToolBuilder {
    /// Set the parameters schema for this tool.
    pub fn parameters(mut self, parameters: Value) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Build the tool.
    pub fn build(self) -> Tool {
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: self.name,
                description: Some(self.description),
                parameters: self.parameters,
            },
        }
    }
}

/// A function that can be called.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolFunction {
    /// The name of the function.
    pub name: String,

    /// A description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The parameters the function accepts (JSON Schema).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

/// A tool call made by the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// The unique ID of this tool call.
    pub id: String,

    /// The type of the tool (always "function" for now).
    #[serde(rename = "type")]
    pub tool_type: String,

    /// The function call details.
    pub function: Function,
}

impl ToolCall {
    /// Create a new tool call.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            tool_type: "function".to_string(),
            function: Function {
                name: name.into(),
                arguments: arguments.into(),
            },
        }
    }

    /// Parse the function arguments as JSON.
    pub fn parse_arguments<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.function.arguments)
    }
}

/// A function call with name and arguments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Function {
    /// The name of the function to call.
    pub name: String,

    /// The arguments to pass to the function (JSON string).
    pub arguments: String,
}

/// A partial tool call delta for streaming.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolCallDelta {
    /// The index of this tool call in the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,

    /// The unique ID of this tool call (only in first chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The type of the tool (only in first chunk).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,

    /// The function call details (partial).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionDelta>,
}

/// A partial function call delta for streaming.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FunctionDelta {
    /// The name of the function (only in first chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Partial arguments string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// How the model should choose which tool to use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Let the model decide - "none", "auto", or "required".
    Mode(String),

    /// Force a specific function.
    Function {
        #[serde(rename = "type")]
        choice_type: String,
        function: ToolChoiceFunction,
    },
}

impl ToolChoice {
    /// Don't use any tools.
    pub fn none() -> Self {
        ToolChoice::Mode("none".to_string())
    }

    /// Let the model decide whether to use tools.
    pub fn auto() -> Self {
        ToolChoice::Mode("auto".to_string())
    }

    /// Require the model to use a tool.
    pub fn required() -> Self {
        ToolChoice::Mode("required".to_string())
    }

    /// Force the model to use a specific function.
    pub fn function(name: impl Into<String>) -> Self {
        ToolChoice::Function {
            choice_type: "function".to_string(),
            function: ToolChoiceFunction { name: name.into() },
        }
    }

    /// Check if this is a mode string.
    pub fn as_mode(&self) -> Option<&str> {
        match self {
            ToolChoice::Mode(mode) => Some(mode),
            _ => None,
        }
    }

    /// Check if this is a specific function choice.
    pub fn as_function(&self) -> Option<&str> {
        match self {
            ToolChoice::Function { function, .. } => Some(&function.name),
            _ => None,
        }
    }
}

/// A specific function to force the model to use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolChoiceFunction {
    /// The name of the function to call.
    pub name: String,
}
