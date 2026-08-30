//! Error taxonomy for the LLM client layer, partitioned by **failure nature**.
//!
//! Three distinct failure natures get three distinct types, so a caller can never conflate them
//! at the type level:
//!
//! - [`BackendConstructError`] — *constructing* a backend failed (`LlmBackend::new`,
//!   `BackendFactory::create`). Precondition/setup failures only; never a generation call.
//! - [`CapabilityError`] — a backend *statically* does not offer a capability
//!   (`CapabilityNegotiation`). Decided without IO; never a provider call.
//! - [`BackendError`] — *operating* an already-constructed backend failed (generation,
//!   streaming, prepare/send/parse, rendering, model catalog, balance). Runtime execution only.

use std::{error::Error as StdError, fmt};

use thiserror::Error;

use just_common::error::TransportError;
use reqwest::StatusCode;

/// Boxed provider error source carried by [`BackendError::Provider`] and
/// [`BackendConstructError::Provider`].
///
/// Kept as `BoxError` rather than a concrete `ProviderError` so that custom backends wrapping an
/// arbitrary provider SDK (not necessarily one built on `just-common`) can carry their own error
/// type. Callers that need the structured `ProviderError` produced by the built-in backends
/// downcast it (as the tests do).
pub type BoxError = Box<dyn StdError + Send + Sync>;

/// Capability names used in client-level unsupported or unavailable errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Capability {
    /// Model catalog listing.
    ModelCatalog,
    /// Balance or quota inspection.
    Balance,
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::ModelCatalog => "model catalog",
            Self::Balance => "balance",
        };

        f.write_str(label)
    }
}

/// Constructing a backend failed: a precondition or setup failure.
///
/// Produced only by [`LlmBackend::new`](crate::LlmBackend::new) and
/// [`BackendFactory::create`](crate::BackendFactory::create). Distinct from [`BackendError`]
/// (operating a backend) and [`CapabilityError`] (static capability gating): construction never
/// performs a generation call, so its failure model is isolated.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BackendConstructError {
    /// No constructor is registered for the requested family (factory dispatch).
    #[error("no backend registered for family '{family}'")]
    UnknownFamily {
        /// The family string that had no registered constructor.
        family: String,
    },

    /// The provider client could not be built.
    ///
    /// Carries the backend family for attribution and the provider build failure as a boxed
    /// source, so callers can downcast to inspect `TransportError::InvalidConfig` /
    /// `TransportError::BuildClient` for the built-in backends, while custom backends may box any
    /// SDK-specific error.
    #[error("failed to build {family} backend: {source}")]
    Provider {
        /// Backend family that failed to build.
        family: &'static str,
        /// Provider-specific source error.
        #[source]
        source: BoxError,
    },
}

impl BackendConstructError {
    /// Creates an unknown-family error (factory dispatch miss).
    pub fn unknown_family(family: impl Into<String>) -> Self {
        Self::UnknownFamily {
            family: family.into(),
        }
    }

    /// Wraps a provider build failure for the given backend family.
    pub fn provider<E>(family: &'static str, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self::Provider {
            family,
            source: Box::new(source),
        }
    }
}

/// Static capability gating failure, returned by
/// [`CapabilityNegotiation`](crate::CapabilityNegotiation).
///
/// Decided without IO: it is a fact about whether a backend *offers* a capability, not whether a
/// live call to it failed. The latter is [`BackendError`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CapabilityError {
    /// The backend never offers the requested capability.
    #[error("{family} does not support {capability}")]
    Unsupported {
        /// Backend family.
        family: &'static str,
        /// Capability that was requested.
        capability: Capability,
    },

    /// The backend is expected to offer the capability but has not implemented it yet.
    #[error("{family} has not implemented {capability}")]
    Unimplemented {
        /// Backend family.
        family: &'static str,
        /// Capability that was expected.
        capability: Capability,
    },

    /// The backend can offer the capability in principle, but not in the current state.
    #[error("{family} cannot currently provide {capability}: {message}")]
    Unavailable {
        /// Backend family.
        family: &'static str,
        /// Capability that is temporarily unavailable.
        capability: Capability,
        /// Additional explanation from the backend adapter.
        message: String,
    },
}

