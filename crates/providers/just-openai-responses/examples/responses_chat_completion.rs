//! Non-streaming response creation using `ResponsesClient`.
//!
//! ```bash
//! JUST_LLM_OPENAI_RESPONSES_API_KEY=your-key \
//! JUST_LLM_OPENAI_RESPONSES_MODEL=gpt-5.6 \
//!   cargo run -p just-openai-responses --example responses_chat_completion
//! ```

use just_openai_responses::ResponsesClient;
use just_openai_responses::types::request::{CreateResponseRequest, ResponseInput};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let api_key = std::env::var("JUST_LLM_OPENAI_RESPONSES_API_KEY")
        .expect("JUST_LLM_OPENAI_RESPONSES_API_KEY must be set");
    let model = std::env::var("JUST_LLM_OPENAI_RESPONSES_MODEL")
        .expect("JUST_LLM_OPENAI_RESPONSES_MODEL must be set");

    let client = ResponsesClient::builder().api_key(&api_key).build()?;

    let request = CreateResponseRequest::new(&model)
        .with_input(ResponseInput::text("Say hello in one sentence."))
        .with_instructions("You are a concise assistant.");

    let response = client.create_response(request).await?;

    println!("{}", response.output_text());
    Ok(())
}
