use crate::providers::error::ProviderError;
use bytes::Bytes;
use futures::stream::Stream;
use pin_project::pin_project;
use serde_json::Value;
use std::pin::Pin;
use std::task::{Context, Poll};

/// SSE event from provider
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
}

impl SseEvent {
    /// Format as SSE output for client
    #[allow(dead_code)]
    pub fn to_sse_string(&self) -> String {
        let mut output = String::new();

        if let Some(ref event_type) = self.event {
            output.push_str(&format!("event: {}\n", event_type));
        }

        output.push_str(&format!("data: {}\n\n", self.data));
        output
    }
}

/// Parse SSE events from a byte stream
pub fn parse_sse_events(input: &str) -> Vec<SseEvent> {
    let mut events = Vec::new();
    let mut current_event: Option<String> = None;
    let mut current_data = String::new();

    for line in input.lines() {
        if line.is_empty() {
            // Empty line marks end of event
            if !current_data.is_empty() {
                events.push(SseEvent {
                    event: current_event.take(),
                    data: std::mem::take(&mut current_data),
                });
            }
        } else if let Some(data) = line.strip_prefix("data: ") {
            if !current_data.is_empty() {
                current_data.push('\n');
            }
            current_data.push_str(data);
        } else if let Some(event) = line.strip_prefix("event: ") {
            current_event = Some(event.to_string());
        }
        // Ignore other fields like "id:", "retry:", etc.
    }

    // Handle case where stream doesn't end with empty line
    if !current_data.is_empty() {
        events.push(SseEvent {
            event: current_event,
            data: current_data,
        });
    }

    events
}

/// Stream adapter that converts a reqwest response body into SSE events.
///
/// Transport errors become [`ProviderError::HttpError`]; invalid or truncated
/// UTF-8 becomes a `502` [`ProviderError::ApiError`]. The stream ends after its
/// first error. Bytes split inside a valid UTF-8 code point are retained.
///
/// ```
/// use bytes::Bytes;
/// use futures::StreamExt;
/// use substrate_gateway::providers::error::ProviderError;
/// use substrate_gateway::providers::streaming::{SseEvent, SseStream};
///
/// let chunks = futures::stream::iter([
///     Ok::<_, reqwest::Error>(Bytes::from_static(b"data: hello\n\n")),
/// ]);
/// let events: Vec<Result<SseEvent, ProviderError>> =
///     futures::executor::block_on(SseStream::new(chunks).collect());
/// assert_eq!(events[0].as_ref().unwrap().data, "hello");
/// ```
#[pin_project]
pub struct SseStream<S> {
    #[pin]
    inner: S,
    buffer: Vec<u8>,
    ended: bool,
    limits: bool,
    wire_bytes: usize,
    pending: Bytes,
    /// Queue of parsed events waiting to be emitted
    event_queue: std::collections::VecDeque<SseEvent>,
}

impl<S> SseStream<S> {
    /// Responses-local bounded byte framing. Other protocol construction is unchanged.
    pub fn responses(stream: S) -> Self {
        let mut value = Self::new(stream);
        value.limits = true;
        value
    }
    pub fn new(stream: S) -> Self {
        Self {
            inner: stream,
            buffer: Vec::new(),
            ended: false,
            limits: false,
            wire_bytes: 0,
            pending: Bytes::new(),
            event_queue: std::collections::VecDeque::new(),
        }
    }
}

impl<S> Stream for SseStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>>,
{
    type Item = Result<SseEvent, ProviderError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        if let Some(event) = this.event_queue.pop_front() {
            return Poll::Ready(Some(Ok(event)));
        }
        if *this.ended {
            return Poll::Ready(None);
        }