impl CapabilityError {
    /// Creates an unsupported-capability error for the given backend.
    pub fn unsupported(family: &'static str, capability: Capability) -> Self {
        Self::Unsupported { family, capability }
    }

    /// Creates an unimplemented-capability error for the given backend.
    pub fn unimplemented(family: &'static str, capability: Capability) -> Self {
        Self::Unimplemented { family, capability }
    }

    /// Creates an unavailable-capability error for the given backend.
    pub fn unavailable(
        family: &'static str,
        capability: Capability,
        message: impl Into<String>,
    ) -> Self {
        Self::Unavailable {
            family,
            capability,
            message: message.into(),
        }
    }
}

/// Operating an already-constructed backend failed: a runtime execution failure.
///
/// Returned by generation, streaming, prepare/send/parse, rendering, model catalog, and
/// balance calls — everything that drives a live backend. Construction failures are
/// [`BackendConstructError`]; static capability gating is [`CapabilityError`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BackendError {
    /// The request was invalid before it reached the provider.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Failed to serialize a request payload or render provider-specific types.
    #[error("serialization error: {source}")]
    Serialization {
        /// Underlying `serde_json` serialization failure.
        #[source]
        source: serde_json::Error,
    },

    /// The underlying provider SDK returned an error.
    ///
    /// Carries the backend family for attribution and the provider error as a boxed source (see
    /// [`BoxError`] for why this is not a concrete `ProviderError`).
    ///
    /// `Display` inlines the source chain, so plain `{}` yields a single informative line. Any
    /// provider response body carried by the source is recoverable via
    /// [`captured_body`](crate::captured_body), not via `Display` or `{:?}`.
    #[error("{family} backend error: {source}")]
    Provider {
        /// Backend family.
        family: &'static str,
        /// Provider-specific source error.
        #[source]
        source: BoxError,
    },
    /// The request cannot be represented on this provider's wire (pre-flight; no IO).
    ///
    /// Distinct from [`BackendError::Serialization`]: that reports a `serde` mechanism
    /// failure, this one reports a semantic mismatch - the request is well-formed but
    /// expresses something this provider's wire cannot carry (e.g. a file-id image
    /// source on a chat-completions endpoint).
    #[error("request cannot be serialized for {family}: {message}")]
    Unserializable {
        /// Backend family.
        family: &'static str,
        /// What could not be represented, and why.
        message: String,
    },
}

impl BackendError {
    /// Creates an invalid-request error with a stable, user-facing message.
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest(message.into())
    }

    /// Creates a serialization error wrapping a `serde_json` failure.
    pub fn serialization(source: serde_json::Error) -> Self {
        Self::Serialization { source }
    }

    /// Wraps a provider-specific source error for the given backend family.
    pub fn provider<E>(family: &'static str, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self::Provider {
            family,
            source: Box::new(source),
        }
    }

    /// Creates a wire-unrepresentable-request error (pre-flight; no IO).
    pub fn unserializable(family: &'static str, message: impl Into<String>) -> Self {
        Self::Unserializable {
            family,
            message: message.into(),
        }
    }
}

/// A provider-side rejection lifted from an HTTP error response.
///
/// [`status`](ProviderRejection::status) is always present once a rejection exists;
/// [`code`](ProviderRejection::code) and [`error_type`](ProviderRejection::error_type)
/// carry the provider's own `error.code` / `error.type` values when its response body
/// was JSON of the common `{"error": {...}}` shape.
///
/// The raw response body is deliberately not carried here: it always remains
/// reachable, verbatim, via [`captured_body`](crate::captured_body).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderRejection {
    /// HTTP status the provider answered with.
    pub status: StatusCode,
    /// Provider-native error code (`error.code`), when present.
    pub code: Option<String>,
    /// Provider-native error type (`error.type`), when present.
    pub error_type: Option<String>,
}

