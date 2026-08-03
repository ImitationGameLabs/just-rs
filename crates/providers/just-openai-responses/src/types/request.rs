//! Request payloads for the Responses API endpoints.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    item::InputItem,
    shared::{Metadata, ReasoningConfig, ResponseIncludable, TextConfig, ToolChoice, Truncation},
    tool::ResponseTool,
};

/// `POST /responses` `input`: either a plain string or an array of input items.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ResponseInput {
    Text(String),
    Items(Vec<InputItem>),
}

impl ResponseInput {
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    pub fn items(items: Vec<InputItem>) -> Self {
        Self::Items(items)
    }
}

impl From<&str> for ResponseInput {
    fn from(value: &str) -> Self {
        Self::text(value)
    }
}

impl From<String> for ResponseInput {
    fn from(value: String) -> Self {
        Self::text(value)
    }
}

impl From<Vec<InputItem>> for ResponseInput {
    fn from(value: Vec<InputItem>) -> Self {
        Self::items(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
}

/// Wire DTO for `POST /responses`.
///
/// Parameters that are too new or unstable to model yet (`moderation`, `context_management`,
/// `prompt_cache_options`, `safety_identifier`, `prompt`) are intentionally omitted; this is a
/// serialization-only struct, so omitted parameters are simply not sent.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateResponseRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<ResponseInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ResponseTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<ResponseIncludable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<Truncation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
}

impl CreateResponseRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            input: None,
            instructions: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            conversation: None,
            store: None,
            stream: None,
            stream_options: None,
            temperature: None,
            top_p: None,
            top_logprobs: None,
            max_output_tokens: None,
            max_tool_calls: None,
            text: None,
            reasoning: None,
            include: None,
            truncation: None,
            metadata: None,
            user: None,
            background: None,
            service_tier: None,
        }
    }

    /// Sets the input, accepting a string, `&str`, `String`, or `Vec<InputItem>`.
    pub fn with_input(mut self, input: impl Into<ResponseInput>) -> Self {
        self.input = Some(input.into());
        self
    }

    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    pub fn with_tools(mut self, tools: Vec<ResponseTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    pub fn with_previous_response_id(mut self, id: impl Into<String>) -> Self {
        self.previous_response_id = Some(id.into());
        self
    }

    /// Sets the `include` list, requesting optional extra data (e.g.
    /// `reasoning.encrypted_content`).
    ///
    /// Setting `include` explicitly opts out of the automatic injection performed by
    /// [`with_reasoning`](Self::with_reasoning): call this first to take full control.
    pub fn with_include(mut self, include: Vec<ResponseIncludable>) -> Self {
        self.include = Some(include);
        self
    }

    /// Configures reasoning for the request.
    ///
    /// When the caller has not explicitly set [`include`](Self::with_include), this automatically
    /// adds `reasoning.encrypted_content` so the encrypted fidelity carrier is returned; set
    /// `include` explicitly first to opt out.
    pub fn with_reasoning(mut self, reasoning: ReasoningConfig) -> Self {
        if self.include.is_none() {
            self.include = Some(vec![ResponseIncludable::ReasoningEncryptedContent]);
        }
        self.reasoning = Some(reasoning);
        self
    }
}

/// Wire DTO for `POST /responses/compact`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CompactRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<ResponseInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
}

impl CompactRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            input: None,
            instructions: None,
            previous_response_id: None,
        }
    }
}

/// Wire DTO returned by `POST /responses/compact`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CompactedResponse {
    pub id: String,
    pub created_at: u64,
    pub object: String,
    pub output: Vec<super::item::OutputItem>,
    pub usage: super::response::ResponseUsage,
}
