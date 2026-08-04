use crate::{
    BackendError,
    types::generation::{GenerationRequest, Message, ToolChoice},
};

/// Validates that a request is suitable for a non-streaming endpoint.
pub fn validate_non_streaming_request(
    request: &GenerationRequest,
    method_name: &'static str,
    streaming_method_name: &'static str,
) -> Result<(), BackendError> {
    validate_common_request(request)?;

    if request.stream.unwrap_or(false) {
        return Err(BackendError::invalid_request(format!(
            "stream=true is not supported by {method_name}; use {streaming_method_name} instead"
        )));
    }

    Ok(())
}

/// Validates a request and ensures streaming is enabled, returning the modified request.
pub fn into_validated_streaming_request(
    mut request: GenerationRequest,
    method_name: &'static str,
) -> Result<GenerationRequest, BackendError> {
    validate_common_request(&request)?;

    if request.stream == Some(false) {
        return Err(BackendError::invalid_request(format!(
            "stream=false is not supported by {method_name}"
        )));
    }

    request.stream = Some(true);
    Ok(request)
}

/// Validates request fields shared by streaming and non-streaming paths.
pub fn validate_common_request(request: &GenerationRequest) -> Result<(), BackendError> {
    // System messages must occupy the leading block: backends that carry the system prompt as a
    // top-level parameter (Responses `instructions`, Anthropic `system`) extract it there, so a
    // system message appearing later in the list cannot be represented faithfully.
    let mut seen_non_system = false;
    for message in &request.messages {
        if matches!(message, Message::System { .. }) {
            if seen_non_system {
                return Err(BackendError::invalid_request(
                    "system messages must precede all non-system messages",
                ));
            }
        } else {
            seen_non_system = true;
        }
    }

    if request.tool_choice.is_some() && request.tools.as_ref().is_none_or(Vec::is_empty) {
        return Err(BackendError::invalid_request(
            "tool_choice requires at least one configured tool",
        ));
    }

    if let Some(ToolChoice::Named(choice)) = &request.tool_choice {
        let tool_exists = request.tools.as_ref().is_some_and(|tools| {
            tools
                .iter()
                .any(|tool| tool.function.name == choice.function.name)
        });

        if !tool_exists {
            return Err(BackendError::invalid_request(format!(
                "tool_choice references unknown tool '{}'",
                choice.function.name
            )));
        }
    }

    if request.top_logprobs.is_some() && request.logprobs != Some(true) {
        return Err(BackendError::invalid_request(
            "top_logprobs requires logprobs=true",
        ));
    }

    Ok(())
}
