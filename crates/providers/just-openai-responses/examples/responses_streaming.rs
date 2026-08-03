//! Streaming a response and printing text deltas as they arrive.
//!
//! ```bash
//! JUST_LLM_OPENAI_RESPONSES_API_KEY=your-key \
//! JUST_LLM_OPENAI_RESPONSES_MODEL=gpt-5.6 \
//!   cargo run -p just-openai-responses --example responses_streaming
//! ```

use futures_util::StreamExt;
use just_openai_responses::ResponsesClient;
use just_openai_responses::types::{
    event::StreamEvent,
    request::{CreateResponseRequest, ResponseInput},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let api_key = std::env::var("JUST_LLM_OPENAI_RESPONSES_API_KEY")
        .expect("JUST_LLM_OPENAI_RESPONSES_API_KEY must be set");
    let model = std::env::var("JUST_LLM_OPENAI_RESPONSES_MODEL")
        .expect("JUST_LLM_OPENAI_RESPONSES_MODEL must be set");

    let client = ResponsesClient::builder().api_key(&api_key).build()?;

    let request = CreateResponseRequest::new(&model)
        .with_input(ResponseInput::text("Write a short haiku about the sea."));

    let mut stream = client.stream_response(request).await?;
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ResponseOutputTextDelta { delta, .. } => print!("{delta}"),
            StreamEvent::ResponseCompleted { .. } => break,
            StreamEvent::Error { message, .. } => {
                eprintln!("stream error: {message}");
                break;
            }
            _ => {}
        }
    }
    println!();
    Ok(())
}
