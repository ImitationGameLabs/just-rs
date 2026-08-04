//! Anthropic Messages API wire DTOs.
//!
//! These types intentionally mirror the Messages API wire format. Every enum that deserializes
//! server data ends with an `Unknown` fallback variant for forward compatibility: on
//! internally-tagged enums the unknown payload is discarded (the event/block still parses, but its
//! body is not retained). Note that `#[serde(other)]` affects deserialization only — serializing
//! such an `Unknown` variant emits the variant name (e.g. `{"type":"Unknown"}`), not the original
//! payload.
#![allow(missing_docs)]

pub mod citation;
pub mod event;
pub mod message;
pub mod models;
pub mod output;
pub mod request;
pub mod thinking;
pub mod tool;
pub mod usage;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        event::{ContentBlockDelta, StreamEvent},
        message::{
            ContentBlock, ContentBlockParam, Message, MessageParam, RefusalStopDetails,
            RefusalStopDetailsType, StopReason,
        },
        request::{CountTokensRequest, CreateMessageRequest, Metadata, ServiceTier, SystemParam},
        thinking::{ThinkingConfig, ThinkingDisplay},
        tool::{Tool, ToolChoice},
        usage::Usage,
    };

    fn message_param() -> MessageParam {
        MessageParam::user("Hello, Claude.")
    }

    #[test]
    fn serializes_minimal_create_request() {
        let request = CreateMessageRequest::new("claude-opus-5", vec![message_param()], 1024);

        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["model"], "claude-opus-5");
        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"], "Hello, Claude.");
        assert_eq!(json["max_tokens"], 1024);
        assert!(json.get("system").is_none());
        assert!(json.get("stream").is_none());
        assert!(json.get("tools").is_none());
    }

    #[test]
    fn serializes_system_as_string_and_blocks() {
        let text = CreateMessageRequest::new("claude-opus-5", vec![message_param()], 1024)
            .with_system(SystemParam::Text("Be concise.".to_owned()));
        let json = serde_json::to_value(text).unwrap();
        assert_eq!(json["system"], "Be concise.");

        let blocks = CreateMessageRequest::new("claude-opus-5", vec![message_param()], 1024)
            .with_system(SystemParam::Blocks(vec![
                super::message::TextBlockParam::new("Be concise."),
            ]));
        let json = serde_json::to_value(blocks).unwrap();
        assert_eq!(json["system"][0]["type"], "text");
        assert_eq!(json["system"][0]["text"], "Be concise.");
    }

    #[test]
    fn serializes_message_content_as_string_or_blocks() {
        let param = MessageParam::assistant("plain");
        assert_eq!(
            serde_json::to_value(&param.content).unwrap(),
            json!("plain")
        );

        let blocks = MessageParam::user_blocks(vec![
            ContentBlockParam::Text {
                text: "part".to_owned(),
                cache_control: None,
                citations: None,
            },
            ContentBlockParam::ToolUse {
                id: "toolu_1".to_owned(),
                name: "get_weather".to_owned(),
                input: json!({"city": "Paris"}),
                cache_control: None,
            },
        ]);
        assert_eq!(
            serde_json::to_value(&blocks.content).unwrap(),
            json!([
                { "type": "text", "text": "part" },
                {
                    "type": "tool_use",
                    "id": "toolu_1",
                    "name": "get_weather",
                    "input": { "city": "Paris" }
                }
            ])
        );
    }

    #[test]
    fn serializes_tool_result_block() {
        let result = MessageParam::user_blocks(vec![ContentBlockParam::ToolResult {
            tool_use_id: "toolu_1".to_owned(),
            content: Some(super::message::MessageContent::Text(
                "25 degrees C".to_owned(),
            )),
            is_error: None,
            cache_control: None,
        }]);
        let json = serde_json::to_value(&result.content).unwrap();
        assert_eq!(
            json[0],
            json!({
                "type": "tool_result",
                "tool_use_id": "toolu_1",
                "content": "25 degrees C"
            })
        );
    }

    #[test]
    fn serializes_tools_and_tool_choice() {
        let tool = Tool::new(
            "get_weather",
            json!({
                "type": "object",
                "properties": { "city": { "type": "string" } },
                "required": ["city"]
            }),
        )
        .with_description("Get the current weather for a city.");

        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["name"], "get_weather");
        assert_eq!(json["description"], "Get the current weather for a city.");
        assert_eq!(json["input_schema"]["type"], "object");

        let choice: ToolChoice =
            serde_json::from_value(json!({"type": "tool", "name": "get_weather"})).unwrap();
        assert!(matches!(
            choice,
            ToolChoice::Tool { ref name, .. } if name == "get_weather"
        ));
        assert_eq!(
            serde_json::to_value(choice).unwrap(),
            json!({"type": "tool", "name": "get_weather"})
        );

        let none: ToolChoice = serde_json::from_value(json!({"type": "none"})).unwrap();
        assert!(matches!(none, ToolChoice::None));
    }

    #[test]
    fn serializes_thinking_config() {
        let config = ThinkingConfig::Enabled {
            budget_tokens: 2048,
            display: Some(ThinkingDisplay::Summarized),
        };
        let json = serde_json::to_value(config).unwrap();
        assert_eq!(json["type"], "enabled");
        assert_eq!(json["budget_tokens"], 2048);
        assert_eq!(json["display"], "summarized");

        let disabled = ThinkingConfig::Disabled;
        assert_eq!(
            serde_json::to_value(disabled).unwrap(),
            json!({"type": "disabled"})
        );
    }

    #[test]
    fn deserializes_message_with_tool_calls_and_thinking() {
        let value = json!({
            "id": "msg_1",
            "type": "message",
            "role": "assistant",
            "content": [
                { "type": "thinking", "thinking": "let me think", "signature": "sig_1" },
                { "type": "text", "text": "The weather is sunny." },
                {
                    "type": "tool_use",
                    "id": "toolu_1",
                    "name": "get_weather",
                    "input": { "city": "Paris" }
                }
            ],
            "model": "claude-opus-5",
            "stop_reason": "tool_use",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 10,
                "output_tokens": 20,
                "cache_creation_input_tokens": 5,
                "cache_read_input_tokens": 3
            }
        });

        let message: Message = serde_json::from_value(value).unwrap();

        assert_eq!(message.text(), "The weather is sunny.");
        assert_eq!(message.thinking().unwrap().signature, "sig_1");
        assert_eq!(message.tool_calls().count(), 1);
        assert_eq!(message.tool_calls().next().unwrap().name, "get_weather");
        assert_eq!(message.stop_reason, Some(StopReason::ToolUse));
        assert_eq!(message.usage.cache_read_input_tokens, Some(3));
    }

    #[test]
    fn stop_reason_variants_deserialize() {
        for (wire, expected) in [
            ("end_turn", StopReason::EndTurn),
            ("max_tokens", StopReason::MaxTokens),
            ("stop_sequence", StopReason::StopSequence),
            ("tool_use", StopReason::ToolUse),
            ("pause_turn", StopReason::PauseTurn),
            ("refusal", StopReason::Refusal),
            (
                "model_context_window_exceeded",
                StopReason::ModelContextWindowExceeded,
            ),
        ] {
            let reason: StopReason = serde_json::from_value(json!(wire)).unwrap();
            assert_eq!(reason, expected, "wire {wire:?}");
        }
    }

    #[test]
    fn unknown_events_and_blocks_fall_back() {
        let unknown_event: StreamEvent =
            serde_json::from_value(json!({"type": "some.new.event", "data": 1})).unwrap();
        assert!(matches!(unknown_event, StreamEvent::Unknown));

        let unknown_delta: ContentBlockDelta =
            serde_json::from_value(json!({"type": "new_delta", "x": 1})).unwrap();
        assert!(matches!(unknown_delta, ContentBlockDelta::Unknown));

        let unknown_block: ContentBlock =
            serde_json::from_value(json!({"type": "server_tool_use"})).unwrap();
        assert!(matches!(unknown_block, ContentBlock::Unknown));

        let unknown_param: ContentBlockParam =
            serde_json::from_value(json!({"type": "web_search_tool_result"})).unwrap();
        assert!(matches!(unknown_param, ContentBlockParam::Unknown));
    }

    #[test]
    fn deserializes_stream_events() {
        let start: StreamEvent = serde_json::from_value(json!({
            "type": "message_start",
            "message": {
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": "claude-opus-5",
                "stop_reason": null,
                "stop_sequence": null,
                "usage": { "input_tokens": 10, "output_tokens": 1 }
            }
        }))
        .unwrap();
        assert!(matches!(start, StreamEvent::MessageStart { .. }));

        let delta: StreamEvent = serde_json::from_value(json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": { "type": "text_delta", "text": "Hello" }
        }))
        .unwrap();
        assert!(matches!(
            delta,
            StreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::TextDelta { text },
                ..
            } if text == "Hello"
        ));

        let delta: StreamEvent = serde_json::from_value(json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": { "type": "input_json_delta", "partial_json": "{\"city\":" }
        }))
        .unwrap();
        assert!(matches!(
            delta,
            StreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::InputJsonDelta { .. },
                ..
            }
        ));

        let message_delta: StreamEvent = serde_json::from_value(json!({
            "type": "message_delta",
            "delta": { "stop_reason": "end_turn", "stop_sequence": null },
            "usage": { "output_tokens": 15 }
        }))
        .unwrap();
        assert!(matches!(
            message_delta,
            StreamEvent::MessageDelta { usage, .. } if usage.as_ref().is_some_and(|u| u.output_tokens == 15)
        ));

        let stop: StreamEvent = serde_json::from_value(json!({"type": "message_stop"})).unwrap();
        assert!(matches!(stop, StreamEvent::MessageStop));

        let error: StreamEvent = serde_json::from_value(json!({
            "type": "error",
            "error": { "type": "overloaded_error", "message": "Overloaded" }
        }))
        .unwrap();
        assert!(matches!(
            error,
            StreamEvent::Error { error } if error.message == "Overloaded"
        ));
    }

    #[test]
    fn serializes_count_tokens_request() {
        let request = CountTokensRequest::new("claude-opus-5", vec![message_param()])
            .with_system(SystemParam::Text("Be concise.".to_owned()));
        let json = serde_json::to_value(request).unwrap();
        assert_eq!(json["model"], "claude-opus-5");
        assert_eq!(json["system"], "Be concise.");
    }

    #[test]
    fn usage_with_thinking_details() {
        let value = json!({
            "input_tokens": 10,
            "output_tokens": 20,
            "output_tokens_details": { "thinking_tokens": 5 }
        });
        let usage: Usage = serde_json::from_value(value).unwrap();
        assert_eq!(
            usage.output_tokens_details.unwrap().thinking_tokens,
            Some(5)
        );
    }

    #[test]
    fn serializes_extended_request_fields() {
        let request = CreateMessageRequest::new("claude-opus-5", vec![message_param()], 1024)
            .with_thinking(ThinkingConfig::Adaptive { display: None })
            .with_metadata(Metadata {
                user_id: Some("user_1".to_owned()),
            })
            .with_service_tier(ServiceTier::StandardOnly)
            .with_cache_control(super::message::CacheControlEphemeral::new());

        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["thinking"]["type"], "adaptive");
        assert_eq!(json["metadata"]["user_id"], "user_1");
        assert_eq!(json["service_tier"], "standard_only");
        assert_eq!(json["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn deserializes_redacted_thinking_block() {
        let value = json!({
            "id": "msg_1",
            "type": "message",
            "role": "assistant",
            "content": [ { "type": "redacted_thinking", "data": "redacted" } ],
            "model": "claude-opus-5",
            "stop_reason": null,
            "stop_sequence": null,
            "usage": { "input_tokens": 1, "output_tokens": 1 }
        });
        let message: Message = serde_json::from_value(value).unwrap();
        assert!(matches!(
            message.content[0],
            ContentBlock::RedactedThinking { .. }
        ));
    }

    #[test]
    fn deserializes_streaming_refusal_with_stop_details() {
        let event: StreamEvent = serde_json::from_value(json!({
            "type": "message_delta",
            "delta": {
                "stop_reason": "refusal",
                "stop_sequence": null,
                "stop_details": {
                    "type": "refusal",
                    "category": "cyber",
                    "explanation": "declined"
                }
            },
            "usage": { "output_tokens": 5 }
        }))
        .unwrap();
        assert!(matches!(
            event,
            StreamEvent::MessageDelta { delta, .. }
                if delta.stop_details.as_ref().is_some_and(|d| d.category.as_deref() == Some("cyber"))
        ));
    }

    #[test]
    fn refusal_stop_details_round_trips_with_discriminator() {
        let details = RefusalStopDetails {
            r#type: RefusalStopDetailsType::Refusal,
            category: None,
            explanation: None,
        };
        assert_eq!(
            serde_json::to_value(&details).unwrap(),
            json!({ "type": "refusal" })
        );
    }

    #[test]
    fn unknown_role_and_type_fall_back() {
        let role: super::message::MessageRole = serde_json::from_value(json!("developer")).unwrap();
        assert!(matches!(role, super::message::MessageRole::Unknown));

        let r#type: super::message::MessageType = serde_json::from_value(json!("other")).unwrap();
        assert!(matches!(r#type, super::message::MessageType::Unknown));
    }

    #[test]
    fn serializes_output_config_json_schema() {
        let config = super::output::OutputConfig::json_schema(json!({
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "required": ["name"],
            "additionalProperties": false
        }));
        let json = serde_json::to_value(config).unwrap();
        assert_eq!(json["format"]["type"], "json_schema");
        assert_eq!(
            json["format"]["schema"]["properties"]["name"]["type"],
            "string"
        );

        let request = CreateMessageRequest::new("claude-opus-5", vec![message_param()], 1024)
            .with_output_config(super::output::OutputConfig::json_schema(json!({
                "type": "object",
                "properties": {}
            })));
        let json = serde_json::to_value(request).unwrap();
        assert_eq!(json["output_config"]["format"]["type"], "json_schema");
    }

    #[test]
    fn serializes_output_config_effort() {
        let config = super::output::OutputConfig {
            effort: Some(super::output::OutputEffort::Xhigh),
            format: None,
        };
        let json = serde_json::to_value(config).unwrap();
        assert_eq!(json["effort"], "xhigh");
        assert!(json.get("format").is_none());
    }

    #[test]
    fn serializes_document_block() {
        let block = ContentBlockParam::Document {
            source: super::message::DocumentSource::Url {
                url: "https://example.com/doc.pdf".to_owned(),
            },
            title: Some("Report".to_owned()),
            context: None,
            citations: Some(super::citation::CitationsConfig { enabled: true }),
            cache_control: None,
        };
        let json = serde_json::to_value(block).unwrap();
        assert_eq!(json["type"], "document");
        assert_eq!(json["source"]["type"], "url");
        assert_eq!(json["source"]["url"], "https://example.com/doc.pdf");
        assert_eq!(json["title"], "Report");
        assert_eq!(json["citations"]["enabled"], true);
    }

    #[test]
    fn deserializes_response_text_with_citations() {
        let value = json!({
            "id": "msg_1",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "The capital is Paris.",
                    "citations": [
                        {
                            "type": "web_search_result_location",
                            "cited_text": "Paris is the capital",
                            "document_index": 0,
                            "document_title": "Wikipedia",
                            "url": "https://example.com/paris",
                            "title": "Paris",
                            "encrypted_index": "abc",
                            "file_id": "file_1"
                        }
                    ]
                }
            ],
            "model": "claude-opus-5",
            "stop_reason": null,
            "stop_sequence": null,
            "usage": { "input_tokens": 1, "output_tokens": 1 }
        });
        let message: Message = serde_json::from_value(value).unwrap();
        match &message.content[0] {
            ContentBlock::Text { citations, .. } => {
                let citation = citations
                    .as_ref()
                    .expect("citations should be present")
                    .first()
                    .unwrap();
                assert!(matches!(
                    citation,
                    super::citation::TextCitation::WebSearchResultLocation { url, file_id, .. }
                        if url == "https://example.com/paris" && file_id.as_deref() == Some("file_1")
                ));
            }
            other => panic!("expected text block, got {other:?}"),
        }
    }

    #[test]
    fn deserializes_citations_delta_event() {
        let event: StreamEvent = serde_json::from_value(json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": {
                "type": "citations_delta",
                "citation": {
                    "type": "char_location",
                    "cited_text": "Paris",
                    "document_index": 0,
                    "document_title": "Doc",
                    "start_char_index": 0,
                    "end_char_index": 5
                }
            }
        }))
        .unwrap();
        assert!(matches!(
            event,
            StreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::CitationsDelta { .. },
                ..
            }
        ));
    }
}
