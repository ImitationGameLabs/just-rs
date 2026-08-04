//! Stream type for Messages API SSE events.

use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use just_common::error::TransportError;
use just_common::transport::sse::JsonEventStream;

use crate::types::event::StreamEvent;

/// Stream of Messages API SSE events.
pub struct MessageEventStream {
    inner: JsonEventStream<StreamEvent>,
}

impl MessageEventStream {
    /// Creates a stream from an SSE HTTP response.
    pub fn from_response(response: reqwest::Response) -> Result<Self, TransportError> {
        Ok(Self {
            inner: JsonEventStream::from_response(response)?,
        })
    }
}

impl fmt::Debug for MessageEventStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageEventStream").finish_non_exhaustive()
    }
}

impl Stream for MessageEventStream {
    type Item = Result<StreamEvent, TransportError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}
