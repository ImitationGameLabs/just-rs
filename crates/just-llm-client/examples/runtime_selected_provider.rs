//! Runtime provider selection from environment variables.
//!
//! Requires both `deepseek` and `openai-compat` features.

mod common;

use just_llm_client::types::generation::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let client = common::client_from_env("You are a concise assistant.")?;
    let prompt = "Say hello in one sentence.";

    println!("--- request 1 ---");
    println!("  [family] {}", client.family());
    println!("  [model] {}", client.model());
    println!("  [system] {}", client.system_prompt().unwrap_or(""),);
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