/// Walks `error`'s source chain (the same walk as [`captured_body`](crate::captured_body)) to the captured
/// HTTP status of a provider error response and lifts the provider's own
/// `error.code` / `error.type` out of the body when it is JSON of the common wrapped
/// shape. Returns `None` when no HTTP status is reachable, i.e. the error did not
/// originate from a provider response (pre-flight [`BackendError::InvalidRequest`] or
/// [`BackendError::Unserializable`]).
pub fn provider_rejection(error: &BackendError) -> Option<ProviderRejection> {
    let mut current: Option<&(dyn StdError + 'static)> = Some(error);
    while let Some(e) = current {
        if let Some(TransportError::HttpStatus { status, body }) =
            e.downcast_ref::<TransportError>()
        {
            let (code, error_type) = lift_provider_error_fields(body);
            return Some(ProviderRejection {
                status: *status,
                code,
                error_type,
            });
        }
        current = e.source();
    }
    None
}

/// Extracts `error.code` / `error.type` from a JSON error body; `None` for both when
/// the body is not JSON of that shape (status alone still identifies the rejection).
fn lift_provider_error_fields(body: &str) -> (Option<String>, Option<String>) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return (None, None);
    };
    let error = &value["error"];
    let code = error
        .get("code")
        .and_then(|v| v.as_str())
        .map(str::to_owned);
    let error_type = error
        .get("type")
        .and_then(|v| v.as_str())
        .map(str::to_owned);
    (code, error_type)
}

#[cfg(test)]
mod tests {
    use super::BackendError;
    use super::provider_rejection;
    use crate::captured_body;
    use just_common::error::{ProviderError, TransportError};
    use reqwest::StatusCode;

    #[test]
    fn captured_body_traverses_backend_error() {
        let te = TransportError::HttpStatus {
            status: StatusCode::BAD_REQUEST,
            body: r#"{"error":"context_length_exceeded"}"#.into(),
        };
        let be = BackendError::provider("openai-compatible", ProviderError::Transport(te));

        // captured_body must cross the BoxError indirection: BackendError::Provider
        // -> boxed ProviderError -> TransportError -> HttpStatus.
        assert_eq!(
            captured_body(&be),
            Some(r#"{"error":"context_length_exceeded"}"#)
        );
    }

    #[test]
    fn provider_rejection_lifts_status_code_and_error_type() {
        let te = TransportError::HttpStatus {
            status: StatusCode::BAD_REQUEST,
            body: r#"{"error":{"code":"image_invalid","type":"invalid_request_error"}}"#.into(),
        };
        let be = BackendError::provider("openai-compatible", ProviderError::Transport(te));

        let rejection = provider_rejection(&be).expect("http rejection must be reachable");
        assert_eq!(rejection.status, StatusCode::BAD_REQUEST);
        assert_eq!(rejection.code.as_deref(), Some("image_invalid"));
        assert_eq!(
            rejection.error_type.as_deref(),
            Some("invalid_request_error")
        );

        // The raw body stays verbatim behind captured_body, not on the rejection.
        assert_eq!(
            captured_body(&be),
            Some(r#"{"error":{"code":"image_invalid","type":"invalid_request_error"}}"#)
        );
    }

    #[test]
    fn provider_rejection_without_json_body_yields_status_only() {
        let te = TransportError::HttpStatus {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: "gateway exploded".into(),
        };
        let be = BackendError::provider("openai-compatible", ProviderError::Transport(te));

        let rejection = provider_rejection(&be).expect("http rejection must be reachable");
        assert_eq!(rejection.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(rejection.code, None);
        assert_eq!(rejection.error_type, None);
    }

    #[test]
    fn provider_rejection_absent_for_pre_flight_errors() {
        let be = BackendError::unserializable("deepseek", "file-id image source");
        assert!(provider_rejection(&be).is_none());
        assert!(matches!(be, BackendError::Unserializable { .. }));
    }
}
