//! Structured-output configuration (`output_config`).
//!
//! `output_config` constrains the response format (a JSON schema) and/or the reasoning effort.
//! The `format` object discriminates on `type` (`"json_schema"`) and carries the schema in a
//! `schema` field.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputEffort {
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OutputFormat {
    #[serde(rename = "json_schema")]
    JsonSchema { schema: serde_json::Value },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OutputConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<OutputEffort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OutputFormat>,
}

impl OutputConfig {
    /// Creates a structured-output config that constrains the response to a JSON schema.
    pub fn json_schema(schema: serde_json::Value) -> Self {
        Self {
            effort: None,
            format: Some(OutputFormat::JsonSchema { schema }),
        }
    }
}
