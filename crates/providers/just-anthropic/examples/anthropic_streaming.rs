//! Streaming text deltas (and thinking, if enabled).
//!
//! ```bash
//! JUST_LLM_ANTHROPIC_API_KEY=your-key \
//! JUST_LLM_ANTHROPIC_MODEL=claude-opus-5 \
//!   cargo run -p just-anthropic --example anthropic_streaming
//! ```

use futures_util::StreamExt;

use just_anthropic::AnthropicClient;
use just_anthropic::types::{
    event::{ContentBlockDelta, StreamEvent},
    message::MessageParam,
    request::CreateMessageRequest,
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
        vec![MessageParam::user("Count from one to five.")],
        1024,
    );

    let mut stream = client.stream_message(request).await?;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::TextDelta { text },
                ..
            } => print!("{text}"),
            StreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::ThinkingDelta { thinking },
                ..
            } => print!("[thinking] {thinking}"),
            StreamEvent::MessageDelta { delta, .. } => {
                if let Some(reason) = delta.stop_reason {
                    println!("\nstop_reason: {reason:?}");
                }
            }
            _ => {}
        }
    }

    Ok(())
}
