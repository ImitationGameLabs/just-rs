use futures_util::StreamExt;
use just_anthropic::{
    AnthropicClient, Error,
    types::{
        event::{ContentBlockDelta, StreamEvent},
        message::{ContentBlock, ContentBlockParam, ImageSource, MessageParam},
        request::{CountTokensRequest, CreateMessageRequest},
        thinking::ThinkingConfig,
        tool::{Tool, ToolChoice},
    },
};
use just_common::error::TransportError;
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, header, method, path},
};

fn client(server: &MockServer) -> AnthropicClient {
    AnthropicClient::builder()
        .api_key("test-key")
        .base_url(server.uri())
        .build()
        .unwrap()
}

fn basic_request() -> CreateMessageRequest {
    CreateMessageRequest::new("claude-opus-5", vec![MessageParam::user("Say hello.")], 256)
}

fn response_body() -> serde_json::Value {
    json!({
        "id": "msg_1",
        "type": "message",
        "role": "assistant",
        "content": [
            { "type": "text", "text": "Hello!" }
        ],
        "model": "claude-opus-5",
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {
            "input_tokens": 10,
            "output_tokens": 5,
            "cache_creation_input_tokens": 0,
            "cache_read_input_tokens": 0
        }
    })
}

#[tokio::test]
async fn creates_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(header("x-api-key", "test-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .and(body_partial_json(json!({
            "model": "claude-opus-5",
            "messages": [{ "role": "user", "content": "Say hello." }],
            "max_tokens": 256
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body()))
        .mount(&server)
        .await;

    let message = client(&server)
        .create_message(basic_request())
        .await
        .unwrap();

    assert_eq!(message.id, "msg_1");
    assert_eq!(message.text(), "Hello!");
    assert_eq!(
        message.stop_reason,
        Some(just_anthropic::types::message::StopReason::EndTurn)
    );
}

#[tokio::test]
async fn rejects_stream_flag_on_non_stream_method() {
    let request = CreateMessageRequest {
        stream: Some(true),
        ..basic_request()
    };

    let error = AnthropicClient::builder()
        .api_key("test-key")
        .base_url("http://127.0.0.1:0")
        .build()
        .unwrap()
        .create_message(request)
        .await
        .unwrap_err();

    assert!(matches!(error, Error::InvalidRequest(_)));
}

#[test]
fn prepare_streaming_forces_stream_true() {
    let client = AnthropicClient::builder()
        .api_key("test-key")
        .base_url("https://api.anthropic.com/v1")
        .build()
        .unwrap();

    let prepared = client.prepare_streaming(basic_request()).unwrap();
    let body = prepared.body().and_then(|b| b.as_bytes()).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(body).unwrap();
    assert_eq!(parsed["stream"], true);

    // Non-streaming prepare must not set stream.
    let prepared = client.prepare(basic_request()).unwrap();
    let body = prepared.body().and_then(|b| b.as_bytes()).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(body).unwrap();
    assert!(parsed.get("stream").is_none());
}

#[tokio::test]
async fn streams_message_events() {
    let server = MockServer::start().await;
    let body = concat!(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-opus-5\",\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":1}}}\n\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hel\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"lo\"}}\n\n",
        "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":15}}\n\n",
        "data: {\"type\":\"message_stop\"}\n\n"
    );

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let mut stream = client(&server)
        .stream_message(basic_request())
        .await
        .unwrap();

    let start = stream.next().await.unwrap().unwrap();
    assert!(matches!(start, StreamEvent::MessageStart { .. }));

    let block_start = stream.next().await.unwrap().unwrap();
    assert!(matches!(
        block_start,
        StreamEvent::ContentBlockStart { index: 0, .. }
    ));

    let delta = stream.next().await.unwrap().unwrap();
    assert!(matches!(
        delta,
        StreamEvent::ContentBlockDelta {
            delta: ContentBlockDelta::TextDelta { text },
            ..
        } if text == "Hel"
    ));

    let delta = stream.next().await.unwrap().unwrap();
    assert!(matches!(
        delta,
        StreamEvent::ContentBlockDelta {
            delta: ContentBlockDelta::TextDelta { text },
            ..
        } if text == "lo"
    ));

    let block_stop = stream.next().await.unwrap().unwrap();
    assert!(matches!(
        block_stop,
        StreamEvent::ContentBlockStop { index: 0 }
    ));

    let message_delta = stream.next().await.unwrap().unwrap();
    assert!(matches!(
        message_delta,
        StreamEvent::MessageDelta { usage, .. }
            if usage.as_ref().is_some_and(|u| u.output_tokens == 15)
    ));

    let stop = stream.next().await.unwrap().unwrap();
    assert!(matches!(stop, StreamEvent::MessageStop));

    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn unknown_stream_event_does_not_break_stream() {
    let server = MockServer::start().await;
    let body = concat!(
        "data: {\"type\":\"some.new.event\",\"payload\":true}\n\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}\n\n",
        "data: {\"type\":\"message_stop\"}\n\n"
    );

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let mut stream = client(&server)
        .stream_message(basic_request())
        .await
        .unwrap();

    let unknown = stream.next().await.unwrap().unwrap();
    assert!(matches!(unknown, StreamEvent::Unknown));

    let delta = stream.next().await.unwrap().unwrap();
    assert!(matches!(
        delta,
        StreamEvent::ContentBlockDelta {
            delta: ContentBlockDelta::TextDelta { text },
            ..
        } if text == "Hi"
    ));

    assert!(stream.next().await.is_some());
}

