//! Tool-calling loop: run model-initiated function calls and feed results back.
//!
//! Anthropic returns tool invocations as `tool_use` content blocks on an assistant message;
//! results are fed back as a `user` message whose content is `tool_result` blocks.
//! Extended-thinking blocks (with their signature) are preserved across turns.
//!
//! ```bash
//! JUST_LLM_ANTHROPIC_API_KEY=your-key \
//! JUST_LLM_ANTHROPIC_MODEL=claude-opus-5 \
//!   cargo run -p just-anthropic --example anthropic_tool_calling
//! ```

use serde_json::json;

use just_anthropic::AnthropicClient;
use just_anthropic::types::{
    message::{ContentBlock, ContentBlockParam, MessageContent, MessageParam},
    request::CreateMessageRequest,
    tool::Tool,
};

fn weather_tool() -> Tool {
    Tool::new(
        "get_weather",
        json!({
            "type": "object",
            "properties": {
                "city": { "type": "string", "description": "City name" }
            },
            "required": ["city"],
            "additionalProperties": false
        }),
    )
    .with_description("Get the current weather for a city.")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let api_key = std::env::var("JUST_LLM_ANTHROPIC_API_KEY")
        .expect("JUST_LLM_ANTHROPIC_API_KEY must be set");
    let model =
        std::env::var("JUST_LLM_ANTHROPIC_MODEL").expect("JUST_LLM_ANTHROPIC_MODEL must be set");

    let client = AnthropicClient::builder().api_key(&api_key).build()?;

    let mut history = vec![MessageParam::user("What is the weather in Paris?")];

    loop {
        let request = CreateMessageRequest::new(&model, history.clone(), 1024)
            .with_tools(vec![weather_tool()]);

        let message = client.create_message(request).await?;

        let calls: Vec<_> = message.tool_calls().cloned().collect();
        if calls.is_empty() {
            println!("{}", message.text());
            break;
        }

        // Preserve the assistant message (including thinking blocks and signatures).
        history.push(MessageParam::assistant_blocks(
            message
                .content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Thinking(thinking) => Some(ContentBlockParam::Thinking {
                        thinking: thinking.thinking.clone(),
                        signature: thinking.signature.clone(),
                    }),
                    ContentBlock::RedactedThinking { data } => {
                        Some(ContentBlockParam::RedactedThinking { data: data.clone() })
                    }
                    ContentBlock::ToolUse(call) => Some(ContentBlockParam::ToolUse {
                        id: call.id.clone(),
                        name: call.name.clone(),
                        input: call.input.clone(),
                        cache_control: None,
                    }),
                    _ => None,
                })
                .collect(),
        ));

        // Feed tool results back as a user message with tool_result blocks.
        let results = calls
            .iter()
            .map(|call| {
                let city: String =
                    serde_json::from_value(call.input["city"].clone()).unwrap_or_default();
                let output = format!("The weather in {city} is 25C and sunny.");
                ContentBlockParam::ToolResult {
                    tool_use_id: call.id.clone(),
                    content: Some(MessageContent::Text(output)),
                    is_error: None,
                    cache_control: None,
                }
            })
            .collect();
        history.push(MessageParam::user_blocks(results));
    }

    Ok(())
}
