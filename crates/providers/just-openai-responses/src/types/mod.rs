//! OpenAI Responses API wire DTOs.
//!
//! These types intentionally mirror the Responses API wire format. Every enum that deserializes
//! server data ends with an `Unknown` fallback variant for forward compatibility: on
//! internally-tagged enums the unknown payload is discarded (the event/item still parses, but its
//! body is not retained). Note that `#[serde(other)]` affects deserialization only — serializing
//! such an `Unknown` variant emits the variant name (`{"type":"Unknown"}`), not the original
//! payload.
#![allow(missing_docs)]

pub mod event;
pub mod item;
pub mod message;
pub mod models;
pub mod request;
pub mod response;
pub mod shared;
pub mod tool;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        event::StreamEvent,
        item::{FunctionCall, InputItem, OutputItem},
        message::{InputContentPart, InputMessage, MessageContent, MessageRole},
        request::{CreateResponseRequest, ResponseInput},
        response::{IncompleteReason, Response, ResponseStatus},
        shared::{
            ReasoningConfig, ResponseIncludable, TextConfig, TextFormat, ToolChoice, ToolChoiceMode,
        },
        tool::ResponseTool,
    };

    #[test]
    fn serializes_minimal_create_request() {
        let request = CreateResponseRequest::new("gpt-5.6")
            .with_input(ResponseInput::text("hello"))
            .with_instructions("Be concise.");

        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["model"], "gpt-5.6");
        assert_eq!(json["input"], "hello");
        assert_eq!(json["instructions"], "Be concise.");
        assert!(json.get("stream").is_none());
        assert!(json.get("tools").is_none());
    }

    #[test]
    fn serializes_input_items_array() {
        let request = CreateResponseRequest::new("gpt-5.6").with_input(ResponseInput::items(vec![
            InputItem::Message(InputMessage::user("What is the weather?")),
            InputItem::FunctionCall(FunctionCall::new(
                "get_weather",
                "call_1",
                r#"{"city":"Paris"}"#,
            )),
            InputItem::FunctionCallOutput(super::item::FunctionCallOutput::new(
                "call_1",
                "{\"temp\":25}",
            )),
        ]));

        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["input"][0]["type"], "message");
        assert_eq!(json["input"][0]["role"], "user");
        assert_eq!(json["input"][0]["content"], "What is the weather?");
        assert_eq!(json["input"][1]["type"], "function_call");
        assert_eq!(json["input"][1]["call_id"], "call_1");
        assert_eq!(json["input"][1]["arguments"], r#"{"city":"Paris"}"#);
        assert_eq!(json["input"][2]["type"], "function_call_output");
        assert_eq!(json["input"][2]["output"], "{\"temp\":25}");
    }

    #[test]
    fn serializes_tools_and_tool_choice() {
        let mut request = CreateResponseRequest::new("gpt-5.6");
        request.tools = Some(vec![ResponseTool::function(
            "get_weather",
            "Get the weather",
            json!({
                "type": "object",
                "properties": { "city": { "type": "string" } },
                "required": ["city"],
                "additionalProperties": false
            }),
        )]);
        request.tool_choice = Some(ToolChoice::Mode(ToolChoiceMode::Auto));

        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["tools"][0]["type"], "function");
        assert_eq!(json["tools"][0]["name"], "get_weather");
        assert_eq!(json["tool_choice"], "auto");
    }

    #[test]
    fn deserializes_tool_choice_objects() {
        let object: ToolChoice =
            serde_json::from_value(json!({"type": "function", "name": "get_weather"})).unwrap();
        assert!(matches!(object, ToolChoice::Object(_)));

        let allowed: ToolChoice = serde_json::from_value(json!({
            "type": "allowed_tools",
            "mode": "required",
            "tools": [{"type": "function", "name": "a"}]
        }))
        .unwrap();
        assert!(matches!(allowed, ToolChoice::Object(_)));

        let mode: ToolChoice = serde_json::from_value(json!("required")).unwrap();
        assert!(matches!(mode, ToolChoice::Mode(ToolChoiceMode::Required)));

        let builtin: ToolChoice = serde_json::from_value(json!({"type": "file_search"})).unwrap();
        assert!(matches!(
            builtin,
            ToolChoice::Object(super::shared::ToolChoiceObject::FileSearch)
        ));
        assert_eq!(
            serde_json::to_value(builtin).unwrap(),
            json!({"type": "file_search"})
        );
    }

    #[test]
    fn deserializes_response_with_output_items() {
        let value = json!({
            "id": "resp_1",
            "object": "response",
            "created_at": 1700000000,
            "status": "completed",
            "model": "gpt-5.6",
            "output": [
                {
                    "id": "msg_1",
                    "type": "message",
                    "role": "assistant",
                    "status": "completed",
                    "content": [
                        {
                            "type": "output_text",
                            "text": "It is 25 degrees.",
                            "annotations": []
                        }
                    ]
                },
                {
                    "type": "reasoning",
                    "id": "rs_1",
                    "summary": [{"type": "summary_text", "text": "checked weather"}],
                    "encrypted_content": "encrypted==",
                    "status": "completed"
                }
            ],
            "usage": {
                "input_tokens": 10,
                "input_tokens_details": { "cache_write_tokens": 0, "cached_tokens": 0 },
                "output_tokens": 5,
                "output_tokens_details": { "reasoning_tokens": 2 },
                "total_tokens": 15
            }
        });

        let response: Response = serde_json::from_value(value).unwrap();

        assert_eq!(response.status, ResponseStatus::Completed);
        assert_eq!(response.output_text(), "It is 25 degrees.");
        assert_eq!(response.output.len(), 2);
        assert!(matches!(response.output[0], OutputItem::Message(_)));
        assert!(matches!(response.output[1], OutputItem::Reasoning(_)));
        assert_eq!(response.usage.as_ref().unwrap().total_tokens, 15);
    }

    #[test]
    fn deserializes_response_with_error_and_incomplete_details() {
        let value = json!({
            "id": "resp_1",
            "object": "response",
            "created_at": 1,
            "status": "incomplete",
            "model": "gpt-5.6",
            "output": [],
            "error": null,
            "incomplete_details": { "reason": "max_output_tokens" }
        });

        let response: Response = serde_json::from_value(value).unwrap();

        assert_eq!(response.status, ResponseStatus::Incomplete);
        assert!(response.error.is_none());
        assert_eq!(
            response.incomplete_details.as_ref().unwrap().reason,
            Some(IncompleteReason::MaxOutputTokens)
        );
    }

    #[test]
    fn deserializes_stream_events_with_dotted_types() {
        let created: StreamEvent = serde_json::from_value(json!({
            "type": "response.created",
            "response": {
                "id": "resp_1",
                "object": "response",
                "created_at": 1,
                "status": "in_progress",
                "model": "gpt-5.6",
                "output": []
            }
        }))
        .unwrap();
        assert!(matches!(created, StreamEvent::ResponseCreated { .. }));

        let delta: StreamEvent = serde_json::from_value(json!({
            "type": "response.output_text.delta",
            "item_id": "msg_1",
            "output_index": 0,
            "content_index": 0,
            "delta": "Hello"
        }))
        .unwrap();
        assert!(matches!(
            delta,
            StreamEvent::ResponseOutputTextDelta { delta, .. } if delta == "Hello"
        ));

        let done: StreamEvent = serde_json::from_value(json!({
            "type": "response.function_call_arguments.done",
            "item_id": "fc_1",
            "output_index": 0,
            "name": "get_weather",
            "arguments": r#"{"city":"Paris"}"#
        }))
        .unwrap();
        assert!(matches!(
            done,
            StreamEvent::ResponseFunctionCallArgumentsDone { name, .. } if name == "get_weather"
        ));
    }

    #[test]
    fn unknown_event_and_item_types_fall_back() {
        let unknown_event: StreamEvent =
            serde_json::from_value(json!({"type": "some.new.event", "data": 1})).unwrap();
        assert!(matches!(unknown_event, StreamEvent::Unknown));

        let unknown_item: InputItem =
            serde_json::from_value(json!({"type": "computer_call", "id": "cc_1"})).unwrap();
        assert!(matches!(unknown_item, InputItem::Unknown));

        let unknown_tool: ResponseTool =
            serde_json::from_value(json!({"type": "code_interpreter"})).unwrap();
        assert!(matches!(unknown_tool, ResponseTool::Unknown));
    }

    #[test]
    fn serializes_text_format_json_schema() {
        let config = TextConfig {
            format: Some(TextFormat::JsonSchema {
                name: "calendar_event".to_owned(),
                schema: json!({"type": "object"}),
                description: None,
                strict: Some(true),
            }),
            verbosity: None,
        };

        let json = serde_json::to_value(config).unwrap();

        assert_eq!(json["format"]["type"], "json_schema");
        assert_eq!(json["format"]["name"], "calendar_event");
        assert_eq!(json["format"]["strict"], true);
    }

    #[test]
    fn response_includable_produces_dotted_strings() {
        let values = [
            (
                ResponseIncludable::FileSearchCallResults,
                "file_search_call.results",
            ),
            (
                ResponseIncludable::WebSearchCallResults,
                "web_search_call.results",
            ),
            (
                ResponseIncludable::WebSearchCallActionSources,
                "web_search_call.action.sources",
            ),
            (
                ResponseIncludable::MessageInputImageImageUrl,
                "message.input_image.image_url",
            ),
            (
                ResponseIncludable::ComputerCallOutputOutputImageUrl,
                "computer_call_output.output.image_url",
            ),
            (
                ResponseIncludable::CodeInterpreterCallOutputs,
                "code_interpreter_call.outputs",
            ),
            (
                ResponseIncludable::ReasoningEncryptedContent,
                "reasoning.encrypted_content",
            ),
            (
                ResponseIncludable::MessageOutputTextLogprobs,
                "message.output_text.logprobs",
            ),
        ];
        for (value, wire) in values {
            assert_eq!(value.as_str(), wire);
            assert_eq!(serde_json::to_value(value).unwrap(), json!(wire));
        }
    }

    #[test]
    fn deserializes_reasoning_summary_and_terminal_events() {
        let summary_delta: StreamEvent = serde_json::from_value(json!({
            "type": "response.reasoning_summary_text.delta",
            "item_id": "rs_1",
            "output_index": 0,
            "summary_index": 2,
            "delta": "reasoned"
        }))
        .unwrap();
        assert!(matches!(
            summary_delta,
            StreamEvent::ResponseReasoningSummaryTextDelta {
                summary_index: 2,
                ..
            }
        ));

        let summary_done: StreamEvent = serde_json::from_value(json!({
            "type": "response.reasoning_summary_text.done",
            "item_id": "rs_1",
            "output_index": 0,
            "summary_index": 2,
            "text": "reasoned about the weather"
        }))
        .unwrap();
        assert!(matches!(
            summary_done,
            StreamEvent::ResponseReasoningSummaryTextDone {
                summary_index: 2,
                text,
                ..
            } if text == "reasoned about the weather"
        ));

        let refusal_done: StreamEvent = serde_json::from_value(json!({
            "type": "response.refusal.done",
            "item_id": "msg_1",
            "output_index": 0,
            "content_index": 0,
            "refusal": "I cannot help with that."
        }))
        .unwrap();
        assert!(matches!(
            refusal_done,
            StreamEvent::ResponseRefusalDone { refusal, .. } if refusal == "I cannot help with that."
        ));

        let reasoning_done: StreamEvent = serde_json::from_value(json!({
            "type": "response.reasoning_text.done",
            "item_id": "rs_1",
            "output_index": 0,
            "content_index": 0,
            "text": "full reasoning"
        }))
        .unwrap();
        assert!(matches!(
            reasoning_done,
            StreamEvent::ResponseReasoningTextDone { text, .. } if text == "full reasoning"
        ));

        let text_done: StreamEvent = serde_json::from_value(json!({
            "type": "response.output_text.done",
            "item_id": "msg_1",
            "output_index": 0,
            "content_index": 0,
            "text": "done",
            "logprobs": [
                { "token": "done", "logprob": -0.1 }
            ]
        }))
        .unwrap();
        assert!(matches!(
            text_done,
            StreamEvent::ResponseOutputTextDone { text, logprobs, .. }
                if text == "done" && logprobs.as_ref().is_some_and(|l| l.len() == 1)
        ));
    }

    #[test]
    fn deserializes_response_with_reasoning_and_conversation() {
        let value = json!({
            "id": "resp_1",
            "object": "response",
            "created_at": 1,
            "status": "completed",
            "model": "gpt-5.6",
            "output": [],
            "reasoning": { "effort": "high", "summary": "auto" },
            "conversation": { "id": "conv_1" }
        });

        let response: Response = serde_json::from_value(value).unwrap();

        assert_eq!(
            response.reasoning.as_ref().unwrap().effort,
            Some(super::shared::ReasoningEffort::High)
        );
        assert_eq!(response.conversation.as_ref().unwrap().id, "conv_1");
    }

    #[test]
    fn reasoning_effort_wire_forms_are_pinned() {
        use super::shared::ReasoningEffort;

        let forms = [
            (ReasoningEffort::None, "none"),
            (ReasoningEffort::Minimal, "minimal"),
            (ReasoningEffort::Low, "low"),
            (ReasoningEffort::Medium, "medium"),
            (ReasoningEffort::High, "high"),
            (ReasoningEffort::Xhigh, "xhigh"),
            (ReasoningEffort::Max, "max"),
            // The fallback pins its variant-name form: a deliberate contract point.
            (ReasoningEffort::Unknown, "Unknown"),
        ];
        for (effort, wire) in forms {
            assert_eq!(serde_json::to_value(effort).unwrap(), json!(wire));
            assert_eq!(
                serde_json::from_value::<ReasoningEffort>(json!(wire)).unwrap(),
                effort
            );
        }

        // Unrecognized values fall back to Unknown instead of failing the parse.
        assert_eq!(
            serde_json::from_value::<ReasoningEffort>(json!("ultra")).unwrap(),
            ReasoningEffort::Unknown
        );
    }

    #[test]
    fn message_content_supports_string_or_parts() {
        let text = MessageContent::Text("plain".to_owned());
        assert_eq!(serde_json::to_value(&text).unwrap(), json!("plain"));

        let parts = MessageContent::Parts(vec![InputContentPart::InputText {
            text: "part".to_owned(),
            prompt_cache_breakpoint: None,
        }]);
        assert_eq!(
            serde_json::to_value(&parts).unwrap(),
            json!([{
                "type": "input_text",
                "text": "part"
            }])
        );
    }

    #[test]
    fn input_role_serializes_known_values() {
        assert_eq!(
            serde_json::to_value(MessageRole::Developer).unwrap(),
            json!("developer")
        );
    }

    #[test]
    fn with_reasoning_auto_injects_encrypted_content_include() {
        let request = CreateResponseRequest::new("gpt-5.6").with_reasoning(ReasoningConfig {
            effort: Some(super::shared::ReasoningEffort::High),
            ..Default::default()
        });
        let json = serde_json::to_value(request).unwrap();
        assert_eq!(json["reasoning"]["effort"], "high");
        assert_eq!(json["include"], json!(["reasoning.encrypted_content"]));
    }

    #[test]
    fn explicit_include_opts_out_of_auto_injection() {
        let request = CreateResponseRequest::new("gpt-5.6")
            .with_include(vec![ResponseIncludable::WebSearchCallResults])
            .with_reasoning(ReasoningConfig {
                effort: Some(super::shared::ReasoningEffort::High),
                ..Default::default()
            });
        let json = serde_json::to_value(request).unwrap();
        assert_eq!(json["include"], json!(["web_search_call.results"]));
    }

    #[test]
    fn tolerates_absent_optional_fields() {
        let value = json!({
            "id": "resp_1",
            "object": "response",
            "created_at": 1700000000,
            "status": "completed",
            "model": "gpt-5.6",
            "output": [
                {
                    "id": "msg_1",
                    "type": "message",
                    "role": "assistant",
                    "content": [
                        {
                            "type": "output_text",
                            "text": "It is 25 degrees.",
                            "logprobs": [{ "token": "It", "logprob": -0.5 }]
                        }
                    ]
                }
            ],
            "usage": {
                "input_tokens": 10,
                "output_tokens": 5,
                "total_tokens": 15
            }
        });

        let response: Response = serde_json::from_value(value).unwrap();

        let usage = response.usage.as_ref().unwrap();
        assert_eq!(usage.input_tokens_details, None);
        assert_eq!(usage.output_tokens_details, None);

        use super::item::OutputItem;
        use super::message::OutputContentPart;
        let OutputItem::Message(message) = &response.output[0] else {
            panic!("expected a message output item");
        };
        assert_eq!(message.status, None);
        let OutputContentPart::OutputText {
            annotations,
            logprobs,
            ..
        } = &message.content[0]
        else {
            panic!("expected an output text content part");
        };
        assert_eq!(*annotations, None);
        let logprobs = logprobs.as_ref().unwrap();
        assert_eq!(logprobs[0].top_logprobs, None);
        assert_eq!(logprobs[0].bytes, None);
    }
}
