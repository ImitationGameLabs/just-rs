use futures_util::StreamExt;
use just_common::error::TransportError;
use just_openai_responses::{
    Error, ResponsesClient,
    types::{
        event::StreamEvent,
        request::{CompactRequest, CreateResponseRequest, ResponseInput},
        shared::ResponseIncludable,
    },
};
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, header, method, path},
};

fn client(server: &MockServer) -> ResponsesClient {
    ResponsesClient::builder()
        .api_key("test-key")
        .base_url(server.uri())
        .build()
        .unwrap()
}

fn basic_request() -> CreateResponseRequest {
    CreateResponseRequest::new("gpt-5.6")
        .with_input(ResponseInput::text("Say hello."))
        .with_instructions("Be concise.")
}

fn response_body() -> serde_json::Value {
    json!({
        "id": "resp_1",
        "object": "response",
        "created_at": 1,
        "status": "completed",
        "model": "gpt-5.6",
        "output": [
            {
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "status": "completed",
                "content": [
                    { "type": "output_text", "text": "Hello!", "annotations": [] }
                ]
            }
        ],
        "usage": {
            "input_tokens": 10,
            "input_tokens_details": { "cache_write_tokens": 0, "cached_tokens": 0 },
            "output_tokens": 5,
            "output_tokens_details": { "reasoning_tokens": 0 },
            "total_tokens": 15
        }
    })
}

#[tokio::test]
async fn creates_non_streaming_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(header("authorization", "Bearer test-key"))
        .and(body_partial_json(json!({
            "model": "gpt-5.6",
            "input": "Say hello.",
            "instructions": "Be concise."
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body()))
        .mount(&server)
        .await;

    let response = client(&server)
        .create_response(basic_request())
        .await
        .unwrap();

    assert_eq!(response.id, "resp_1");
    assert_eq!(response.output_text(), "Hello!");
}

#[tokio::test]
async fn rejects_stream_flag_on_non_stream_method() {
    let request = CreateResponseRequest {
        stream: Some(true),
        ..basic_request()
    };

    let error = ResponsesClient::builder()
        .api_key("test-key")
        .base_url("http://127.0.0.1:0")
        .build()
        .unwrap()
        .create_response(request)
        .await
        .unwrap_err();

    assert!(matches!(error, Error::InvalidRequest(_)));
}

#[tokio::test]
async fn streams_response_events() {
    let server = MockServer::start().await;
    let body = concat!(
        "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\",\"object\":\"response\",\"created_at\":1,\"status\":\"in_progress\",\"model\":\"gpt-5.6\",\"output\":[]}}\n\n",
        "data: {\"type\":\"response.output_text.delta\",\"item_id\":\"msg_1\",\"output_index\":0,\"content_index\":0,\"delta\":\"Hel\"}\n\n",
        "data: {\"type\":\"response.output_text.delta\",\"item_id\":\"msg_1\",\"output_index\":0,\"content_index\":0,\"delta\":\"lo\"}\n\n",
        "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_1\",\"object\":\"response\",\"created_at\":1,\"status\":\"completed\",\"model\":\"gpt-5.6\",\"output\":[]}}\n\n",
        "data: [DONE]\n\n"
    );

    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let mut stream = client(&server)
        .stream_response(basic_request())
        .await
        .unwrap();

    let created = stream.next().await.unwrap().unwrap();
    assert!(matches!(created, StreamEvent::ResponseCreated { .. }));

    let delta = stream.next().await.unwrap().unwrap();
    assert!(matches!(
        delta,
        StreamEvent::ResponseOutputTextDelta { delta, .. } if delta == "Hel"
    ));

    let delta = stream.next().await.unwrap().unwrap();
    assert!(matches!(
        delta,
        StreamEvent::ResponseOutputTextDelta { delta, .. } if delta == "lo"
    ));

    let completed = stream.next().await.unwrap().unwrap();
    assert!(matches!(completed, StreamEvent::ResponseCompleted { .. }));

    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn unknown_stream_event_does_not_break_stream() {
    let server = MockServer::start().await;
    let body = concat!(
        "data: {\"type\":\"some.new.event\",\"payload\":true}\n\n",
        "data: {\"type\":\"response.output_text.delta\",\"item_id\":\"msg_1\",\"output_index\":0,\"content_index\":0,\"delta\":\"Hi\"}\n\n",
        "data: [DONE]\n\n"
    );

    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let mut stream = client(&server)
        .stream_response(basic_request())
        .await
        .unwrap();

    let unknown = stream.next().await.unwrap().unwrap();
    assert!(matches!(unknown, StreamEvent::Unknown));

    let delta = stream.next().await.unwrap().unwrap();
    assert!(matches!(
        delta,
        StreamEvent::ResponseOutputTextDelta { delta, .. } if delta == "Hi"
    ));

    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn retrieves_response_with_include() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/responses/resp_1"))
        .and(header("authorization", "Bearer test-key"))
        .and(|request: &wiremock::Request| {
            request.url.query().is_some_and(|q| {
                q.contains("include=reasoning.encrypted_content")
                    && q.contains("include=file_search_call.results")
            })
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body()))
        .mount(&server)
        .await;

    let response = client(&server)
        .retrieve_response(
            "resp_1",
            Some(&[
                ResponseIncludable::ReasoningEncryptedContent,
                ResponseIncludable::FileSearchCallResults,
            ]),
            None,
        )
        .await
        .unwrap();

    assert_eq!(response.id, "resp_1");
}

#[tokio::test]
async fn cancels_response_without_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses/resp_1/cancel"))
        .and(header("authorization", "Bearer test-key"))
        .and(|request: &wiremock::Request| request.body.is_empty())
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body()))
        .mount(&server)
        .await;

    let response = client(&server).cancel_response("resp_1").await.unwrap();

    assert_eq!(response.id, "resp_1");
}

