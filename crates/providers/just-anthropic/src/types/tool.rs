//! Tool definitions and tool-choice configuration for the Messages API.
//!
//! Server tools (web search, code execution, etc.) are not modeled as first-class types yet;
//! pass their wire shape through the `serde_json::Value` [`Tool::input_schema`] or omit them.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use super::message::CacheControlEphemeral;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControlEphemeral>,
}

impl Tool {
    /// Creates a tool with the given name and input JSON schema.
    pub fn new(name: impl Into<String>, input_schema: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema,
            cache_control: None,
        }
    }

    /// Sets the tool's description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Marks this tool as a prompt-cache breakpoint.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ToolChoice {
    #[serde(rename = "auto")]
    Auto {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    #[serde(rename = "any")]
    Any {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    #[serde(rename = "tool")]
    Tool {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    #[serde(rename = "none")]
    None,
}
