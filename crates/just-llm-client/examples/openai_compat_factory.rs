//! Backend factory initialization.
//!
//! Demonstrates building a [`BackendFactory`] by hand and obtaining a [`GenerationClient`] from it.

mod common;

use just_llm_client::{
    BackendFactory, GenerationClient, GenerationClientOptions, family,
    provider::OpenAiCompatBackend, types::generation::Message,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let api_key = common::expect_env("JUST_LLM_OPENAI_COMPAT_API_KEY");
    let base_url = common::expect_env("JUST_LLM_OPENAI_COMPAT_BASE_URL");
    let model = common::expect_env("JUST_LLM_OPENAI_COMPAT_MODEL");
    let prompt = "Say hello in one sentence.";

    // Build an empty factory and register the OpenAI-compatible backend explicitly.
    // (`BackendFactory::new()` would already pre-seed it under default features; this shows the
    // explicit-registration escape hatch and the centralized family constant.)
    let mut factory = BackendFactory::empty();
    factory.register::<OpenAiCompatBackend>();
    let backend = factory.create(
        family::OPENAI_COMPATIBLE,
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .use_rustls_tls(),
        &api_key,
        Some(&base_url),
    )?;
    let client = GenerationClient::new(
        backend,
        GenerationClientOptions::new(model).with_system_prompt("You are a concise assistant."),
    );

    println!("--- request 1 ---");
    println!("  [family] {}", client.family());
    println!("  [model] {}", client.model());
    println!("  [system] You are a concise assistant.");
    println!("  [user] {prompt}");

    let request = client.create_request(vec![Message::user(prompt)]);
    let response = client.generate(request).await?;

    println!("\n--- response 1 ---");
    if let Some(text) = response.reasoning().and_then(|r| r.text.as_deref()) {
        println!("  [reasoning] {text}");
    }
    println!("  [assistant] {}", response.text().unwrap_or_default());
    Ok(())
}
