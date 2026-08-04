//! Protocol-neutral generation request.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    Message,
    shared::{ReasoningEffort, ResponseFormat, StopSequence, ToolChoice, ToolDefinition},
};

/// Normalized generation request understood by LLM client backends.
///
/// The field set is the protocol-neutral common core: shared across chat completions, the
/// Responses API, and Anthropic Messages. Provider-specific parameters (e.g. Responses `store` or
/// Anthropic `thinking`) are intentionally absent; callers needing them use the provider crate
/// directly.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GenerationRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<ToolChoice>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub stop: Option<StopSequence>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub response_format: Option<ResponseFormat>,
    pub logprobs: Option<bool>,
    pub top_logprobs: Option<u8>,
    pub reasoning_effort: Option<ReasoningEffort>,
}

impl GenerationRequest {
    /// Creates a minimal request with provider-neutral defaults.
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            tools: None,
            tool_choice: None,
            temperature: None,
            top_p: None,
            top_k: None,
            max_tokens: None,
            stream: None,
            stop: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
            logprobs: None,
            top_logprobs: None,
            reasoning_effort: None,
        }
    }

    /// Sets the configured tools for the request.
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the tool-choice behavior for the request.
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the sampling temperature for the request.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the nucleus-sampling cutoff for the request.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Restricts sampling to the top-K tokens.
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Sets the maximum generated-token count for the request.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets custom stop sequences for the request.
    pub fn with_stop_sequences(mut self, stop: StopSequence) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Sets the frequency penalty for the request.
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    /// Sets the presence penalty for the request.
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    /// Sets the response-format hint for the request.
    pub fn with_response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Requests logprobs for the generated tokens.
    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Sets how many top logprobs to return per token (requires `logprobs = true`).
    pub fn with_top_logprobs(mut self, top_logprobs: u8) -> Self {
        self.top_logprobs = Some(top_logprobs);
        self
    }

    /// Sets the requested reasoning effort.
    pub fn with_reasoning_effort(mut self, reasoning_effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(reasoning_effort);
        self
    }

    /// Inserts a system message at the end of the leading system-message block.
    ///
    /// Repeated calls preserve the order they are invoked while keeping system messages grouped
    /// at the front of the request.
    pub fn with_system_prompt(self, content: impl Into<String>) -> Self {
        self.prepend_system_message(content)
    }

    /// Inserts a system message at the end of the leading system-message block.
    pub fn prepend_system_message(mut self, content: impl Into<String>) -> Self {
        let insert_index = self
            .messages
            .iter()
            .take_while(|message| message.role() == "system")
            .count();
        self.messages.insert(insert_index, Message::system(content));
        self
    }

    /// Prepends a message to the request.
    pub fn prepend_message(mut self, message: Message) -> Self {
        self.messages.insert(0, message);
        self
    }
}
