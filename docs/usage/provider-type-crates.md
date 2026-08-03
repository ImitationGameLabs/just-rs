# Provider type crates

The workspace includes three concrete provider type crates:

- `just-deepseek`
- `just-openai-compat`
- `just-openai-responses`

These crates provide wire-level DTOs, HTTP transport helpers, and thin client wrappers that closely
mirror the upstream provider API shapes.

If you want a provider-neutral abstraction, see [just-llm-client](just-llm-client.md).

## DeepSeek types

- `types::chat` — Chat completion request/response types, messages, tool definitions
- `types::models` — Model listing response types
- `types::balance` — Balance and quota response types

## OpenAI-compatible types

- `types::chat` — Chat completion request/response types, messages, tool definitions
- `types::models` — Model listing response types

## OpenAI Responses types

- `types::request` — Response creation and compaction request types
- `types::response` — The `Response` object, usage, and error details
- `types::item` — Input and output items (messages, function calls, reasoning, ...)
- `types::event` — SSE streaming events
- `types::tool` — Tool definitions (function, custom, web search, file search)
- `types::models` — Model listing response types

## Usage

```rust
use just_deepseek::types::chat::{ChatCompletionRequest, ChatMessage};

let request = ChatCompletionRequest::new(
    "deepseek-v4-pro",
    vec![
        ChatMessage::system("You are a concise assistant."),
        ChatMessage::user("Say hello in one sentence."),
    ],
);

let json = serde_json::to_string_pretty(&request)?;
println!("{json}");
```

Pair with `just-llm-client` for HTTP transport and provider-neutral abstractions.

## Environment variables for examples

```bash
# DeepSeek examples
JUST_LLM_DEEPSEEK_API_KEY=your-deepseek-api-key
#JUST_LLM_DEEPSEEK_BASE_URL=https://api.deepseek.com
JUST_LLM_DEEPSEEK_MODEL=deepseek-v4-flash

# OpenAI-compatible examples
JUST_LLM_OPENAI_COMPAT_API_KEY=your-openai-compatible-api-key
JUST_LLM_OPENAI_COMPAT_BASE_URL=https://your-compatible-endpoint/v1
JUST_LLM_OPENAI_COMPAT_MODEL=gpt-4.1-mini

# OpenAI Responses examples
JUST_LLM_OPENAI_RESPONSES_API_KEY=your-openai-api-key
#JUST_LLM_OPENAI_RESPONSES_BASE_URL=https://api.openai.com/v1
JUST_LLM_OPENAI_RESPONSES_MODEL=gpt-5.6
```

## Runnable examples

```bash
cargo run -p just-llm-client --example deepseek_simple_chat
cargo run -p just-llm-client --example openai_compat_simple_chat
cargo run -p just-openai-responses --example responses_chat_completion
cargo run -p just-openai-responses --example responses_streaming
cargo run -p just-openai-responses --example responses_tool_calling
```

The `deepseek_simple_chat` and `openai_compat_simple_chat` examples use `just-llm-client`
backends that internally serialize into the provider DTOs from these crates; the
`responses_*` examples use the `just-openai-responses` client directly.