#[tokio::test]
async fn counts_tokens() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/messages/count_tokens"))
        .and(header("x-api-key", "test-key"))
        .and(body_partial_json(json!({
            "model": "claude-opus-5",
            "messages": [{ "role": "user", "content": "Say hello." }]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "input_tokens": 12
        })))
        .mount(&server)
        .await;

    let request = CountTokensRequest::new("claude-opus-5", vec![MessageParam::user("Say hello.")]);
    let response = client(&server).count_tokens(request).await.unwrap();

    assert_eq!(response.input_tokens, 12);
}

#[tokio::test]
async fn lists_models() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "type": "model",
                    "id": "claude-opus-5",
                    "display_name": "Claude Opus 5",
                    "created_at": "2025-05-01T00:00:00Z"
                }
            ]
        })))
        .mount(&server)
        .await;

    let response = client(&server).list_models().await.unwrap();

    assert_eq!(response.data[0].id, "claude-opus-5");
    assert_eq!(
        response.data[0].display_name.as_deref(),
        Some("Claude Opus 5")
    );
}

#[tokio::test]
async fn sends_custom_api_version_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [] })))
        .mount(&server)
        .await;

    let client = AnthropicClient::builder()
        .api_key("test-key")
        .api_version("2023-06-01")
        .base_url(server.uri())
        .build()
        .unwrap();
    let response = client.list_models().await.unwrap();

    assert!(response.data.is_empty());
}

#[tokio::test]
async fn streams_tool_use_and_thinking() {
    let server = MockServer::start().await;
    let body = concat!(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-opus-5\",\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":1}}}\n\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"thinking\":\"\",\"signature\":\"\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"need weather\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"signature_delta\",\"signature\":\"sig_1\"}}\n\n",
        "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
        "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"get_weather\",\"input\":{}}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"city\\\":\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"Paris\\\"}\"}}\n\n",
        "data: {\"type\":\"content_block_stop\",\"index\":1}\n\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":30}}\n\n",
        "data: {\"type\":\"message_stop\"}\n\n"
    );

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let mut stream = client(&server)
        .stream_message(basic_request())
        .await
        .unwrap();

    let mut thinking_deltas = Vec::new();
    let mut signature = None;
    let mut input_json = String::new();

    while let Some(event) = stream.next().await {
        let event = event.unwrap();
        match event {
            StreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::ThinkingDelta { thinking },
                ..
            } => thinking_deltas.push(thinking),
            StreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::SignatureDelta { signature: sig },
                ..
            } => signature = Some(sig),
            StreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::InputJsonDelta { partial_json },
                ..
            } => input_json.push_str(&partial_json),
            _ => {}
        }
    }

    assert_eq!(thinking_deltas, vec!["need weather"]);
    assert_eq!(signature.as_deref(), Some("sig_1"));
    assert_eq!(input_json, r#"{"city":"Paris"}"#);
}

#[tokio::test]
async fn preserves_http_error_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid auth"))
        .mount(&server)
        .await;

    let error = client(&server)
        .create_message(basic_request())
        .await
        .unwrap_err();

    match error {
        Error::Transport(TransportError::HttpStatus { status, body }) => {
            assert_eq!(status.as_u16(), 401);
            assert_eq!(body, "invalid auth");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn error_display_does_not_dump_raw_response_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_string("sensitive response body"))
        .mount(&server)
        .await;

    let error = client(&server)
        .create_message(basic_request())
        .await
        .unwrap_err();

    assert!(!error.to_string().contains("sensitive response body"));
}

#[tokio::test]
async fn stream_message_preserves_http_error_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid auth"))
        .mount(&server)
        .await;

    let error = client(&server)
        .stream_message(basic_request())
        .await
        .unwrap_err();

    match error {
        Error::Transport(TransportError::HttpStatus { status, body }) => {
            assert_eq!(status.as_u16(), 401);
            assert_eq!(body, "invalid auth");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn rejects_streaming_message_without_sse_content_type() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "not-a-stream"
        })))
        .mount(&server)
        .await;

    let error = client(&server)
        .stream_message(basic_request())
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        Error::Transport(TransportError::InvalidResponse(_))
    ));
}

