//! Input and output item wire types.
//!
//! `InputItem` and `OutputItem` are separate enums because their message-content shapes differ
//! (input messages may carry a plain string; output messages always carry content parts), but
//! they share the function-call and reasoning item sub-types.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::message::{
    InputMessage, ItemStatus, MessageContent, OutputMessage, ReasoningText, SummaryText,
};

/// A model-initiated function call (`type: "function_call"`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub call_id: String,
    pub name: String,
    /// A JSON-encoded string of the arguments to pass to the function.
    ///
    /// Defaults to an empty string when absent so that a streaming
    /// `response.output_item.added` event (which emits items before arguments are complete) does
    /// not fail deserialization.
    #[serde(default)]
    pub arguments: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

impl FunctionCall {
    pub fn new(
        name: impl Into<String>,
        call_id: impl Into<String>,
        arguments: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            call_id: call_id.into(),
            name: name.into(),
            arguments: arguments.into(),
            status: None,
            caller: None,
            namespace: None,
        }
    }
}

/// The result of a function tool call (`type: "function_call_output"`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FunctionCallOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub call_id: String,
    pub output: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
}

impl FunctionCallOutput {
    pub fn new(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            id: None,
            call_id: call_id.into(),
            output: MessageContent::Text(output.into()),
            name: None,
            namespace: None,
            caller: None,
            status: None,
        }
    }
}

/// A custom-tool call with free-form text input (`type: "custom_tool_call"`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CustomToolCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub call_id: String,
    pub input: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

/// The result of a custom-tool call (`type: "custom_tool_call_output"`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CustomToolCallOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub call_id: String,
    pub output: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
}

/// A reasoning item (`type: "reasoning"`). Content is encrypted by default.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReasoningItem {
    pub id: String,
    pub summary: Vec<SummaryText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ReasoningText>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum WebSearchStatus {
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "searching")]
    Searching,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(other)]
    Unknown,
}

/// A web-search tool call (`type: "web_search_call"`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebSearchCall {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<Value>,
    pub status: WebSearchStatus,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileSearchStatus {
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "searching")]
    Searching,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "incomplete")]
    Incomplete,
    #[serde(rename = "failed")]
    Failed,
    #[serde(other)]
    Unknown,
}

/// A file-search tool call (`type: "file_search_call"`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileSearchCall {
    pub id: String,
    pub queries: Vec<String>,
    pub status: FileSearchStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<Value>>,
}

/// Request-side input items (`POST /responses` body `input`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum InputItem {
    #[serde(rename = "message")]
    Message(InputMessage),
    #[serde(rename = "function_call")]
    FunctionCall(FunctionCall),
    #[serde(rename = "function_call_output")]
    FunctionCallOutput(FunctionCallOutput),
    #[serde(rename = "reasoning")]
    Reasoning(ReasoningItem),
    #[serde(rename = "custom_tool_call")]
    CustomToolCall(CustomToolCall),
    #[serde(rename = "custom_tool_call_output")]
    CustomToolCallOutput(CustomToolCallOutput),
    #[serde(rename = "web_search_call")]
    WebSearchCall(WebSearchCall),
    #[serde(rename = "file_search_call")]
    FileSearchCall(FileSearchCall),
    #[serde(other)]
    Unknown,
}

/// Response-side output items (`POST /responses` body `output`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum OutputItem {
    #[serde(rename = "message")]
    Message(OutputMessage),
    #[serde(rename = "function_call")]
    FunctionCall(FunctionCall),
    #[serde(rename = "function_call_output")]
    FunctionCallOutput(FunctionCallOutput),
    #[serde(rename = "reasoning")]
    Reasoning(ReasoningItem),
    #[serde(rename = "custom_tool_call")]
    CustomToolCall(CustomToolCall),
    #[serde(rename = "custom_tool_call_output")]
    CustomToolCallOutput(CustomToolCallOutput),
    #[serde(rename = "web_search_call")]
    WebSearchCall(WebSearchCall),
    #[serde(rename = "file_search_call")]
    FileSearchCall(FileSearchCall),
    #[serde(other)]
    Unknown,
}

impl OutputItem {
    /// The output text of this item, if it is a completed assistant message.
    pub fn output_text(&self) -> Option<&str> {
        match self {
            Self::Message(message) => message.content.iter().find_map(|part| match part {
                super::message::OutputContentPart::OutputText { text, .. } => Some(text.as_str()),
                _ => None,
            }),
            _ => None,
        }
    }
}
