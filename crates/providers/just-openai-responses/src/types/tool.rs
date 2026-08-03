//! Tool definition wire types (`POST /responses` body `tools`).
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchContextSize {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(other)]
    Unknown,
}

/// A tool made available to the model, discriminated by `type`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ResponseTool {
    #[serde(rename = "function")]
    Function {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// A JSON Schema object describing the function parameters.
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
        /// A JSON Schema object describing the JSON value encoded in string outputs.
        #[serde(skip_serializing_if = "Option::is_none")]
        output_schema: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_callers: Option<Vec<String>>,
    },
    #[serde(rename = "custom")]
    Custom {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
    },
    #[serde(rename = "web_search")]
    WebSearch {
        #[serde(skip_serializing_if = "Option::is_none")]
        filters: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<SearchContextSize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<Value>,
    },
    #[serde(rename = "file_search")]
    FileSearch {
        vector_store_ids: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filters: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_num_results: Option<u32>,
    },
    #[serde(other)]
    Unknown,
}

impl ResponseTool {
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
    ) -> Self {
        Self::Function {
            name: name.into(),
            description: Some(description.into()),
            parameters: Some(parameters),
            strict: None,
            output_schema: None,
            defer_loading: None,
            allowed_callers: None,
        }
    }

    /// A web-search tool that respects an optional allowed-domain list.
    pub fn web_search(allowed_domains: Option<Vec<String>>) -> Self {
        Self::WebSearch {
            filters: allowed_domains
                .map(|domains| serde_json::json!({ "allowed_domains": domains })),
            search_context_size: None,
            user_location: None,
        }
    }

    /// A file-search tool scoped to the given vector stores.
    pub fn file_search(vector_store_ids: Vec<String>) -> Self {
        Self::FileSearch {
            vector_store_ids,
            filters: None,
            max_num_results: None,
        }
    }
}
