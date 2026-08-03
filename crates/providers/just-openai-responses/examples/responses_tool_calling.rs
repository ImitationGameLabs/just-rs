//! Tool-calling loop: run model-initiated function calls and feed results back.
//!
//! ```bash
//! JUST_LLM_OPENAI_RESPONSES_API_KEY=your-key \
//! JUST_LLM_OPENAI_RESPONSES_MODEL=gpt-5.6 \
//!   cargo run -p just-openai-responses --example responses_tool_calling
//! ```

use serde_json::json;

use just_openai_responses::ResponsesClient;
use just_openai_responses::types::{
    item::{FunctionCallOutput, InputItem, OutputItem},
    message::InputMessage,
    request::{CreateResponseRequest, ResponseInput},
    tool::ResponseTool,
};

fn weather_tool() -> ResponseTool {
    ResponseTool::function(
        "get_weather",
        "Get the current weather for a city.",
        json!({
            "type": "object",
            "properties": {
                "city": { "type": "string", "description": "City name" }
            },
            "required": ["city"],
            "additionalProperties": false
        }),
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let api_key = std::env::var("JUST_LLM_OPENAI_RESPONSES_API_KEY")
        .expect("JUST_LLM_OPENAI_RESPONSES_API_KEY must be set");
    let model = std::env::var("JUST_LLM_OPENAI_RESPONSES_MODEL")
        .expect("JUST_LLM_OPENAI_RESPONSES_MODEL must be set");

    let client = ResponsesClient::builder().api_key(&api_key).build()?;

    let mut input: Vec<InputItem> = vec![InputItem::Message(InputMessage::user(
        "What is the weather in Paris?",
    ))];

    loop {
        let request = CreateResponseRequest::new(&model)
            .with_input(ResponseInput::items(input.clone()))
            .with_tools(vec![weather_tool()]);

        let response = client.create_response(request).await?;

        // Preserve model output (including reasoning items) for the next turn.
        input.extend(
            response
                .output
                .iter()
                .cloned()
                .filter_map(|item| match item {
                    OutputItem::FunctionCall(call) => Some(InputItem::FunctionCall(call)),
                    OutputItem::Reasoning(reasoning) => Some(InputItem::Reasoning(reasoning)),
                    _ => None,
                }),
        );

        let calls: Vec<_> = response.function_calls().cloned().collect();
        if calls.is_empty() {
            println!("{}", response.output_text());
            break;
        }

        for call in &calls {
            let city: String =
                serde_json::from_str(&call.arguments).expect("arguments must parse as JSON");
            let result = format!("The weather in {city} is 25C and sunny.");
            input.push(InputItem::FunctionCallOutput(FunctionCallOutput::new(
                call.call_id.clone(),
                result,
            )));
        }
    }

    Ok(())
}