        if *this.limits {
            use crate::models::response_transport::{PAYLOAD_LIMIT, WIRE_LIMIT};
            loop {
                if !this.pending.is_empty() {
                    // One frame at a time: a huge transport chunk never becomes a huge
                    // copied buffer or an unbounded event queue.
                    while !this.pending.is_empty() {
                        let count = this
                            .pending
                            .iter()
                            .position(|b| *b == b'\n')
                            .map_or(this.pending.len(), |i| i + 1);
                        if crate::models::response_transport::bounded_add(
                            this.buffer.len(),
                            count,
                            PAYLOAD_LIMIT,
                        )
                        .is_err()
                        {
                            *this.ended = true;
                            this.buffer.clear();
                            *this.pending = Bytes::new();
                            return Poll::Ready(Some(Err(ProviderError::ApiError {
                                status: 502,
                                message: "Responses frame limit exceeded".into(),
                            })));
                        }
                        this.buffer.extend_from_slice(&this.pending[..count]);
                        *this.pending = this.pending.slice(count..);
                        if this.buffer.ends_with(b"\n\n") || this.buffer.ends_with(b"\r\n\r\n") {
                            let events = match std::str::from_utf8(this.buffer) {
                                Ok(text) => parse_sse_events(text),
                                Err(_) => {
                                    *this.ended = true;
                                    this.buffer.clear();
                                    return Poll::Ready(Some(Err(ProviderError::ApiError {
                                        status: 502,
                                        message: "Invalid Responses UTF-8".into(),
                                    })));
                                }
                            };
                            this.buffer.clear();
                            if let Some(event) = events.into_iter().next() {
                                return Poll::Ready(Some(Ok(event)));
                            }
                        }
                    }
                }
                match this.inner.as_mut().poll_next(cx) {
                    Poll::Ready(Some(Ok(bytes))) => {
                        let Ok(total) = crate::models::response_transport::bounded_add(
                            *this.wire_bytes,
                            bytes.len(),
                            WIRE_LIMIT,
                        ) else {
                            *this.ended = true;
                            this.buffer.clear();
                            return Poll::Ready(Some(Err(ProviderError::ApiError {
                                status: 502,
                                message: "Responses wire limit exceeded".into(),
                            })));
                        };
                        *this.wire_bytes = total;
                        *this.pending = bytes;
                    }
                    Poll::Ready(Some(Err(_))) => {
                        *this.ended = true;
                        this.buffer.clear();
                        return Poll::Ready(Some(Err(ProviderError::ApiError {
                            status: 502,
                            message: "Responses body transport failed".into(),
                        })));
                    }
                    Poll::Ready(None) => {
                        *this.ended = true;
                        if !this.buffer.is_empty() {
                            this.buffer.clear();
                            return Poll::Ready(Some(Err(ProviderError::ApiError {
                                status: 502,
                                message: "Truncated Responses frame".into(),
                            })));
                        }
                        return Poll::Ready(None);
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
        loop {
            let end = match this.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    this.buffer.extend_from_slice(&bytes);
                    // Delimiters are ASCII: frame bytes before attempting UTF-8 decoding.
                    let lf = this
                        .buffer
                        .windows(2)
                        .rposition(|w| w == b"\n\n")
                        .map(|i| i + 2);
                    let crlf = this
                        .buffer
                        .windows(4)
                        .rposition(|w| w == b"\r\n\r\n")
                        .map(|i| i + 4);
                    match lf.into_iter().chain(crlf).max() {
                        Some(end) => end,
                        None => continue,
                    }
                }
                Poll::Ready(Some(Err(error))) => {
                    *this.ended = true;
                    this.buffer.clear();
                    return Poll::Ready(Some(Err(ProviderError::HttpError(error))));
                }
                Poll::Ready(None) => {
                    *this.ended = true;
                    this.buffer.len()
                }
                Poll::Pending => return Poll::Pending,
            };
            let events = match std::str::from_utf8(&this.buffer[..end]) {
                Ok(text) => parse_sse_events(text),
                Err(_) => {
                    *this.ended = true;
                    this.buffer.clear();
                    return Poll::Ready(Some(Err(ProviderError::ApiError {
                        status: 502,
                        message: "Invalid UTF-8 in upstream SSE stream".to_string(),
                    })));
                }
            };
            this.buffer.drain(..end);
            this.event_queue.extend(events);
            if let Some(event) = this.event_queue.pop_front() {
                return Poll::Ready(Some(Ok(event)));
            }
            if *this.ended {
                return Poll::Ready(None);
            }
        }
    }
}

/// Stream adapter that logs useful information from SSE events while passing through original bytes
#[pin_project]
pub struct LoggingSseStream<S> {
    #[pin]
    inner: S,
    provider_name: String,
    model_name: String,
    buffer: Vec<u8>,
    logged_message_start: bool,
    start_time: std::time::Instant,
    first_token_time: Option<std::time::Instant>,
    output_tokens: u64,
    input_tokens: u64,
    cache_creation: u64,
    cache_read: u64,
}

impl<S> LoggingSseStream<S> {
    pub fn new(stream: S, provider_name: String, model_name: String) -> Self {
        Self {
            inner: stream,
            provider_name,
            model_name,
            buffer: Vec::new(),
            logged_message_start: false,
            start_time: std::time::Instant::now(),
            first_token_time: None,
            output_tokens: 0,
            input_tokens: 0,
            cache_creation: 0,
            cache_read: 0,
        }
    }
}

impl<S, E> Stream for LoggingSseStream<S>
where
    S: Stream<Item = Result<Bytes, E>>,
{
    type Item = Result<Bytes, E>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.as_mut().project().inner.poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                // Accumulate bytes for parsing and track events
                let this = self.as_mut().project();
                this.buffer.extend_from_slice(&bytes);

                // Clone data we need for event processing
                let buffer_clone = this.buffer.clone();

                // Parse events from accumulated buffer
                if let Ok(text) = std::str::from_utf8(&buffer_clone) {
                    if text.contains("\n\n") {
                        let events = parse_sse_events(text);

                        for event in events {
                            match event.event.as_deref() {
                                Some("message_start") if !*this.logged_message_start => {
                                    // Extract cache stats
                                    if let Ok(json) = serde_json::from_str::<Value>(&event.data) {
                                        if let Some(message) = json.get("message") {
                                            if let Some(usage) = message.get("usage") {
                                                *this.input_tokens = usage
                                                    .get("input_tokens")
                                                    .and_then(|v| v.as_u64())
                                                    .unwrap_or(0);
                                                *this.cache_creation = usage
                                                    .get("cache_creation_input_tokens")
                                                    .and_then(|v| v.as_u64())
                                                    .unwrap_or(0);
                                                *this.cache_read = usage
                                                    .get("cache_read_input_tokens")
                                                    .and_then(|v| v.as_u64())
                                                    .unwrap_or(0);
                                            }
                                        }
                                    }
                                    *this.logged_message_start = true;
                                }
                                Some("content_block_delta")
                                    // Mark first token arrival
                                    if this.first_token_time.is_none() =>
                                {
                                    *this.first_token_time = Some(std::time::Instant::now());
                                }
                                Some("message_delta") => {
                                    // Track tokens (output_tokens always, input_tokens for OpenAI providers)
                                    if let Ok(json) = serde_json::from_str::<Value>(&event.data) {
                                        if let Some(usage) = json.get("usage") {
                                            *this.output_tokens += usage
                                                .get("output_tokens")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0);
                                            // OpenAI providers include input_tokens in message_delta instead of message_start
                                            if let Some(input) =
                                                usage.get("input_tokens").and_then(|v| v.as_u64())
                                            {
                                                if input > 0 && *this.input_tokens == 0 {
                                                    *this.input_tokens = input;
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Keep buffer from growing unbounded
                let this = self.as_mut().project();
                if this.buffer.len() > 1024 * 10 {
                    this.buffer.clear();
                }

                // Pass through original bytes unchanged
                Poll::Ready(Some(Ok(bytes)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                // Stream ended - log final stats
                let this = self.as_ref().project_ref();
                let total_time = this.start_time.elapsed();
                let ttft = this
                    .first_token_time
                    .map(|t| t.duration_since(*this.start_time))
                    .unwrap_or(total_time);

                let tok_per_sec = if total_time.as_secs_f64() > 0.0 && *this.output_tokens > 0 {
                    *this.output_tokens as f64 / total_time.as_secs_f64()
                } else {
                    0.0
                };

                let total_input = *this.input_tokens + *this.cache_creation + *this.cache_read;

                // Build cache info string if caching was used
                let cache_info = if *this.cache_creation > 0 || *this.cache_read > 0 {
                    let cache_pct = (*this.cache_read * 100)
                        .checked_div(total_input)
                        .unwrap_or(0);
                    format!(" cache:{}%", cache_pct)
                } else {
                    String::new()
                };
                // Keep last 2 slash-separated segments for cleaner logs (e.g. "accounts/fireworks/models/x" -> "models/x")
                let model_display: std::borrow::Cow<str> = {
                    let slash_count = this.model_name.matches('/').count();
                    if slash_count >= 2 {
                        let parts: Vec<&str> = this.model_name.rsplitn(3, '/').collect();
                        format!("{}/{}", parts[1], parts[0]).into()
                    } else {
                        this.model_name.as_str().into()
                    }
                };
                tracing::info!(
                    "📊 {}:{} {}ms ttft:{}ms {:.1}t/s out:{} in:{}{}",
                    this.provider_name,
                    model_display,
                    total_time.as_millis(),
                    ttft.as_millis(),
                    tok_per_sec,
                    *this.output_tokens,
                    total_input,
                    cache_info
                );

                // Clear buffer
                self.as_mut().project().buffer.clear();
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_single_event() {
        let input = "event: message\ndata: {\"test\":\"value\"}\n\n";
        let events = parse_sse_events(input);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("message"));
        assert_eq!(events[0].data, "{\"test\":\"value\"}");
    }

    #[test]
    fn test_parse_sse_multiple_events() {
        let input = "event: start\ndata: {\"a\":1}\n\nevent: delta\ndata: {\"b\":2}\n\n";
        let events = parse_sse_events(input);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event.as_deref(), Some("start"));
        assert_eq!(events[1].event.as_deref(), Some("delta"));
    }

    #[test]
    fn test_parse_sse_no_event_type() {
        let input = "data: plain data\n\n";
        let events = parse_sse_events(input);

        assert_eq!(events.len(), 1);
        assert!(events[0].event.is_none());
        assert_eq!(events[0].data, "plain data");
    }

    #[tokio::test]
    async fn gwa05_utf8_survives_every_byte_split_and_one_byte_chunks() {
        use futures::StreamExt;
        let fixture = "event: response.output_text.delta\ndata: {\"delta\":\"café 漢 🙂\"}\n\nevent: response.function_call_arguments.done\ndata: {\"arguments\":\"{\\\"city\\\":\\\"café 漢 🙂\\\"}\"}\n\n";
        let expected = parse_sse_events(fixture);
        for split in 0..=fixture.len() {
            let chunks = vec![
                Ok(Bytes::copy_from_slice(&fixture.as_bytes()[..split])),
                Ok(Bytes::copy_from_slice(&fixture.as_bytes()[split..])),
            ];
            let actual = SseStream::new(futures::stream::iter(chunks))
                .collect::<Vec<_>>()
                .await;
            let actual = actual.into_iter().map(Result::unwrap).collect::<Vec<_>>();
            assert_eq!(actual.len(), expected.len(), "split={split}");
            for (actual, expected) in actual.iter().zip(&expected) {
                assert_eq!(actual.event, expected.event, "split={split}");
                assert_eq!(actual.data, expected.data, "split={split}");
            }
        }
        let chunks = fixture
            .as_bytes()
            .iter()
            .map(|byte| Ok(Bytes::copy_from_slice(&[*byte])))
            .collect::<Vec<_>>();
        let actual = SseStream::new(futures::stream::iter(chunks))
            .collect::<Vec<_>>()
            .await;
        assert_eq!(actual.len(), 2);
        assert_eq!(
            actual[0].as_ref().unwrap().data,
            "{\"delta\":\"café 漢 🙂\"}"
        );
        assert_eq!(
            actual[1].as_ref().unwrap().data,
            "{\"arguments\":\"{\\\"city\\\":\\\"café 漢 🙂\\\"}\"}"
        );
    }

    #[tokio::test]
    async fn gwa05_crlf_multiline_and_unterminated_eof_preserve_existing_framing() {
        use futures::StreamExt;
        let input = "event: delta\r\ndata: café\r\ndata: 漢 🙂\r\n\r\nevent: final\ndata: done";
        for split in 0..=input.len() {
            let chunks = [
                Ok(Bytes::copy_from_slice(&input.as_bytes()[..split])),
                Ok(Bytes::copy_from_slice(&input.as_bytes()[split..])),
            ];
            let events = SseStream::new(futures::stream::iter(chunks))
                .collect::<Vec<_>>()
                .await;
            assert_eq!(events.len(), 2, "split={split}");
            assert_eq!(
                events[0].as_ref().unwrap().data,
                "café\n漢 🙂",
                "split={split}"
            );
            assert_eq!(events[1].as_ref().unwrap().data, "done", "split={split}");
        }
    }

    #[tokio::test]
    async fn gwa05_invalid_and_truncated_utf8_fail_once_without_replacement() {
        use futures::StreamExt;
        for input in [&b"data: \xff\n\n"[..], &b"data: \xf0\x9f"[..]] {
            let source = futures::stream::iter([
                Ok(Bytes::copy_from_slice(input)),
                Ok(Bytes::from_static(b"data: false success\n\n")),
            ]);
            let mut stream = SseStream::new(source);
            let error = stream
                .next()
                .await
                .expect("one explicit error")
                .unwrap_err();
            assert!(matches!(error, ProviderError::ApiError { status: 502, .. }));
            assert!(stream.next().await.is_none());
            assert!(stream.next().await.is_none());
        }
    }

    #[tokio::test]
    async fn gwa05_transport_error_fuses_and_drop_releases_source() {
        use futures::StreamExt;
        use std::sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            Arc,
        };
        struct Source {
            error: Option<reqwest::Error>,
            polls: Arc<AtomicUsize>,
            dropped: Arc<AtomicBool>,
        }
        impl Stream for Source {
            type Item = Result<Bytes, reqwest::Error>;
            fn poll_next(
                mut self: Pin<&mut Self>,
                _: &mut Context<'_>,
            ) -> Poll<Option<Self::Item>> {
                self.polls.fetch_add(1, Ordering::SeqCst);
                Poll::Ready(self.error.take().map(Err))
            }
        }
        impl Drop for Source {
            fn drop(&mut self) {
                self.dropped.store(true, Ordering::SeqCst);
            }
        }
        let error = reqwest::Client::new()
            .get("http://[")
            .send()
            .await
            .unwrap_err();
        let polls = Arc::new(AtomicUsize::new(0));
        let dropped = Arc::new(AtomicBool::new(false));
        let source = Source {
            error: Some(error),
            polls: polls.clone(),
            dropped: dropped.clone(),
        };
        let mut stream = SseStream::new(source);
        assert!(matches!(
            stream.next().await.unwrap(),
            Err(ProviderError::HttpError(_))
        ));
        assert!(stream.next().await.is_none());
        assert_eq!(polls.load(Ordering::SeqCst), 1);
        drop(stream);
        assert!(dropped.load(Ordering::SeqCst));
        let source = Source {
            error: None,
            polls: polls.clone(),
            dropped: dropped.clone(),
        };
        dropped.store(false, Ordering::SeqCst);
        drop(SseStream::new(source));
        assert!(dropped.load(Ordering::SeqCst));
        assert_eq!(
            polls.load(Ordering::SeqCst),
            1,
            "drop/cancel must not poll the source"
        );
    }

    #[tokio::test]
    async fn gwa05_truncated_codepoint_at_eof_is_an_explicit_error() {
        use futures::StreamExt;
        let mut stream = SseStream::new(futures::stream::iter([Ok(Bytes::from_static(
            b"data: \xf0\x9f",
        ))]));
        assert!(matches!(
            stream.next().await.unwrap(),
            Err(ProviderError::ApiError { status: 502, .. })
        ));
        assert!(stream.next().await.is_none());
    }
}
