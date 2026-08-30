//! Multimodal chat over the openai-compat wire: one user message carrying a text
//! part, a base64 image part built from an in-repo fixture, and a remote-URL
//! image part.
//!
//! Requires the `openai-compat` feature and a vision-capable endpoint behind the
//! `JUST_LLM_OPENAI_COMPAT_*` environment variables.

mod common;

use just_llm_client::{
    BackendFactory, captured_body, provider_rejection,
    types::generation::{ContentPart, GenerationRequest, ImageDetail, ImageSource, Message},
};

/// 8x8 solid-red PNG shipped next to this example, so the base64 part needs no
/// download and the example stays reproducible.
const RED_PNG: &[u8] = include_bytes!("assets/red_8x8.png");

/// Minimal standard base64 encoder, so the example can turn the fixture into the
/// wire-format string without adding a `base64` dependency to the crate manifest.
fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        for shift in [18, 12, 6, 0] {
            out.push(ALPHABET[((n >> shift) & 0x3f) as usize] as char);
        }
    }
    match bytes.len() % 3 {
        1 => {
            out.truncate(out.len() - 2);
            out.push_str("==");
        }
        2 => {
            out.truncate(out.len() - 1);
            out.push('=');
        }
        _ => {}
    }
    out
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("failed to load .env file");

    let api_key = common::expect_env("JUST_LLM_OPENAI_COMPAT_API_KEY");
    let base_url = common::expect_env("JUST_LLM_OPENAI_COMPAT_BASE_URL");
    let model = common::expect_env("JUST_LLM_OPENAI_COMPAT_MODEL");

    let http = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .read_timeout(std::time::Duration::from_secs(120))
        .use_rustls_tls();
    let backend = BackendFactory::new().create(
        "openai-compatible",
        http,
        &api_key,
        Some(base_url.as_str()),
    )?;

    let prompt = "Describe each image in one short sentence.";
    println!("--- request ---");
    println!("  [text] {prompt}");
    println!("  [image 1] 8x8 red png (base64, detail: low)");
    println!("  [image 2] https://www.python.org/static/img/python-logo.png");

    let message = Message::user_parts(vec![
        ContentPart::Text {
            text: prompt.to_string(),
        },
        ContentPart::Image {
            source: ImageSource::Base64 {
                data: base64_encode(RED_PNG),
                media_type: "image/png".to_string(),
            },
            detail: Some(ImageDetail::Low),
        },
        ContentPart::Image {
            source: ImageSource::Url {
                url: "https://www.python.org/static/img/python-logo.png".to_string(),
            },
            detail: None,
        },
    ]);

    let prepared = backend.prepare(GenerationRequest::new(model, vec![message]))?;
    let raw = backend.send(prepared).await?;
    println!("\n--- raw response ---");
    println!(
        "  status={} content-type={:?}",
        raw.status(),
        raw.headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
    );

    println!("\n--- parsed ---");
    match backend.parse(raw).await {
        Ok(response) => println!("  [assistant] {}", response.text().unwrap_or_default()),
        Err(err) => {
            eprintln!("  [error] {err}");
            if let Some(rejection) = provider_rejection(&err) {
                eprintln!(
                    "  [rejection] status={} code={:?} error_type={:?}",
                    rejection.status, rejection.code, rejection.error_type
                );
            }
            if let Some(body) = captured_body(&err) {
                eprintln!("  [body] {body}");
            }
            return Err(err.into());
        }
    }
    Ok(())
}
