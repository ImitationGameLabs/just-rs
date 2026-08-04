# just-anthropic

Wire-level types and a thin async client for the Anthropic **Messages API**.

## Overview

This crate provides serde-serializable request and response types that mirror the Messages API
shape (`POST /v1/messages`, including SSE streaming, plus token counting and model listing), a
thin async client (`AnthropicClient`), and re-exported HTTP transport helpers.

- **[`types`]** — Wire-level DTOs (request/response types with serde derives).
- **[`transport`]** — Re-exported HTTP helpers from `just-common` (`build_client_with_headers`,
  `endpoint_url`, `ensure_success`, `parse_json`, `JsonEventStream`).

For a provider-neutral abstraction with capability traits and runtime provider selection, use
[**just-llm-client**].

[`types`]: https://docs.rs/just-anthropic/latest/just_anthropic/types
[`transport`]: https://docs.rs/just-anthropic/latest/just_anthropic/transport
[**just-llm-client**]: https://crates.io/crates/just-llm-client

## Quick start

### Non-streaming message

```rust
use just_anthropic::AnthropicClient;
use just_anthropic::types::{
    message::MessageParam,
    request::CreateMessageRequest,
};

let client = AnthropicClient::builder()
    .api_key("your-api-key")
    .build()?;

let request = CreateMessageRequest::new(
    "claude-opus-5",
    vec![MessageParam::user("Say hello in one sentence.")],
    256,
);

let message = client.create_message(request).await?;
println!("{}", message.text());
```

### Streaming

```rust
use futures_util::StreamExt;
use just_anthropic::types::event::{ContentBlockDelta, StreamEvent};

let mut stream = client
    .stream_message(request)
    .await?;

while let Some(event) = stream.next().await {
    if let StreamEvent::ContentBlockDelta {
        delta: ContentBlockDelta::TextDelta { text },
        ..
    } = event? {
        print!("{text}");
    }
}
```

### Tool calling

Anthropic returns tool invocations as `tool_use` content blocks on an assistant message; feed
results back as a `user` message carrying `tool_result` blocks. Extended-thinking blocks (with
their signature) should be preserved across turns for multi-turn continuity. See
`examples/anthropic_tool_calling.rs` for a full loop.

## Highlights

- **Wire-level DTOs** — Types mirror the Messages API shape with serde derives and
  forward-compatible `Unknown` fallback variants on every enum that deserializes server data.
- **All core endpoints** — create, stream, token counting, and model listing.
- **Extended thinking** — `thinking` config on requests; `thinking`/`redacted_thinking` blocks
  and `thinking_delta`/`signature_delta` events on responses.
- **Prompt caching** — `cache_control` breakpoints on cacheable blocks; cache token counts in
  `Usage`.
- **Transport helpers** — Re-exported HTTP utilities from `just-common` for convenience.

> Looking for a provider-neutral interface with capability traits, runtime provider selection,
> and tool dispatch? Check out [**just-llm-client**].

## Examples

```bash
# Non-streaming
JUST_LLM_ANTHROPIC_API_KEY=your-key \
JUST_LLM_ANTHROPIC_MODEL=claude-opus-5 \
  cargo run -p just-anthropic --example anthropic_chat

# Streaming text deltas (and thinking)
JUST_LLM_ANTHROPIC_API_KEY=your-key \
JUST_LLM_ANTHROPIC_MODEL=claude-opus-5 \
  cargo run -p just-anthropic --example anthropic_streaming

# Tool-calling loop
JUST_LLM_ANTHROPIC_API_KEY=your-key \
JUST_LLM_ANTHROPIC_MODEL=claude-opus-5 \
  cargo run -p just-anthropic --example anthropic_tool_calling
```

## Scope notes

- Authentication uses the `x-api-key` header plus `anthropic-version` (default `2023-06-01`);
  both are injected by the builder and can be overridden (`api_version`) or replaced via
  `http_client(reqwest::ClientBuilder)`.
- Server tools (web search, code execution, bash, text editor, etc.) are not yet modeled as
  first-class types; their content blocks parse into the `Unknown` fallback variant, and request
  parameters for them are omitted from the request types.
- The Message Batches API is not implemented yet; token counting is fully supported.
- Document blocks and citations are not yet modeled; they parse into the `Unknown` fallback
  variant.

## Ecosystem

| Crate                 | Description                                                |
| --------------------- | ---------------------------------------------------------- |
| [**just-llm-client**] | Provider-neutral LLM client — recommended entry point      |
| [just-deepseek]       | Wire-level DTOs and transport helpers for the DeepSeek API |
| [just-openai-compat]  | Wire-level DTOs for any OpenAI-compatible API              |
| [just-openai-responses] | Wire-level DTOs for the OpenAI Responses API             |
| [just-common]         | Shared HTTP transport, SSE parsing, and error types        |

[just-deepseek]: https://crates.io/crates/just-deepseek
[just-openai-compat]: https://crates.io/crates/just-openai-compat
[just-openai-responses]: https://crates.io/crates/just-openai-responses
[just-common]: https://crates.io/crates/just-common
