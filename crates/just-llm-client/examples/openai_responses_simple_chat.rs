mod common;

use just_llm_client::{
    LlmBackend,
    provider::OpenAiResponsesBackend,
    types::generation::{GenerationRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let api_key = common::expect_env("JUST_LLM_OPENAI_RESPONSES_API_KEY");
    let model = common::expect_env("JUST_LLM_OPENAI_RESPONSES_MODEL");
    let prompt = "Say hello in one sentence.";

    let backend = OpenAiResponsesBackend::new(
        reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .read_timeout(std::time::Duration::from_secs(120))
            .use_rustls_tls(),
        &api_key,
        None,
    )?;

    println!("--- request 1 ---");
    println!("  [system] You are a concise assistant.");
    println!("  [user] {prompt}");

    let response = backend
        .generate(
            GenerationRequest::new(model, vec![Message::user(prompt)])
                .with_system_prompt("You are a concise assistant."),
        )
        .await?;

    println!("\n--- response 1 ---");
    if let Some(text) = response.reasoning().and_then(|r| r.text.as_deref()) {
        println!("  [reasoning] {text}");
    }
    println!("  [assistant] {}", response.text().unwrap_or_default());
    Ok(())
}
