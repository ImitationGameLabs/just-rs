//! Shared error types for the transport layer and provider clients.
//!
//! # Rendering contract
//!
//! - [`Display`](std::fmt::Display) is terse and log-safe: it never carries a
//!   captured response body, so rendered error strings stay bounded and free of
//!   raw (potentially large or sensitive) provider payloads. The
//!   `error_display_does_not_dump_raw_response_body` tests in the provider crates
//!   guard this.
//! - [`Debug`](std::fmt::Debug) is **not** the diagnostics channel:
//!   `anyhow::Error`'s `Debug` renders its cause chain through each error's
//!   `Display`, so once an error is boxed into `anyhow`, a body that `Display`
//!   omits is hidden from `{:?}` as well.
//! - To recover a captured body programmatically, use [`captured_body`], which
//!   walks the `source()` chain and downcasts.

use std::error::Error as StdError;
use std::string::FromUtf8Error;

use thiserror::Error;

use reqwest::StatusCode;

/// Errors produced by the shared HTTP/SSE transport layer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TransportError {
    #[error("invalid configuration: {0}")]
    InvalidConfig(&'static str),

    #[error("failed to build http client: {0}")]
    BuildClient(#[source] reqwest::Error),

    #[error("request failed: {0}")]
    Transport(#[source] reqwest::Error),

    /// A non-success HTTP status, with the full response body captured on `body`.
    ///
    /// `body` is read under the shared size cap (`MAX_BODY_BYTES`). Bodies that fit are captured
    /// in full; an oversized error body instead surfaces as [`BodyTooLarge`](Self::BodyTooLarge)
    /// and this variant is not produced. So when this variant is present, `body` is complete —
    /// never a truncated prefix.
    ///
    /// `body` is intentionally **not** part of [`Display`](std::fmt::Display) (the `#[error]` line
    /// carries only `status`), so rendered error strings stay terse and log-safe. Recover it via
    /// [`captured_body`] or by reading the field after downcasting through the `source()` chain;
    /// see the module-level docs for why `Debug` (`{:?}`) is not a reliable channel.
    #[error("api returned {status}")]
    HttpStatus { status: StatusCode, body: String },

    /// A streamed response chunk could not be deserialized.
    ///
    /// Produced only by the SSE event parser, one event at a time. When a
    /// `TransportError` is lifted into a [`ProviderError`] via `From`,
    /// this surfaces as `ProviderError::Transport(TransportError::Deserialize)` —
    /// **not** as `ProviderError::Deserialize`, which is reserved for full-body
    /// failures produced by `parse_json`. Consumers matching for deserialization
    /// failures across both paths must account for both variants.
    ///
    /// `body` is the SSE chunk payload that failed to parse; it is accessible via
    /// [`captured_body`], not `Display`.
    #[error("failed to deserialize response body: {source}")]
    Deserialize {
        #[source]
        source: serde_json::Error,
        body: String,
    },

    #[error("failed to decode streamed response as UTF-8: {0}")]
    Utf8(#[source] FromUtf8Error),

    /// A non-streaming response body exceeded the shared size cap and was not fully buffered.
    ///
    /// Produced by the capped body reader (`read_body_text`). Distinct from
    /// [`InvalidResponse`](Self::InvalidResponse), which reports *content* problems (empty body,
    /// malformed structure) rather than a size limit. The reader stops before the offending chunk
    /// is appended, so no body text is carried here. When the overflow occurs while reading an
    /// *error* body, the HTTP status is not carried here either — it is visible only to callers
    /// that read status off the raw response before consuming the body (the `prepare`/`send` path).
    #[error("response body exceeded {limit}-byte limit")]
    BodyTooLarge { limit: usize },

    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

/// Generic error type for OpenAI-compatible API provider clients.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProviderError {
    /// Transport-layer error from the shared HTTP/SSE layer.
    ///
    /// This is also where streaming/chunk deserialization failures land: an SSE
    /// event that fails to parse originates as `TransportError::Deserialize` and
    /// is wrapped here, **not** as the [`Deserialize`](Self::Deserialize) variant
    /// below, which is reserved for full response-body failures from `parse_json`.
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),

    /// The request shape was invalid for the selected client method.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Failed to serialize the request body.
    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),

    /// Failed to deserialize a full response body.
    ///
    /// Produced only by `parse_json`, which deserializes the entire HTTP response
    /// body. Streaming/chunk deserialization failures are a separate concern: they
    /// originate as `TransportError::Deserialize` and surface on
    /// [`Transport`](Self::Transport) (via `From<TransportError>`), not here.
    ///
    /// `body` is the full response body that failed to parse; it is accessible via
    /// [`captured_body`], not `Display`.
    #[error("failed to deserialize response body: {source}")]
    Deserialize {
        #[source]
        source: serde_json::Error,
        body: String,
    },
}

/// Walk the `source()` chain of `error` and return the first captured diagnostic
/// `body`, if any.
///
/// Three variants carry a `body: String` that [`Display`](std::fmt::Display)
/// deliberately omits (to keep rendered error strings terse and log-safe). This
/// accessor recovers it in one call, so consumers need not hand-roll a `source()`
/// + downcast walk:
///
/// - [`TransportError::HttpStatus`] — the provider's error-response body.
/// - [`ProviderError::Deserialize`] — the full response body that failed to parse.
/// - [`TransportError::Deserialize`] — the **SSE chunk** payload that failed to
///   parse (a single event's data, not a response body).
///
/// Returns `None` if no body-carrying error is reachable.
///
/// # Why not `{:?}`?
///
/// `Debug` is not a reliable channel for this: `anyhow::Error`'s `Debug` renders
/// its cause chain through each error's `Display`, so once an error is boxed into
/// `anyhow`, the body — which `Display` omits — is hidden from `{:?}` as well.
/// Prefer this accessor. For an `&anyhow::Error`, pass
/// `error.as_ref::<dyn std::error::Error>()` (anyhow implements `AsRef<dyn StdError>`).
pub fn captured_body<'a>(error: &'a (dyn StdError + 'static)) -> Option<&'a str> {
    // Chains here are statically bounded (Backend -> Provider -> Transport,
    // depth <= 3); no cycle guard is needed.
    let mut current: Option<&(dyn StdError + 'static)> = Some(error);
    while let Some(e) = current {
        // HttpStatus and Deserialize both carry the captured text as `body`.
        if let Some(TransportError::HttpStatus { body, .. })
        | Some(TransportError::Deserialize { body, .. }) = e.downcast_ref::<TransportError>()
        {
            return Some(body);
        }
        if let Some(ProviderError::Deserialize { body, .. }) = e.downcast_ref::<ProviderError>() {
            return Some(body);
        }
        current = e.source();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    /// After dropping `#[error(transparent)]` from `ProviderError::Transport`, the wrapped
    /// `TransportError` must be reachable by walking `source()` and downcasting — so a consumer
    /// can recover, e.g. a 429 status from the error object. This contract would have failed
    /// before the change: `transparent` flattened `TransportError` out of the source chain.
    #[test]
    fn transport_error_reachable_via_source_chain() {
        let te = TransportError::HttpStatus {
            status: StatusCode::TOO_MANY_REQUESTS,
            body: "rate limited".into(),
        };
        let pe: ProviderError = te.into(); // ProviderError::Transport(te)

        let mut cur: Option<&(dyn StdError + 'static)> = Some(&pe);
        let mut found = None;
        while let Some(e) = cur {
            if let Some(t) = e.downcast_ref::<TransportError>() {
                found = Some(t);
                break;
            }
            cur = e.source();
        }

        let found = found.expect("TransportError must be reachable via the source chain");
        assert!(
            matches!(
                found,
                TransportError::HttpStatus { status, .. } if *status == StatusCode::TOO_MANY_REQUESTS
            ),
            "expected HttpStatus 429, got {found:?}"
        );
    }

    #[test]
    fn captured_body_finds_http_status() {
        let body = r#"{"error":"bad request"}"#;
        let te = TransportError::HttpStatus {
            status: StatusCode::BAD_REQUEST,
            body: body.into(),
        };
        // Bare TransportError.
        assert_eq!(captured_body(&te), Some(body));
        // Lifted through ProviderError::Transport via #[from].
        let pe: ProviderError = te.into();
        assert_eq!(captured_body(&pe), Some(body));
    }

    #[test]
    fn captured_body_finds_transport_deserialize() {
        let payload = "data: not-json";
        let te = TransportError::Deserialize {
            source: serde_json::from_str::<serde_json::Value>("bad json").unwrap_err(),
            body: payload.into(),
        };
        // Bare.
        assert_eq!(captured_body(&te), Some(payload));
        // Lifted via ProviderError::Transport(...) — the SSE-chunk path.
        let pe: ProviderError = te.into();
        assert_eq!(captured_body(&pe), Some(payload));
    }

    #[test]
    fn captured_body_finds_provider_deserialize() {
        let body = r#"{"not":"valid for schema"}"#;
        let pe = ProviderError::Deserialize {
            source: serde_json::from_str::<serde_json::Value>("bad json").unwrap_err(),
            body: body.into(),
        };
        assert_eq!(captured_body(&pe), Some(body));
    }

    #[test]
    fn captured_body_none_when_absent() {
        let config = TransportError::InvalidConfig("missing base url");
        assert_eq!(captured_body(&config), None);

        // BodyTooLarge deliberately carries no body text (see its doc).
        let too_large = TransportError::BodyTooLarge {
            limit: 8 * 1024 * 1024,
        };
        assert_eq!(captured_body(&too_large), None);

        // The remaining body-less variants must also yield None.
        let utf8 = TransportError::Utf8(String::from_utf8(vec![0xff, 0xfe]).unwrap_err());
        assert_eq!(captured_body(&utf8), None);

        let invalid = TransportError::InvalidResponse("malformed structure".into());
        assert_eq!(captured_body(&invalid), None);
    }
}
