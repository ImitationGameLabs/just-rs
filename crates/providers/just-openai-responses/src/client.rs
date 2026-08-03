//! OpenAI Responses API client.

use just_common::transport::http::{endpoint_url, ensure_success, parse_json};
use reqwest::Method;

use crate::{
    Error,
    types::{
        models::ListModelsResponse,
        request::{CompactRequest, CompactedResponse, CreateResponseRequest},
        response::{DeleteResponse, Response},
        shared::ResponseIncludable,
    },
};

/// Async client for the OpenAI Responses API.
///
/// Holds a pre-configured `reqwest::Client` and base URL. Construct via
/// [`ResponsesClient::builder()`] or [`ResponsesClient::new()`].
#[derive(Clone, Debug)]
pub struct ResponsesClient {
    http: reqwest::Client,
    base_url: String,
}

impl ResponsesClient {
    // --- construction, accessors, and the prepare/send (raw HTTP) surface ---

    /// Creates a new client from pre-built components.
    ///
    /// The HTTP client should already have auth headers set (e.g. via
    /// [`just_common::transport::http::build_client`]).
    pub fn new(http: reqwest::Client, base_url: String) -> Self {
        Self { http, base_url }
    }

    /// Returns a builder for constructing a new client.
    pub fn builder() -> crate::client_builder::ResponsesClientBuilder {
        crate::client_builder::ResponsesClientBuilder::new()
    }

    /// Returns the underlying HTTP client.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http
    }

    /// Returns the configured base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Prepares a non-streaming response request for later execution.
    ///
    /// Serializes the request body and builds a complete `reqwest::Request`.
    /// This is a synchronous operation (no IO).
    pub fn prepare(&self, request: CreateResponseRequest) -> Result<reqwest::Request, Error> {
        if request.stream.unwrap_or(false) {
            return Err(Error::InvalidRequest(
                "stream=true is not supported by prepare; use prepare_streaming instead".into(),
            ));
        }
        self.build_request(
            Method::POST,
            "/responses",
            Some(serde_json::to_string(&request)?),
        )
    }

    /// Prepares a streaming response request for later execution.
    ///
    /// Forces `stream = true` on the request, then serializes and builds.
    /// This is a synchronous operation (no IO).
    pub fn prepare_streaming(
        &self,
        mut request: CreateResponseRequest,
    ) -> Result<reqwest::Request, Error> {
        request.stream = Some(true);
        self.build_request(
            Method::POST,
            "/responses",
            Some(serde_json::to_string(&request)?),
        )
    }

    /// Sends a prepared request and returns the raw HTTP response without checking status.
    ///
    /// Callers must handle non-success statuses themselves. For automatic status checking and
    /// deserialization, use [`create_response`](Self::create_response) or
    /// [`stream_response`](Self::stream_response).
    pub async fn send(&self, request: reqwest::Request) -> Result<reqwest::Response, Error> {
        self.http
            .execute(request)
            .await
            .map_err(just_common::error::TransportError::Transport)
            .map_err(Error::from)
    }

    /// Builds a `reqwest::Request` from an HTTP method, endpoint path, and optional serialized
    /// JSON body.
    ///
    /// When `body` is `None` no `Content-Type` header and no body are set (used by body-less
    /// POSTs such as cancel).
    fn build_request(
        &self,
        method: Method,
        path: &str,
        body: Option<String>,
    ) -> Result<reqwest::Request, Error> {
        let url = endpoint_url(&self.base_url, path)?;
        let mut request = self.http.request(method, url);
        if let Some(body) = body {
            request = request
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body);
        }
        request
            .build()
            .map_err(just_common::error::TransportError::Transport)
            .map_err(Error::from)
    }
}

impl ResponsesClient {
    // --- typed operations (hide HTTP entirely) ---

    /// Parses a raw HTTP response into a provider-native `Response`.
    ///
    /// Performs HTTP status checking and JSON deserialization. Combine with
    /// [`prepare`](Self::prepare) and [`send`](Self::send) for full control over the request
    /// lifecycle, e.g. to inspect response headers before consuming the body.
    pub async fn parse(&self, response: reqwest::Response) -> Result<Response, Error> {
        parse_json(response).await
    }

    /// Parses a raw HTTP response into a provider-native streaming event stream.
    ///
    /// Checks the HTTP status first (the SSE stream parser assumes a 2xx event stream).
    pub async fn parse_streaming(
        &self,
        response: reqwest::Response,
    ) -> Result<crate::ResponsesEventStream, Error> {
        let response = ensure_success(response).await?;
        crate::ResponsesEventStream::from_response(response).map_err(Error::Transport)
    }

    /// Executes a non-streaming response request.
    pub async fn create_response(&self, request: CreateResponseRequest) -> Result<Response, Error> {
        let response = self.send(self.prepare(request)?).await?;
        self.parse(response).await
    }

    /// Starts a streaming response request.
    pub async fn stream_response(
        &self,
        request: CreateResponseRequest,
    ) -> Result<crate::ResponsesEventStream, Error> {
        let response = self.send(self.prepare_streaming(request)?).await?;
        self.parse_streaming(response).await
    }

    /// Retrieves a previously stored response.
    ///
    /// `include` requests optional data (e.g. `reasoning.encrypted_content`,
    /// `file_search_call.results`); `include_obfuscation` toggles obfuscation of API keys in
    /// returned data.
    pub async fn retrieve_response(
        &self,
        response_id: &str,
        include: Option<&[ResponseIncludable]>,
        include_obfuscation: Option<bool>,
    ) -> Result<Response, Error> {
        // The query values here are restricted to safe ASCII (dotted include names and
        // booleans), so no percent-encoding is required.
        let mut query = Vec::new();
        if let Some(include) = include {
            for value in include {
                query.push(format!("include={}", value.as_str()));
            }
        }
        if let Some(include_obfuscation) = include_obfuscation {
            query.push(format!("include_obfuscation={include_obfuscation}"));
        }
        let path = if query.is_empty() {
            format!("/responses/{response_id}")
        } else {
            format!("/responses/{response_id}?{}", query.join("&"))
        };
        let url = endpoint_url(&self.base_url, &path)?;
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(just_common::error::TransportError::Transport)?;
        parse_json(response).await
    }

    /// Cancels a response created with `background: true`.
    pub async fn cancel_response(&self, response_id: &str) -> Result<Response, Error> {
        let path = format!("/responses/{response_id}/cancel");
        let request = self.build_request(Method::POST, &path, None)?;
        let response = self.send(request).await?;
        self.parse(response).await
    }

    /// Compacts a chain of responses into a shorter set of items.
    pub async fn compact_response(
        &self,
        request: CompactRequest,
    ) -> Result<CompactedResponse, Error> {
        let request = self.build_request(
            Method::POST,
            "/responses/compact",
            Some(serde_json::to_string(&request)?),
        )?;
        let response = self.send(request).await?;
        parse_json(response).await
    }

    /// Deletes a stored response.
    pub async fn delete_response(&self, response_id: &str) -> Result<DeleteResponse, Error> {
        let path = format!("/responses/{response_id}");
        let url = endpoint_url(&self.base_url, &path)?;
        let response = self
            .http
            .delete(url)
            .send()
            .await
            .map_err(just_common::error::TransportError::Transport)?;
        parse_json(response).await
    }

    /// Lists models currently exposed by the configured endpoint.
    pub async fn list_models(&self) -> Result<ListModelsResponse, Error> {
        let url = endpoint_url(&self.base_url, "/models")?;
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(just_common::error::TransportError::Transport)?;
        parse_json(response).await
    }
}