#[tokio::test]
async fn lists_models_via_injected_http_client() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "type": "model",
                    "id": "claude-opus-5",
                    "display_name": "Claude Opus 5"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = AnthropicClient::builder()
        .api_key("test-key")
        .base_url(server.uri())
        .http_client(reqwest::Client::builder())
        .build()
        .unwrap();
    let response = client.list_models().await.unwrap();

    assert_eq!(response.data[0].id, "claude-opus-5");
}

#[tokio::test]
async fn invalid_utf8_body_maps_to_utf8_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(vec![0xFFu8, 0xFE, 0x00], "application/json"),
        )
        .mount(&server)
        .await;

    let error = client(&server).list_models().await.unwrap_err();

    assert!(
        matches!(error, Error::Transport(TransportError::Utf8(_))),
        "invalid-UTF-8 body must surface as TransportError::Utf8, got {error:?}"
    );
}

#[tokio::test]
async fn oversized_success_body_maps_to_body_too_large() {
    let server = MockServer::start().await;
    let big = vec![b'a'; 9 * 1024 * 1024]; // 9 MiB > 8 MiB cap
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(big, "application/json"))
        .mount(&server)
        .await;

    let error = client(&server).list_models().await.unwrap_err();

    assert!(
        matches!(error, Error::Transport(TransportError::BodyTooLarge { .. })),
        "oversized success body must surface as BodyTooLarge, got {error:?}"
    );
}

#[tokio::test]
async fn tool_calling_request_body_shapes() {
    let weather = Tool::new(
        "get_weather",
        json!({
            "type": "object",
            "properties": { "city": { "type": "string" } },
            "required": ["city"]
        }),
    )
    .with_description("Get the current weather for a city.");

    let request = CreateMessageRequest::new(
        "claude-opus-5",
        vec![
            MessageParam::user("What is the weather in Paris?"),
            MessageParam::user_blocks(vec![
                just_anthropic::types::message::ContentBlockParam::ToolResult {
                    tool_use_id: "toolu_1".to_owned(),
                    content: Some(just_anthropic::types::message::MessageContent::Text(
                        "25C and sunny".to_owned(),
                    )),
                    is_error: None,
                    cache_control: None,
                },
            ]),
        ],
        1024,
    )
    .with_tools(vec![weather])
    .with_tool_choice(ToolChoice::Any {
        disable_parallel_tool_use: None,
    })
    .with_thinking(ThinkingConfig::Enabled {
        budget_tokens: 2048,
        display: None,
    });

    let body = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["tools"][0]["name"], "get_weather");
    assert_eq!(parsed["tool_choice"]["type"], "any");
    assert_eq!(parsed["thinking"]["type"], "enabled");
    assert_eq!(parsed["thinking"]["budget_tokens"], 2048);
    assert_eq!(parsed["messages"][1]["content"][0]["type"], "tool_result");
    assert_eq!(
        parsed["messages"][1]["content"][0]["tool_use_id"],
        "toolu_1"
    );
}

#[tokio::test]
async fn deserializes_tool_use_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_2",
            "type": "message",
            "role": "assistant",
            "content": [
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
            "usage": { "input_tokens": 10, "output_tokens": 20 }
        })))
        .mount(&server)
        .await;

    let message = client(&server)
        .create_message(basic_request())
        .await
        .unwrap();

    assert_eq!(message.tool_calls().count(), 1);
    let call = message.tool_calls().next().unwrap();
    assert_eq!(call.name, "get_weather");
    assert_eq!(call.input, json!({ "city": "Paris" }));
    assert!(matches!(message.content[0], ContentBlock::ToolUse(_)));
}

#[test]
fn prepare_serializes_file_id_image_source() {
    let client = AnthropicClient::builder()
        .api_key("test-key")
        .base_url("https://api.anthropic.com/v1")
        .build()
        .unwrap();
    let request = CreateMessageRequest::new(
        "claude-opus-5",
        vec![MessageParam::user_blocks(vec![ContentBlockParam::Image {
            source: ImageSource::File {
                file_id: "file_abc123".to_string(),
            },
            cache_control: None,
        }])],
        256,
    );

    let prepared = client.prepare(request).unwrap();
    let body = prepared.body().and_then(|b| b.as_bytes()).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(body).unwrap();

    let block = &parsed["messages"][0]["content"][0];
    assert_eq!(block["type"], "image");
    assert_eq!(block["source"]["type"], "file");
    assert_eq!(block["source"]["file_id"], "file_abc123");
}
