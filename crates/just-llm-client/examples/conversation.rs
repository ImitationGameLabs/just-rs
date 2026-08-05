mod common;

use just_llm_client::{
    GenerationClient, GenerationClientOptions, LlmBackend, provider::OpenAiResponsesBackend,
    types::generation::Message,
};

/// Multi-turn conversation via the stateful [`Conversation`](just_llm_client::Conversation).
///
/// The caller owns the message list and passes the full logical context on every turn; the
/// `Conversation` sends only the new messages (plus `previous_response_id`) on Responses-family
/// backends when the turn is a pure append, and a full request otherwise.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let api_key = common::expect_env("JUST_LLM_OPENAI_RESPONSES_API_KEY");
    let model = common::expect_env("JUST_LLM_OPENAI_RESPONSES_MODEL");

    let backend = OpenAiResponsesBackend::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .use_rustls_tls(),
        &api_key,
        None,
    )?;

    let client = GenerationClient::new(
        backend,
        GenerationClientOptions::new(model).with_system_prompt("You are a concise assistant."),
    );
    let mut conv = client.conversation();
    println!("stateful chaining: {}", conv.is_stateful());

    // Turn 1: full request, stored server-side so the chain can continue.
    let first = conv
        .generate(client.create_request(vec![Message::user("Tell me a one-sentence joke.")]))
        .await?;
    println!("\n--- turn 1 ---");
    println!("  [assistant] {}", first.text().unwrap_or_default());

    // Turn 2: mirror the assistant turn and append a follow-up; only the delta is sent.
    let mirror = conv.last_message().expect("first turn produced a message");
    let second = conv
        .generate(client.create_request(vec![
            Message::user("Tell me a one-sentence joke."),
            mirror,
            Message::user("Now explain why it is funny."),
        ]))
        .await?;
    println!("\n--- turn 2 ---");
    println!("  [assistant] {}", second.text().unwrap_or_default());

    Ok(())
}
