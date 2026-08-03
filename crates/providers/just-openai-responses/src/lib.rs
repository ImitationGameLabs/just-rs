//! OpenAI Responses API provider SDK.
//!
//! This crate exposes a thin Rust client and wire-level DTOs for OpenAI's Responses API
//! (`POST /responses` with SSE streaming, plus retrieve/cancel/compact/delete and model
//! listing). Request and response types under [`types`] mirror the wire format; the client
//! under [`ResponsesClient`] handles the HTTP lifecycle with the same prepare/send/parse
//! pattern used across the workspace's provider crates.
//!
//! Some types intentionally overlap with `just-openai-compat` (e.g. model listing, text
//! configuration, tool choice) because the two crates are standalone wire-level type libraries
//! for different OpenAI API surfaces. This duplication is accepted per the workspace's DTO
//! layering convention.
#![warn(missing_docs)]

mod client;
mod client_builder;
mod error;
mod stream;
pub mod transport;
pub mod types;

pub use client::ResponsesClient;
pub use client_builder::ResponsesClientBuilder;
pub use error::Error;
pub use stream::ResponsesEventStream;
