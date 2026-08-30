//! Message and content-block types for the Messages API.
//!
//! Request-side [`MessageParam`] content is a string or an array of [`ContentBlockParam`] blocks
//! (text, image, thinking, tool use, tool result). The response-side [`Message`] carries a
//! [`ContentBlock`] array. Every block enum ends with an `Unknown` fallback variant so a
//! forward-compatible server never breaks deserialization.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{citation::TextCitation, usage::Usage};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    /// The Messages API schema lists `system` as a role value, but the guide places system
    /// instructions in the top-level `system` parameter; a `system`-role message may be rejected.
    System,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheTtl {
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheControlType {
    Ephemeral,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CacheControlEphemeral {
    pub r#type: CacheControlType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<CacheTtl>,
}

impl CacheControlEphemeral {
    /// Creates a prompt-cache breakpoint with the default `5m` TTL.
    pub fn new() -> Self {
        Self {
            r#type: CacheControlType::Ephemeral,
            ttl: None,
        }
    }
}

impl Default for CacheControlEphemeral {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlockParam>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ContentBlockParam {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControlEphemeral>,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<Vec<TextCitation>>,
    },
    #[serde(rename = "image")]
    Image {
        source: ImageSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControlEphemeral>,
    },
    #[serde(rename = "document")]
    Document {
        source: DocumentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<super::citation::CitationsConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControlEphemeral>,
    },
    #[serde(rename = "thinking")]
    Thinking { thinking: String, signature: String },
    #[serde(rename = "redacted_thinking")]
    RedactedThinking { data: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControlEphemeral>,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<MessageContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControlEphemeral>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ImageSource {
    Base64 {
        data: String,
        media_type: ImageMediaType,
    },
    Url {
        url: String,
    },
    File {
        file_id: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageMediaType {
    Jpeg,
    Png,
    Gif,
    Webp,
    /// Unrecognized media type, preserved verbatim for lossless round-tripping.
    Unknown(String),
}

impl Serialize for ImageMediaType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
            Self::Unknown(value) => value.as_str(),
        };
        serializer.serialize_str(value)
    }
}

impl<'de> Deserialize<'de> for ImageMediaType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "image/jpeg" => Self::Jpeg,
            "image/png" => Self::Png,
            "image/gif" => Self::Gif,
            "image/webp" => Self::Webp,
            _ => Self::Unknown(value.clone()),
        })
    }
}

/// Source of a document content block (PDF or plain text).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DocumentSource {
    Base64 {
        data: String,
        media_type: DocumentMediaType,
    },
    Text {
        data: String,
        media_type: PlainTextMediaType,
    },
    Content {
        content: MessageContent,
    },
    Url {
        url: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentMediaType {
    #[serde(rename = "application/pdf")]
    Pdf,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlainTextMediaType {
    #[serde(rename = "text/plain")]
    Plain,
    #[serde(other)]
    Unknown,
}

/// A single text block used by the top-level `system` parameter.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextBlockParam {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControlEphemeral>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<TextCitation>>,
    #[serde(rename = "type")]
    pub r#type: TextBlockType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TextBlockType {
    Text,
}

impl TextBlockParam {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            cache_control: None,
            citations: None,
            r#type: TextBlockType::Text,
        }
    }

    /// Marks this block as a prompt-cache breakpoint.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MessageParam {
    pub role: MessageRole,
    pub content: MessageContent,
}

impl MessageParam {
    /// Creates a `user` message with plain-text content.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Text(content.into()),
        }
    }

    /// Creates an `assistant` message with plain-text content.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(content.into()),
        }
    }

    /// Creates a `user` message carrying content blocks (e.g. tool results).
    pub fn user_blocks(blocks: Vec<ContentBlockParam>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Blocks(blocks),
        }
    }

    /// Creates an `assistant` message carrying content blocks (e.g. prior tool calls).
    pub fn assistant_blocks(blocks: Vec<ContentBlockParam>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Blocks(blocks),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ThinkingBlock {
    pub thinking: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<Vec<TextCitation>>,
    },
    #[serde(rename = "thinking")]
    Thinking(ThinkingBlock),
    #[serde(rename = "redacted_thinking")]
    RedactedThinking { data: String },
    #[serde(rename = "tool_use")]
    ToolUse(ToolUseBlock),
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Message,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    PauseTurn,
    Refusal,
    ModelContextWindowExceeded,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RefusalStopDetailsType {
    Refusal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RefusalStopDetails {
    #[serde(rename = "type")]
    pub r#type: RefusalStopDetailsType,
    /// Null when the refusal does not map to a named category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: MessageType,
    pub role: MessageRole,
    pub content: Vec<ContentBlock>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_details: Option<RefusalStopDetails>,
}

impl Message {
    /// Concatenates the text of all `text` content blocks in order.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Returns the model's tool invocations, in order.
    pub fn tool_calls(&self) -> impl Iterator<Item = &ToolUseBlock> {
        self.content.iter().filter_map(|block| match block {
            ContentBlock::ToolUse(call) => Some(call),
            _ => None,
        })
    }

    /// Returns the first thinking block, if any.
    pub fn thinking(&self) -> Option<&ThinkingBlock> {
        self.content.iter().find_map(|block| match block {
            ContentBlock::Thinking(thinking) => Some(thinking),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ImageMediaType;

    #[test]
    fn image_media_type_preserves_unknown_values() {
        assert_eq!(
            serde_json::to_value(ImageMediaType::Unknown("image/heic".to_owned())).unwrap(),
            serde_json::json!("image/heic")
        );
        let back: ImageMediaType = serde_json::from_value(serde_json::json!("image/heic")).unwrap();
        assert_eq!(back, ImageMediaType::Unknown("image/heic".to_owned()));

        assert_eq!(
            serde_json::to_value(ImageMediaType::Png).unwrap(),
            serde_json::json!("image/png")
        );
        let back: ImageMediaType = serde_json::from_value(serde_json::json!("image/png")).unwrap();
        assert_eq!(back, ImageMediaType::Png);
    }
}
