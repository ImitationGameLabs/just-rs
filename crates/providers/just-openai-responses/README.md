# just-openai-responses

Wire-level types and a thin async client for the OpenAI **Responses API**.

## Overview

This crate provides serde-serializable request and response types that mirror the Responses API
shape (`POST /responses`, including SSE streaming, plus retrieve/cancel/compact/delete and model
listing), a thin async client (`ResponsesClient`), and re-exported HTTP transport helpers.

- **[`types`]** — Wire-level DTOs (request/response types with serde derives).
- **[`transport`]** — Re-exported HTTP helpers from `just-common` (`build_client`, `endpoint_url`,
  `ensure_success`, `parse_json`, `JsonEventStream`).

For a provider-neutral abstraction with capability traits and runtime provider selection, use
[**just-llm-client**].

[`types`]: https://docs.rs/just-openai-responses/latest/just_openai_responses/types
[`transport`]: https://docs.rs/just-openai-responses/latest/just_openai_responses/transport
[**just-llm-client**]: https://crates.io/crates/just-llm-client

## Quick start

### Non-streaming response

```rust
use just_openai_responses::ResponsesClient;
use just_openai_responses::types::request::{CreateResponseRequest, ResponseInput};

let client = ResponsesClient::builder()
    .api_key("your-api-key")
    .build()?;

let request = CreateResponseRequest::new("gpt-5.6")
    .with_input(ResponseInput::text("Say hello in one sentence."))
    .with_instructions("You are a concise assistant.");

let response = client.create_response(request).await?;
println!("{}", response.output_text());
```

### Streaming

```rust
use futures_util::StreamExt;
use just_openai_responses::types::event::StreamEvent;

let mut stream = client
    .stream_response(request)
    .await?;

while let Some(event) = stream.next().await {
    if let StreamEvent::ResponseOutputTextDelta { delta, .. } = event? {
        print!("{delta}");
    }
}
```

### Tool calling

Model-initiated function calls arrive as `function_call` output items carrying a `call_id`; feed
results back as `function_call_output` input items, preserving `reasoning` items too. See
`examples/responses_tool_calling.rs` for a full loop.

## Highlights

- **Wire-level DTOs** — Types mirror the Responses API shape with serde derives and
  forward-compatible `Unknown` fallback variants on every enum that deserializes server data.
- **All endpoints** — create, stream, retrieve (with `include`), cancel, compact, delete, and
  model listing.
- **Stateful conversations** — chain turns with `previous_response_id` or the `conversation`
  parameter.
- **Transport helpers** — Re-exported HTTP utilities from `just-common` for convenience.

> Looking for a provider-neutral interface with capability traits, runtime provider selection,
> and tool dispatch? Check out [**just-llm-client**].

## Examples

```bash
# Non-streaming
JUST_LLM_OPENAI_RESPONSES_API_KEY=your-key \
JUST_LLM_OPENAI_RESPONSES_MODEL=gpt-5.6 \
  cargo run -p just-openai-responses --example responses_chat_completion

# Streaming text deltas
JUST_LLM_OPENAI_RESPONSES_API_KEY=your-key \
JUST_LLM_OPENAI_RESPONSES_MODEL=gpt-5.6 \
  cargo run -p just-openai-responses --example responses_streaming

# Tool-calling loop
JUST_LLM_OPENAI_RESPONSES_API_KEY=your-key \
JUST_LLM_OPENAI_RESPONSES_MODEL=gpt-5.6 \
  cargo run -p just-openai-responses --example responses_tool_calling
```

## Scope notes

- Some types intentionally overlap with `just-openai-compat` (e.g. model listing, text
  configuration, tool choice) because the two crates are standalone wire-level type libraries
  for different OpenAI API surfaces. This duplication is accepted per the workspace's DTO
  layering convention.
- Unstable Responses API surfaces (computer use, shell, MCP, programmatic tool calling, etc.)
  are not yet modeled as first-class types; their items/tools/events parse into the `Unknown`
  fallback variant, and request parameters for them are omitted from `CreateResponseRequest`.
- WebSocket mode is not implemented yet; HTTP SSE streaming is fully supported.

## Ecosystem

| Crate                 | Description                                                |
| --------------------- | ---------------------------------------------------------- |
| [**just-llm-client**] | Provider-neutral LLM client — recommended entry point      |
| [just-deepseek]       | Wire-level DTOs and transport helpers for the DeepSeek API |
| [just-openai-compat]  | Wire-level DTOs for any OpenAI-compatible API              |
| [just-common]         | Shared HTTP transport, SSE parsing, and error types        |

[just-deepseek]: https://crates.io/crates/just-deepseek
[just-openai-compat]: https://crates.io/crates/just-openai-compat
[just-common]: https://crates.io/crates/just-common
