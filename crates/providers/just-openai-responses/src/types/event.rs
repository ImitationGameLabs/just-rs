//! Streaming SSE events (`POST /responses` with `stream: true`).
//!
//! Every event discriminates on the dotted `type` field. Core events are modeled; unknown event
//! types fall into [`StreamEvent::Unknown`] so a forward-compatible server never breaks the
//! stream. The `response.created`/`response.completed`/etc. events carry the full [`Response`].
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{item::OutputItem, message::OutputContentPart, response::Response};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OutputDeltaLogprob {
    pub token: String,
    pub logprob: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopDeltaLogprob>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TopDeltaLogprob {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprob: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "response.created")]
    ResponseCreated {
        response: Response,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.in_progress")]
    ResponseInProgress {
        response: Response,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.queued")]
    ResponseQueued {
        response: Response,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.completed")]
    ResponseCompleted {
        response: Response,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.failed")]
    ResponseFailed {
        response: Response,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.incomplete")]
    ResponseIncomplete {
        response: Response,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.output_item.added")]
    ResponseOutputItemAdded {
        item: OutputItem,
        output_index: u32,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.output_item.done")]
    ResponseOutputItemDone {
        item: OutputItem,
        output_index: u32,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.content_part.added")]
    ResponseContentPartAdded {
        item_id: String,
        output_index: u32,
        content_index: u32,
        part: OutputContentPart,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.content_part.done")]
    ResponseContentPartDone {
        item_id: String,
        output_index: u32,
        content_index: u32,
        part: OutputContentPart,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.output_text.delta")]
    ResponseOutputTextDelta {
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        logprobs: Option<Vec<OutputDeltaLogprob>>,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.output_text.done")]
    ResponseOutputTextDone {
        item_id: String,
        output_index: u32,
        content_index: u32,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        logprobs: Option<Vec<OutputDeltaLogprob>>,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.refusal.delta")]
    ResponseRefusalDelta {
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.refusal.done")]
    ResponseRefusalDone {
        item_id: String,
        output_index: u32,
        content_index: u32,
        refusal: String,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.function_call_arguments.delta")]
    ResponseFunctionCallArgumentsDelta {
        item_id: String,
        output_index: u32,
        delta: String,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.function_call_arguments.done")]
    ResponseFunctionCallArgumentsDone {
        item_id: String,
        output_index: u32,
        name: String,
        arguments: String,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.reasoning_text.delta")]
    ResponseReasoningTextDelta {
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.reasoning_text.done")]
    ResponseReasoningTextDone {
        item_id: String,
        output_index: u32,
        content_index: u32,
        text: String,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.reasoning_summary_text.delta")]
    ResponseReasoningSummaryTextDelta {
        item_id: String,
        output_index: u32,
        summary_index: u32,
        delta: String,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.reasoning_summary_text.done")]
    ResponseReasoningSummaryTextDone {
        item_id: String,
        output_index: u32,
        summary_index: u32,
        text: String,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.file_search_call.in_progress")]
    ResponseFileSearchCallInProgress {
        item_id: String,
        output_index: u32,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.file_search_call.searching")]
    ResponseFileSearchCallSearching {
        item_id: String,
        output_index: u32,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.file_search_call.completed")]
    ResponseFileSearchCallCompleted {
        item_id: String,
        output_index: u32,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.web_search_call.in_progress")]
    ResponseWebSearchCallInProgress {
        item_id: String,
        output_index: u32,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.web_search_call.searching")]
    ResponseWebSearchCallSearching {
        item_id: String,
        output_index: u32,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.web_search_call.completed")]
    ResponseWebSearchCallCompleted {
        item_id: String,
        output_index: u32,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.custom_tool_call_input.delta")]
    ResponseCustomToolCallInputDelta {
        item_id: String,
        output_index: u32,
        delta: String,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "response.custom_tool_call_input.done")]
    ResponseCustomToolCallInputDone {
        item_id: String,
        output_index: u32,
        input: String,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "error")]
    Error {
        code: Option<String>,
        message: String,
        param: Option<String>,
        sequence_number: Option<u64>,
    },
    #[serde(rename = "ping")]
    Ping { sequence_number: Option<u64> },
    #[serde(other)]
    Unknown,
}