#[tokio::test]
async fn compacts_response_chain() {
    let server = MockServer::start().await;
    let compacted = json!({
        "id": "resp_compacted",
        "object": "response.compaction",
        "created_at": 1,
        "output": [
            {
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "status": "completed",
                "content": [
                    { "type": "output_text", "text": "summarized", "annotations": [] }
                ]
            }
        ],
        "usage": {
            "input_tokens": 5,
            "input_tokens_details": { "cache_write_tokens": 0, "cached_tokens": 0 },
            "output_tokens": 3,
            "output_tokens_details": { "reasoning_tokens": 0 },
            "total_tokens": 8
        }
    });

    Mock::given(method("POST"))
        .and(path("/responses/compact"))
        .and(body_partial_json(json!({
            "model": "gpt-5.6",
            "previous_response_id": "resp_1"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(compacted))
        .mount(&server)
        .await;

    let request = CompactRequest {
        previous_response_id: Some("resp_1".to_owned()),
        ..CompactRequest::new("gpt-5.6")
    };
    let response = client(&server).compact_response(request).await.unwrap();

    assert_eq!(response.object, "response.compaction");
    assert_eq!(response.output.len(), 1);
    assert_eq!(response.usage.total_tokens, 8);
}

#[tokio::test]
async fn deletes_response() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/responses/resp_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp_1",
            "object": "response",
            "deleted": true
        })))
        .mount(&server)
        .await;

    let response = client(&server).delete_response("resp_1").await.unwrap();

    assert!(response.deleted);
}

#[tokio::test]
async fn lists_models() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [
                {
                    "id": "gpt-5.6",
                    "object": "model",
                    "owned_by": "openai"
                }
            ]
        })))
        .mount(&server)
        .await;

    let response = client(&server).list_models().await.unwrap();

    assert_eq!(response.data[0].id, "gpt-5.6");
}

#[tokio::test]
async fn preserves_http_error_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid auth"))
        .mount(&server)
        .await;

    let error = client(&server)
        .create_response(basic_request())
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
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(401).set_body_string("sensitive response body"))
        .mount(&server)
        .await;

    let error = client(&server)
        .create_response(basic_request())
        .await
        .unwrap_err();

    assert!(!error.to_string().contains("sensitive response body"));
}

#[tokio::test]
async fn stream_response_preserves_http_error_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid auth"))
        .mount(&server)
        .await;

    let error = client(&server)
        .stream_response(basic_request())
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
async fn rejects_streaming_response_without_sse_content_type() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "not-a-stream"
        })))
        .mount(&server)
        .await;

    let error = client(&server)
        .stream_response(basic_request())
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
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [
                {
                    "id": "gpt-5.6",
                    "object": "model",
                    "owned_by": "openai"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = ResponsesClient::builder()
        .api_key("test-key")
        .base_url(server.uri())
        .http_client(reqwest::Client::builder())
        .build()
        .unwrap();
    let response = client.list_models().await.unwrap();

    assert_eq!(response.data[0].id, "gpt-5.6");
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
