//! Request DTOs for the Messages and token-counting endpoints.
//!
//! `max_tokens` is required by the Messages API, so it is part of [`CreateMessageRequest::new`]
//! rather than a `with_*` setter. Optional parameters follow the `with_*` builder convention used
//! across the workspace's provider crates.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    message::{CacheControlEphemeral, MessageParam, TextBlockParam},
    output::OutputConfig,
    thinking::ThinkingConfig,
    tool::{Tool, ToolChoice},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SystemParam {
    Text(String),
    Blocks(Vec<TextBlockParam>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    Auto,
    StandardOnly,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateMessageRequest {
    pub model: String,
    pub messages: Vec<MessageParam>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControlEphemeral>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_config: Option<OutputConfig>,
}

impl CreateMessageRequest {
    /// Creates a request with the required parameters (`model`, `messages`, `max_tokens`).
    pub fn new(model: impl Into<String>, messages: Vec<MessageParam>, max_tokens: u32) -> Self {
        Self {
            model: model.into(),
            messages,
            max_tokens,
            system: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: None,
            thinking: None,
            tool_choice: None,
            tools: None,
            metadata: None,
            service_tier: None,
            cache_control: None,
            output_config: None,
        }
    }

    /// Sets the top-level `system` prompt (string or text blocks).
    pub fn with_system(mut self, system: SystemParam) -> Self {
        self.system = Some(system);
        self
    }

    /// Sets the sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the nucleus-sampling cutoff.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Restricts sampling to the top-K tokens.
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Sets custom stop sequences.
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Configures extended thinking.
    pub fn with_thinking(mut self, thinking: ThinkingConfig) -> Self {
        self.thinking = Some(thinking);
        self
    }

    /// Sets how the model should use the provided tools.
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the available tool definitions.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets request metadata (e.g. a user id for abuse detection).
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the service tier (priority vs standard capacity).
    pub fn with_service_tier(mut self, service_tier: ServiceTier) -> Self {
        self.service_tier = Some(service_tier);
        self
    }

    /// Marks the last cacheable block as a prompt-cache breakpoint.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Constrains the response format and/or reasoning effort (structured outputs).
    pub fn with_output_config(mut self, output_config: OutputConfig) -> Self {
        self.output_config = Some(output_config);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CountTokensRequest {
    pub model: String,
    pub messages: Vec<MessageParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

impl CountTokensRequest {
    /// Creates a token-counting request with the required parameters (`model`, `messages`).
    pub fn new(model: impl Into<String>, messages: Vec<MessageParam>) -> Self {
        Self {
            model: model.into(),
            messages,
            system: None,
            thinking: None,
            tool_choice: None,
            tools: None,
        }
    }

    /// Sets the top-level `system` prompt to count tokens for.
    pub fn with_system(mut self, system: SystemParam) -> Self {
        self.system = Some(system);
        self
    }

    /// Sets the thinking configuration to count tokens for.
    pub fn with_thinking(mut self, thinking: ThinkingConfig) -> Self {
        self.thinking = Some(thinking);
        self
    }

    /// Sets the tool-choice configuration to count tokens for.
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the tool definitions to count tokens for.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CountTokensResponse {
    pub input_tokens: u64,
}
