//! Protocol-neutral streaming generation events and stream wrapper.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use super::shared::{FinishReason, Usage};

/// Incremental tool-call payload emitted during streaming.
///
/// `arguments`, when present, is an **incremental fragment** to be appended to the running value
/// for the tool call identified by [`index`](Self::index) (all three protocols stream partial
/// fragments, so consumers accumulate: `arguments += delta`). The stable `index` identifies the
/// in-flight tool call across deltas, so parallel calls can be accumulated unambiguously. `id` and
/// `name` typically arrive on the first delta for the call, when the protocol exposes them.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolCallDelta {
    /// Stable index identifying the in-flight tool call (chat `tool_calls[].index`, Anthropic
    /// content-block `index`, Responses `output_index`). `None` when the protocol does not carry it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Incremental JSON fragment; append to the accumulated arguments for this tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// One streaming generation event.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GenerationEvent {
    Text { delta: String },
    Reasoning { delta: String },
    ToolCall { delta: ToolCallDelta },
    Usage { usage: Usage },
    End { finish_reason: Option<FinishReason> },
}

impl GenerationEvent {
    /// Returns the finish reason for an [`End`](Self::End) event.
    pub fn finish_reason(&self) -> Option<&FinishReason> {
        match self {
            Self::End { finish_reason } => finish_reason.as_ref(),
            _ => None,
        }
    }
}
