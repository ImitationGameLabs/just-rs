//! Single-turn non-streaming message exchange.
//!
//! ```bash
//! JUST_LLM_ANTHROPIC_API_KEY=your-key \
//! JUST_LLM_ANTHROPIC_MODEL=claude-opus-5 \
//!   cargo run -p just-anthropic --example anthropic_chat
//! ```

use just_anthropic::AnthropicClient;
use just_anthropic::types::{
    message::MessageParam,
    request::{CreateMessageRequest, SystemParam},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let api_key = std::env::var("JUST_LLM_ANTHROPIC_API_KEY")
        .expect("JUST_LLM_ANTHROPIC_API_KEY must be set");
    let model =
        std::env::var("JUST_LLM_ANTHROPIC_MODEL").expect("JUST_LLM_ANTHROPIC_MODEL must be set");

    let client = AnthropicClient::builder().api_key(&api_key).build()?;

    let request = CreateMessageRequest::new(
        &model,
        vec![MessageParam::user("Say hello in one sentence.")],
        256,
    )
    .with_system(SystemParam::Text("You are a concise assistant.".to_owned()));

    let message = client.create_message(request).await?;
    println!("{}", message.text());
    println!("stop_reason: {:?}", message.stop_reason);

    Ok(())
}
