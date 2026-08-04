# just-llm-client

Just a lightweight, composable, and minimal LLM client — not an agent framework.

## Quick start

```toml
# Cargo.toml
[dependencies]
just-llm-client = "0.2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust,no_run
use just_llm_client::{
    LlmBackend,
    provider::DeepSeekBackend,
    types::generation::{GenerationRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = DeepSeekBackend::new(
        reqwest::Client::builder().use_rustls_tls(),
        "your-api-key",
        None,
    )?;

    let response = backend
        .generate(
            GenerationRequest::new(
                "deepseek-chat",
                vec![Message::user("Say hello in one sentence.")],
            )
            .with_system_prompt("You are a concise assistant."),
        )
        .await?;

    println!("{}", response.text().unwrap_or_default());
    Ok(())
}
```

## Feature flags

| Feature         | Default | Description                                     |
| --------------- | ------- | ----------------------------------------------- |
| `deepseek`      | yes     | Enables the [`just-deepseek`] backend           |
| `openai-compat` | yes     | Enables the [`just-openai-compat`] backend      |
| `responses`     | no      | Enables the [`just-openai-responses`] backend   |
| `anthropic`     | no      | Enables the [`just-anthropic`] backend          |

The DeepSeek and OpenAI-compatible backends are enabled by default. Disable default features and
enable only what you need:

```toml
[dependencies]
just-llm-client = { version = "0.2", default-features = false, features = ["deepseek"] }
```

## Ecosystem

| Crate                | Description                                     |
| -------------------- | ----------------------------------------------- |
| [just-deepseek]      | DeepSeek API client + wire-level types          |
| [just-openai-compat] | OpenAI-compatible API client + wire-level types |
| [just-openai-responses] | OpenAI Responses API client + wire-level types |
| [just-anthropic]     | Anthropic Messages API client + wire-level types |
| [just-common]        | Shared HTTP transport and error types           |

[just-deepseek]: https://crates.io/crates/just-deepseek
[just-openai-compat]: https://crates.io/crates/just-openai-compat
[just-openai-responses]: https://crates.io/crates/just-openai-responses
[just-anthropic]: https://crates.io/crates/just-anthropic
[just-common]: https://crates.io/crates/just-common
