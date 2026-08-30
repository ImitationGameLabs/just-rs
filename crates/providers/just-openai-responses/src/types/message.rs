//! Message and content-part wire types shared by input items, output items, and events.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "developer")]
    Developer,
    #[serde(other)]
    Unknown,
}

/// Output messages always carry the `assistant` role.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputRole {
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phase {
    #[serde(rename = "commentary")]
    Commentary,
    #[serde(rename = "final_answer")]
    FinalAnswer,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemStatus {
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "incomplete")]
    Incomplete,
    #[serde(other)]
    Unknown,
}

/// Message content is either a plain string (input) or an array of content parts.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<InputContentPart>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageDetail {
    Low,
    High,
    Auto,
    Original,
    /// Unrecognized detail value, preserved verbatim for lossless round-tripping.
    Unknown(String),
}

impl Serialize for ImageDetail {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            Self::Low => "low",
            Self::High => "high",
            Self::Auto => "auto",
            Self::Original => "original",
            Self::Unknown(value) => value.as_str(),
        };
        serializer.serialize_str(value)
    }
}

impl<'de> Deserialize<'de> for ImageDetail {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "low" => Self::Low,
            "high" => Self::High,
            "auto" => Self::Auto,
            "original" => Self::Original,
            _ => Self::Unknown(value.clone()),
        })
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileDetail {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "auto")]
    Auto,
    #[serde(other)]
    Unknown,
}

/// Marks the exact end of a reusable prompt prefix. Mode is always `explicit`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PromptCacheBreakpoint {
    #[serde(default = "default_explicit_mode")]
    pub mode: String,
}

fn default_explicit_mode() -> String {
    "explicit".to_owned()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum InputContentPart {
    #[serde(rename = "input_text")]
    InputText {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        prompt_cache_breakpoint: Option<PromptCacheBreakpoint>,
    },
    #[serde(rename = "input_image")]
    InputImage {
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<ImageDetail>,
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        prompt_cache_breakpoint: Option<PromptCacheBreakpoint>,
    },
    #[serde(rename = "input_file")]
    InputFile {
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<FileDetail>,
    },
    #[serde(other)]
    Unknown,
}

/// An input message carrying `role`, `content`, and optional `phase`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InputMessage {
    pub role: MessageRole,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<Phase>,
}

impl InputMessage {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: MessageContent::Text(content.into()),
            phase: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    pub fn developer(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Developer, content)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OutputMessage {
    pub id: String,
    pub role: OutputRole,
    pub content: Vec<OutputContentPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<Phase>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum OutputContentPart {
    #[serde(rename = "output_text")]
    OutputText {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<Vec<Annotation>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        logprobs: Option<Vec<OutputLogprob>>,
    },
    #[serde(rename = "refusal")]
    Refusal { refusal: String },
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Annotation {
    #[serde(rename = "file_citation")]
    FileCitation {
        file_id: String,
        filename: String,
        index: u32,
    },
    #[serde(rename = "url_citation")]
    UrlCitation {
        start_index: u32,
        end_index: u32,
        title: String,
        url: String,
    },
    #[serde(rename = "file_path")]
    FilePath { file_id: String, index: u32 },
    #[serde(rename = "container_file_citation")]
    ContainerFileCitation {
        container_id: String,
        file_id: String,
        filename: String,
        start_index: u32,
        end_index: u32,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OutputLogprob {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    pub logprob: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<TopLogprob>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TopLogprob {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    pub logprob: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SummaryText {
    #[serde(rename = "summary_text")]
    Summary { text: String },
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ReasoningText {
    #[serde(rename = "reasoning_text")]
    Reasoning { text: String },
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::ImageDetail;

    #[test]
    fn image_detail_preserves_unknown_values() {
        assert_eq!(
            serde_json::to_value(ImageDetail::Unknown("hd".to_owned())).unwrap(),
            serde_json::json!("hd")
        );
        let back: ImageDetail = serde_json::from_value(serde_json::json!("hd")).unwrap();
        assert_eq!(back, ImageDetail::Unknown("hd".to_owned()));

        assert_eq!(
            serde_json::to_value(ImageDetail::Low).unwrap(),
            serde_json::json!("low")
        );
        let back: ImageDetail = serde_json::from_value(serde_json::json!("low")).unwrap();
        assert_eq!(back, ImageDetail::Low);
    }
}
