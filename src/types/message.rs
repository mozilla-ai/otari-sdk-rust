//! Message types for chat completions.

use serde::{Deserialize, Serialize};

use super::tool::ToolCall;

/// The role of a message in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message - sets the behavior of the assistant.
    System,
    /// User message - input from the user.
    User,
    /// Assistant message - response from the model.
    Assistant,
    /// Tool message - result from a tool call.
    Tool,
}

impl Role {
    /// Returns the string representation of the role.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        }
    }
}

/// Content of a message - can be simple text or multiple parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Content {
    /// Simple text content.
    Text(String),
    /// Multiple content parts (text, images, etc.).
    Parts(Vec<ContentPart>),
}

impl Content {
    /// Create text content.
    pub fn text(text: impl Into<String>) -> Self {
        Content::Text(text.into())
    }

    /// Create content with multiple parts.
    pub fn parts(parts: Vec<ContentPart>) -> Self {
        Content::Parts(parts)
    }

    /// Get the text content if this is simple text.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text(text) => Some(text),
            Content::Parts(_) => None,
        }
    }

    /// Extract all text from the content.
    pub fn extract_text(&self) -> String {
        match self {
            Content::Text(text) => text.clone(),
            Content::Parts(parts) => parts
                .iter()
                .filter_map(|p| match p {
                    ContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

impl From<String> for Content {
    fn from(text: String) -> Self {
        Content::Text(text)
    }
}

impl From<&str> for Content {
    fn from(text: &str) -> Self {
        Content::Text(text.to_string())
    }
}

/// A part of multi-part content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content part.
    #[serde(rename = "text")]
    Text { text: String },

    /// Image URL content part.
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

impl ContentPart {
    /// Create a text content part.
    pub fn text(text: impl Into<String>) -> Self {
        ContentPart::Text { text: text.into() }
    }

    /// Create an image URL content part.
    pub fn image_url(url: impl Into<String>) -> Self {
        ContentPart::ImageUrl {
            image_url: ImageUrl::new(url),
        }
    }

    /// Create an image from base64 data.
    pub fn image_base64(data: impl Into<String>, media_type: impl Into<String>) -> Self {
        ContentPart::ImageUrl {
            image_url: ImageUrl::base64(data, media_type),
        }
    }
}

/// An image URL reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageUrl {
    /// The URL of the image (can be a data: URL for base64).
    pub url: String,

    /// The detail level for the image (low, high, auto).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ImageUrl {
    /// Create a new ImageUrl from a URL string.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            detail: None,
        }
    }

    /// Create an ImageUrl from base64-encoded data.
    pub fn base64(data: impl Into<String>, media_type: impl Into<String>) -> Self {
        Self {
            url: format!("data:{};base64,{}", media_type.into(), data.into()),
            detail: None,
        }
    }

    /// Set the detail level.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Check if this is a base64 data URL.
    pub fn is_base64(&self) -> bool {
        self.url.starts_with("data:")
    }

    /// Parse a base64 data URL into (media_type, data).
    pub fn parse_base64(&self) -> Option<(&str, &str)> {
        if !self.is_base64() {
            return None;
        }

        // Format: data:<media_type>;base64,<data>
        let after_data = self.url.strip_prefix("data:")?;
        let semi_idx = after_data.find(';')?;
        let media_type = &after_data[..semi_idx];
        let data = after_data.strip_prefix(&format!("{};base64,", media_type))?;
        Some((media_type, data))
    }
}

/// A message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// The role of the message author.
    pub role: Role,

    /// The content of the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,

    /// Optional name for the message author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Tool calls made by the assistant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// The ID of the tool call this message is responding to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    /// Create a new message with the given role and content.
    pub fn new(role: Role, content: impl Into<Content>) -> Self {
        Self {
            role,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, Content::text(content))
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, Content::text(content))
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, Content::text(content))
    }

    /// Create an assistant message with tool calls.
    pub fn assistant_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: Role::Assistant,
            content: None,
            name: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }

    /// Create a tool result message.
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: Some(Content::text(content)),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }

    /// Create a user message with multiple content parts.
    pub fn user_with_parts(parts: Vec<ContentPart>) -> Self {
        Self::new(Role::User, Content::parts(parts))
    }

    /// Set the name of the message author.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Get the text content of the message, if any.
    pub fn text_content(&self) -> Option<String> {
        self.content.as_ref().map(|c| c.extract_text())
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            role: Role::User,
            content: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }
}
