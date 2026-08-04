//! Anthropic Messages API provider SDK.
//!
//! This crate exposes a thin Rust client and wire-level DTOs for Anthropic's Messages API
//! (`POST /v1/messages` with SSE streaming, plus token counting and model listing). Request and
//! response types under [`types`] mirror the wire format; the client under [`AnthropicClient`]
//! handles the HTTP lifecycle with the same prepare/send/parse pattern used across the
//! workspace's provider crates.
//!
//! Authentication uses the `x-api-key` header together with the `anthropic-version` header
//! (defaulting to `2023-06-01`), injected via [`just_common::transport::http::build_client_with_headers`].
#![warn(missing_docs)]

mod client;
mod client_builder;
mod error;
mod stream;
pub mod transport;
pub mod types;

pub use client::AnthropicClient;
pub use client_builder::AnthropicClientBuilder;
pub use error::Error;
pub use stream::MessageEventStream;
