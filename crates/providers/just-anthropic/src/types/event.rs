//! Streaming SSE events (`POST /v1/messages` with `stream: true`).
//!
//! Every event discriminates on its `type` field. Core events are modeled; unknown event types
//! fall into [`StreamEvent::Unknown`] so a forward-compatible server never breaks the stream.
//! The `message_delta.usage` carries cumulative token counts that are only partially populated
//! (typically `output_tokens` alone), hence the separate [`StreamUsage`] type with optional fields.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    citation::TextCitation,
    message::{ContentBlock, RefusalStopDetails, StopReason},
    usage::OutputTokensDetails,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockDelta {
    TextDelta {
        text: String,
    },
    InputJsonDelta {
        partial_json: String,
    },
    ThinkingDelta {
        thinking: String,
    },
    SignatureDelta {
        signature: String,
    },
    CitationsDelta {
        citation: TextCitation,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MessageDeltaBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_details: Option<RefusalStopDetails>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StreamUsage {
    pub output_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<OutputTokensDetails>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiError {
    #[serde(rename = "type")]
    pub r#type: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: super::message::Message },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: u64,
        content_block: ContentBlock,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: u64,
        delta: ContentBlockDelta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u64 },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDeltaBody,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<StreamUsage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: ApiError },
    #[serde(other)]
    Unknown,
}
