//! Citation types attached to text content blocks.
//!
//! Citations locate the source Claude referenced (web-search results or uploaded documents) with
//! a type-specific span. All variants share `cited_text`/`document_index`/`document_title`;
//! response-side citations additionally carry a `file_id` (absent for request-side citations).
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

/// Whether document citations are enabled.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CitationsConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextCitation {
    CharLocation {
        cited_text: String,
        document_index: u32,
        document_title: String,
        start_char_index: u32,
        end_char_index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
    },
    PageLocation {
        cited_text: String,
        document_index: u32,
        document_title: String,
        start_page_number: u32,
        end_page_number: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
    },
    ContentBlockLocation {
        cited_text: String,
        document_index: u32,
        document_title: String,
        start_block_index: u32,
        end_block_index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
    },
    WebSearchResultLocation {
        cited_text: String,
        encrypted_index: String,
        title: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
    },
    SearchResultLocation {
        cited_text: String,
        end_block_index: u32,
        search_result_index: u32,
        source: String,
        start_block_index: u32,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
    },
    #[serde(other)]
    Unknown,
}
