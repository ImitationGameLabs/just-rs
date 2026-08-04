//! OpenAI Responses LLM backend adapter.
//!
//! [`OpenAiResponsesBackend`] wraps a [`just_openai_responses::ResponsesClient`] and implements
//! [`LlmBackend`], mapping the semantic [`GenerationRequest`] to the Responses API. System prompts
//! are extracted to the top-level `instructions` field, tool results round-trip through
//! `function_call_output` items, and reasoning preserves its `id`/`encrypted_content` fidelity
//! carriers. Balance inspection is negotiated explicitly and returns
//! [`CapabilityError::Unsupported`](crate::CapabilityError::Unsupported) because the Responses API
//! does not expose a balance endpoint.
//!
//! Construct from raw inputs (API key + optional base URL) via the [`LlmBackend::new`] trait
//! method ([`LlmBackend`] must be in scope), via [`OpenAiResponsesBackend::from_provider_client`]
//! with a pre-built provider client, or through a [`BackendFactory`](crate::BackendFactory) that
//! registers this backend.

mod conversions;

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    capability::{CapabilityNegotiation, GenerationStream, Identifiable, ModelCatalog},
    error::{BackendConstructError, BackendError, CapabilityError},
    provider::validation::{into_validated_streaming_request, validate_non_streaming_request},
    types::{
        generation::{GenerationRequest, GenerationResponse, Message, ToolDefinition},
        model::{ModelCatalogResponse, ModelInfo},
    },
};

use super::LlmBackend;

/// `just-llm-client` adapter for the OpenAI Responses API.
///
/// Delegates HTTP dispatch to a [`just_openai_responses::ResponsesClient`] and handles type
/// conversion between normalized client types and the Responses wire format.
#[derive(Clone, Debug)]
pub struct OpenAiResponsesBackend {
    client: just_openai_responses::ResponsesClient,
}

impl OpenAiResponsesBackend {
    /// Creates a new backend from a pre-built provider client.
    pub fn from_provider_client(client: just_openai_responses::ResponsesClient) -> Self {
        Self { client }
    }
}

impl Identifiable for OpenAiResponsesBackend {
    fn family(&self) -> &'static str {
        crate::family::OPENAI_RESPONSES
    }
}

impl CapabilityNegotiation for OpenAiResponsesBackend {
    fn model_catalog(&self) -> Result<&dyn ModelCatalog, CapabilityError> {
        Ok(self)
    }
}

#[async_trait]
impl LlmBackend for OpenAiResponsesBackend {
    // --- prepare / send (raw HTTP surface) ---

    fn prepare(&self, request: GenerationRequest) -> Result<reqwest::Request, BackendError> {
        validate_non_streaming_request(&request, "prepare", "prepare_streaming")?;
        let provider_req: just_openai_responses::types::request::CreateResponseRequest =
            request.try_into()?;
        self.client
            .prepare(provider_req)
            .map_err(|e| BackendError::provider(self.family(), e))
    }

    fn prepare_streaming(
        &self,
        request: GenerationRequest,
    ) -> Result<reqwest::Request, BackendError> {
        let request = into_validated_streaming_request(request, "prepare_streaming")?;
        let provider_req: just_openai_responses::types::request::CreateResponseRequest =
            request.try_into()?;
        self.client
            .prepare_streaming(provider_req)
            .map_err(|e| BackendError::provider(self.family(), e))
    }

    async fn send(&self, prepared: reqwest::Request) -> Result<reqwest::Response, BackendError> {
        self.client
            .send(prepared)
            .await
            .map_err(|e| BackendError::provider(self.family(), e))
    }

    // --- parse + rendering ---

    async fn parse(&self, response: reqwest::Response) -> Result<GenerationResponse, BackendError> {
        let native: just_openai_responses::types::response::Response = self
            .client
            .parse(response)
            .await
            .map_err(|e| BackendError::provider(self.family(), e))?;
        Ok(native.into())
    }

    async fn parse_streaming(
        &self,
        response: reqwest::Response,
    ) -> Result<GenerationStream, BackendError> {
        let stream = self
            .client
            .parse_streaming(response)
            .await
            .map_err(|e| BackendError::provider(self.family(), e))?;
        let mapped = super::flatten_events(stream, conversions::event_to_generation_events);
        Ok(GenerationStream::new(mapped))
    }

    fn render_messages(&self, messages: &[Message]) -> Result<String, BackendError> {
        // Responses carries the system prompt as the top-level `instructions` field, so only the
        // non-system messages belong in the `input` items this render targets.
        let (_, remaining) = conversions::extract_instructions(messages.to_vec())?;
        let provider_messages = remaining
            .into_iter()
            .map(conversions::message_to_input_items)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        serde_json::to_string(&provider_messages).map_err(BackendError::serialization)
    }

    fn render_tools(&self, tools: &[ToolDefinition]) -> Result<String, BackendError> {
        let provider_tools: Vec<just_openai_responses::types::tool::ResponseTool> =
            tools.iter().cloned().map(conversions::wire_tool).collect();
        serde_json::to_string(&provider_tools).map_err(BackendError::serialization)
    }

    fn family() -> &'static str
    where
        Self: Sized,
    {
        crate::family::OPENAI_RESPONSES
    }

    /// Build a shared OpenAI Responses backend from raw inputs.
    ///
    /// `base_url = None` uses the provider default (`https://api.openai.com/v1`).
    #[allow(clippy::new_ret_no_self)]
    fn new(
        http: reqwest::ClientBuilder,
        api_key: &str,
        base_url: Option<&str>,
    ) -> Result<Arc<dyn LlmBackend>, BackendConstructError>
    where
        Self: Sized,
    {
        let mut builder = just_openai_responses::ResponsesClient::builder()
            .api_key(api_key)
            .http_client(http);
        if let Some(url) = base_url {
            builder = builder.base_url(url);
        }
        let client = builder
            .build()
            .map_err(|e| BackendConstructError::provider(crate::family::OPENAI_RESPONSES, e))?;
        Ok(Arc::new(Self::from_provider_client(client)))
    }
}

#[async_trait]
impl ModelCatalog for OpenAiResponsesBackend {
    async fn list_models(&self) -> Result<ModelCatalogResponse, BackendError> {
        let models = self
            .client
            .list_models()
            .await
            .map_err(|e| BackendError::provider(self.family(), e))?;

        Ok(ModelCatalogResponse {
            data: models
                .data
                .into_iter()
                .map(|model| ModelInfo {
                    id: model.id,
                    object: Some(model.object),
                    owned_by: Some(model.owned_by),
                })
                .collect(),
        })
    }
}
