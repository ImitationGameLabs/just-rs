mod common;

use just_llm_client::{
    LlmBackend,
    provider::DeepSeekBackend,
    types::generation::{FunctionDefinition, GenerationRequest, Message, ToolDefinition, ToolType},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let api_key = common::expect_env("JUST_LLM_DEEPSEEK_API_KEY");
    let base_url = std::env::var("JUST_LLM_DEEPSEEK_BASE_URL").ok();
    let model = common::expect_env("JUST_LLM_DEEPSEEK_MODEL");

    let backend = DeepSeekBackend::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .use_rustls_tls(),
        &api_key,
        base_url.as_deref(),
    )?;

    let tools = vec![ToolDefinition {
        kind: ToolType::Function,
        function: FunctionDefinition {
            name: "sum".to_owned(),
            description: Some("Add two numbers together.".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "The first number." },
                    "y": { "type": "number", "description": "The second number." }
                },
                "required": ["x", "y"]
            })),
            strict: None,
        },
    }];

    let request = GenerationRequest::new(model, vec![Message::user("What is 12345 + 67890?")])
        .with_system_prompt("You are a helpful math assistant. Use the provided tools.")
        .with_tools(tools);

    println!("--- request 1 ---");
    println!("  [system] You are a helpful math assistant. Use the provided tools.");
    println!("  [user] What is 12345 + 67890?");

    let response = backend.generate(request).await?;
    println!("\n--- response 1 ---");
    if let Some(text) = response.reasoning().and_then(|r| r.text.as_deref()) {
        println!("  [reasoning] {text}");
    }
    let response_model = response.model.clone();
    let reasoning = response.reasoning().cloned();

    let tool_calls = response.tool_calls();
    let call = &tool_calls[0];
    println!("  [tool call] {}({})", call.name, call.arguments);

    // Execute the tool locally.
    let args: serde_json::Value = serde_json::from_str(&call.arguments)?;
    let x: f64 = args["x"].as_f64().expect("x is not a number");
    let y: f64 = args["y"].as_f64().expect("y is not a number");
    let result = x + y;
    println!("\n--- request 2 ---");
    println!("  [tool result] {x} + {y} = {result}");

    // Build the assistant message, preserving reasoning content for DeepSeek thinking mode.
    let assistant_msg = Message::assistant_tool_calls(None, tool_calls.to_vec(), reasoning);

    // Send the tool result back for a final answer.
    let follow_up = GenerationRequest::new(
        response_model,
        vec![
            Message::user("What is 12345 + 67890?"),
            assistant_msg,
            Message::tool(serde_json::json!({"result": result}).to_string(), &call.id),
        ],
    )
    .with_system_prompt("You are a helpful math assistant. Use the provided tools.")
    .with_tools(vec![ToolDefinition {
        kind: ToolType::Function,
        function: FunctionDefinition {
            name: "sum".to_owned(),
            description: Some("Add two numbers together.".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "The first number." },
                    "y": { "type": "number", "description": "The second number." }
                },
                "required": ["x", "y"]
            })),
            strict: None,
        },
    }]);

    println!("\n--- response 2 ---");
    let final_response = backend.generate(follow_up).await?;
    if let Some(text) = final_response.reasoning().and_then(|r| r.text.as_deref()) {
        println!("  [reasoning] {text}");
    }
    println!(
        "  [assistant] {}",
        final_response.text().unwrap_or_default()
    );

    Ok(())
}
