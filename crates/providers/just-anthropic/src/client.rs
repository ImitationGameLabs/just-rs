//! Anthropic Messages API client.

use just_common::transport::http::{endpoint_url, ensure_success, parse_json};
use reqwest::Method;

use crate::{
    Error,
    types::{
        message::Message,
        models::ListModelsResponse,
        request::{CountTokensRequest, CountTokensResponse, CreateMessageRequest},
    },
};

/// Async client for the Anthropic Messages API.
///
/// Holds a pre-configured `reqwest::Client` and base URL. Construct via
/// [`AnthropicClient::builder()`] or [`AnthropicClient::new()`].
#[derive(Clone, Debug)]
pub struct AnthropicClient {
    http: reqwest::Client,
    base_url: String,
}

impl AnthropicClient {
    // --- construction, accessors, and the prepare/send (raw HTTP) surface ---

    /// Creates a new client from pre-built components.
    ///
    /// The HTTP client should already have auth headers set (e.g. via
    /// [`just_common::transport::http::build_client_with_headers`]).
    pub fn new(http: reqwest::Client, base_url: String) -> Self {
        Self { http, base_url }
    }

    /// Returns a builder for constructing a new client.
    pub fn builder() -> crate::client_builder::AnthropicClientBuilder {
        crate::client_builder::AnthropicClientBuilder::new()
    }

    /// Returns the underlying HTTP client.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http
    }

    /// Returns the configured base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Prepares a non-streaming message request for later execution.
    ///
    /// Serializes the request body and builds a complete `reqwest::Request`.
    /// This is a synchronous operation (no IO).
    pub fn prepare(&self, request: CreateMessageRequest) -> Result<reqwest::Request, Error> {
        if request.stream.unwrap_or(false) {
            return Err(Error::InvalidRequest(
                "stream=true is not supported by prepare; use prepare_streaming instead".into(),
            ));
        }
        self.build_request(
            Method::POST,
            "messages",
            Some(serde_json::to_string(&request)?),
        )
    }

    /// Prepares a streaming message request for later execution.
    ///
    /// Forces `stream = true` on the request, then serializes and builds.
    /// This is a synchronous operation (no IO).
    pub fn prepare_streaming(
        &self,
        mut request: CreateMessageRequest,
    ) -> Result<reqwest::Request, Error> {
        request.stream = Some(true);
        self.build_request(
            Method::POST,
            "messages",
            Some(serde_json::to_string(&request)?),
        )
    }

    /// Sends a prepared request and returns the raw HTTP response without checking status.
    ///
    /// Callers must handle non-success statuses themselves. For automatic status checking and
    /// deserialization, use [`create_message`](Self::create_message) or
    /// [`stream_message`](Self::stream_message).
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
    /// When `body` is `None` no `Content-Type` header and no body are set.
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

impl AnthropicClient {
    // --- typed operations (hide HTTP entirely) ---

    /// Parses a raw HTTP response into a provider-native `Message`.
    ///
    /// Performs HTTP status checking and JSON deserialization. Combine with
    /// [`prepare`](Self::prepare) and [`send`](Self::send) for full control over the request
    /// lifecycle, e.g. to inspect response headers before consuming the body.
    pub async fn parse(&self, response: reqwest::Response) -> Result<Message, Error> {
        parse_json(response).await
    }

    /// Parses a raw HTTP response into a provider-native streaming event stream.
    ///
    /// Checks the HTTP status first (the SSE stream parser assumes a 2xx event stream).
    pub async fn parse_streaming(
        &self,
        response: reqwest::Response,
    ) -> Result<crate::MessageEventStream, Error> {
        let response = ensure_success(response).await?;
        crate::MessageEventStream::from_response(response).map_err(Error::Transport)
    }

    /// Executes a non-streaming message request.
    pub async fn create_message(&self, request: CreateMessageRequest) -> Result<Message, Error> {
        let response = self.send(self.prepare(request)?).await?;
        self.parse(response).await
    }

    /// Starts a streaming message request.
    pub async fn stream_message(
        &self,
        request: CreateMessageRequest,
    ) -> Result<crate::MessageEventStream, Error> {
        let response = self.send(self.prepare_streaming(request)?).await?;
        self.parse_streaming(response).await
    }

    /// Counts the tokens an input would consume without generating a response.
    pub async fn count_tokens(
        &self,
        request: CountTokensRequest,
    ) -> Result<CountTokensResponse, Error> {
        let request = self.build_request(
            Method::POST,
            "messages/count_tokens",
            Some(serde_json::to_string(&request)?),
        )?;
        let response = self.send(request).await?;
        parse_json(response).await
    }

    /// Lists models currently exposed by the configured endpoint.
    pub async fn list_models(&self) -> Result<ListModelsResponse, Error> {
        let url = endpoint_url(&self.base_url, "models")?;
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(just_common::error::TransportError::Transport)?;
        parse_json(response).await
    }
}
