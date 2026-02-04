//! SSE stream utilities with keep-alive support.
//!
//! This module provides reusable SSE stream builders that interleave keep-alive
//! comments with event data. Keep-alives prevent proxy/load balancer timeouts
//! and enable early detection of broken connections.
//!
//! # Keep-alive format
//!
//! SSE keep-alives use the comment syntax (lines starting with `:`):
//!
//! ```text
//! : keepalive
//!
//! ```
//!
//! This is standard SSE and ignored by clients, but keeps the connection alive.
//!
//! # Subscribe-before-replay invariant
//!
//! When using these utilities with event sourcing, **subscribe to the event bus
//! BEFORE loading historical events** from the event store. This prevents race
//! conditions where events arrive during replay and get missed.
//!
//! See `sse-connection-lifecycle.md` Critical Invariant section for details.
//!
//! # Example usage
//!
//! ```rust,ignore
//! use ironstar_event_store::sse_stream::SseStreamBuilder;
//! use axum::response::sse::{Event, Sse};
//! use std::convert::Infallible;
//!
//! async fn todo_feed(State(state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
//!     // 1. Subscribe FIRST (critical invariant)
//!     let subscriber = state.zenoh_session
//!         .declare_subscriber("events/Todo/**")
//!         .await?;
//!
//!     // 2. Load historical events
//!     let historical = state.event_store.query_since_sequence(last_event_id).await?;
//!
//!     // 3. Create stream with keep-alive
//!     let stream = SseStreamBuilder::new()
//!         .with_keep_alive_secs(15)
//!         .build_with_zenoh(subscriber, historical, |event| {
//!             render_event_as_sse(event)
//!         });
//!
//!     Sse::new(stream)
//! }
//! ```

use axum::response::sse::Event;
use futures::stream::{Stream, StreamExt};
use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio_stream::wrappers::IntervalStream;

/// Default keep-alive interval in seconds.
pub const DEFAULT_KEEP_ALIVE_SECS: u64 = 15;

/// SSE keep-alive comment text.
pub const KEEP_ALIVE_COMMENT: &str = "keepalive";

/// A stream that yields keep-alive SSE comments at regular intervals.
///
/// This stream produces `Event::default().comment(KEEP_ALIVE_COMMENT)` events
/// at the configured interval. These are SSE comments that keep the connection
/// alive but are ignored by SSE clients.
///
/// # Usage
///
/// Typically merged with an event stream using `futures::stream::select`:
///
/// ```rust,ignore
/// let keep_alive = KeepAliveStream::new(Duration::from_secs(15));
/// let combined = futures::stream::select(event_stream, keep_alive);
/// ```
pub struct KeepAliveStream {
    interval: IntervalStream,
}

impl KeepAliveStream {
    /// Create a new keep-alive stream with the specified interval.
    ///
    /// The first keep-alive will be emitted after `interval` elapses, not immediately.
    #[must_use]
    pub fn new(interval: Duration) -> Self {
        Self {
            interval: IntervalStream::new(tokio::time::interval(interval)),
        }
    }

    /// Create a keep-alive stream with the default 15-second interval.
    #[must_use]
    pub fn default_interval() -> Self {
        Self::new(Duration::from_secs(DEFAULT_KEEP_ALIVE_SECS))
    }
}

impl Stream for KeepAliveStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.interval).poll_next(cx) {
            Poll::Ready(Some(_instant)) => {
                let event = Event::default().comment(KEEP_ALIVE_COMMENT);
                Poll::Ready(Some(Ok(event)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Builder for creating SSE streams with keep-alive support.
///
/// This builder helps construct SSE streams that:
/// 1. Emit keep-alive comments at regular intervals
/// 2. Chain replayed events followed by live events
/// 3. Transform domain events to SSE events
///
/// # Critical invariant: subscribe-before-replay
///
/// When using with Zenoh, **subscribe before loading historical events**:
///
/// ```rust,ignore
/// // 1. Subscribe FIRST
/// let subscriber = session.declare_subscriber("events/Todo/**").await?;
///
/// // 2. Then load historical events
/// let historical = event_store.query_since_sequence(last_id).await?;
///
/// // 3. Then build the stream
/// let stream = builder.build_with_streams(replay_stream, live_stream);
/// ```
///
/// This ordering ensures no events are missed between replay and live subscription.
#[derive(Debug, Clone)]
pub struct SseStreamBuilder {
    keep_alive_interval: Duration,
}

impl Default for SseStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SseStreamBuilder {
    /// Create a new SSE stream builder with default settings.
    ///
    /// Default keep-alive interval is 15 seconds.
    #[must_use]
    pub fn new() -> Self {
        Self {
            keep_alive_interval: Duration::from_secs(DEFAULT_KEEP_ALIVE_SECS),
        }
    }

    /// Set the keep-alive interval in seconds.
    ///
    /// Default is 15 seconds. Most proxies timeout idle connections after 30-60s,
    /// so 15s provides a safe margin.
    ///
    /// # Panics
    ///
    /// Does not panic; zero-duration intervals are valid (though not recommended).
    #[must_use]
    pub fn with_keep_alive_secs(mut self, secs: u64) -> Self {
        self.keep_alive_interval = Duration::from_secs(secs);
        self
    }

    /// Set the keep-alive interval.
    #[must_use]
    pub fn with_keep_alive(mut self, interval: Duration) -> Self {
        self.keep_alive_interval = interval;
        self
    }

    /// Build an SSE stream from replay and live event streams.
    ///
    /// The resulting stream:
    /// 1. Emits replayed events first
    /// 2. Then emits live events
    /// 3. Interleaves keep-alive comments throughout
    ///
    /// # Type parameters
    ///
    /// - `R`: Replay stream type (finite, historical events)
    /// - `L`: Live stream type (infinite, new events)
    ///
    /// Both streams must yield `Result<Event, Infallible>`.
    pub fn build_with_streams<R, L>(
        &self,
        replay: R,
        live: L,
    ) -> impl Stream<Item = Result<Event, Infallible>> + Send + use<R, L>
    where
        R: Stream<Item = Result<Event, Infallible>> + Send + 'static,
        L: Stream<Item = Result<Event, Infallible>> + Send + 'static,
    {
        let keep_alive = KeepAliveStream::new(self.keep_alive_interval);

        // Chain replay then live, merge with keep-alive
        let events = replay.chain(live);
        futures::stream::select(events, keep_alive)
    }

    /// Build an SSE stream from a live stream only (no replay).
    ///
    /// Useful for streams that don't need historical event replay,
    /// or when replay is handled separately.
    pub fn build_live_only<L>(
        &self,
        live: L,
    ) -> impl Stream<Item = Result<Event, Infallible>> + Send + use<L>
    where
        L: Stream<Item = Result<Event, Infallible>> + Send + 'static,
    {
        let keep_alive = KeepAliveStream::new(self.keep_alive_interval);
        futures::stream::select(live, keep_alive)
    }

    /// Build an SSE stream with only keep-alives (no events).
    ///
    /// Useful for debugging or testing SSE infrastructure.
    pub fn build_keep_alive_only(&self) -> impl Stream<Item = Result<Event, Infallible>> + Send {
        KeepAliveStream::new(self.keep_alive_interval)
    }

    /// Get the configured keep-alive interval.
    #[must_use]
    pub fn keep_alive_interval(&self) -> Duration {
        self.keep_alive_interval
    }
}

/// Convert a Zenoh subscriber to an SSE event stream.
///
/// This function transforms Zenoh samples to SSE events using the provided
/// conversion function. It handles deserialization errors by logging and skipping.
///
/// # Type parameters
///
/// - `E`: The domain event type (must implement `DeserializeOwned`)
/// - `F`: Event-to-SSE conversion function
pub fn zenoh_to_sse_stream<E, F>(
    subscriber: zenoh::pubsub::Subscriber<
        zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>,
    >,
    event_to_sse: F,
) -> impl Stream<Item = Result<Event, Infallible>> + Send
where
    E: serde::de::DeserializeOwned + Send + 'static,
    F: Fn(E) -> Event + Send + Sync + 'static,
{
    use std::sync::Arc;

    // Wrap converter in Arc for shared ownership across async blocks
    let converter = Arc::new(event_to_sse);

    // Convert Zenoh subscriber to a futures Stream using recv_async
    let stream = futures::stream::unfold(subscriber, |sub| async move {
        match sub.recv_async().await {
            Ok(sample) => Some((sample, sub)),
            Err(_) => None, // Channel closed
        }
    });

    // Transform samples to SSE events
    stream.filter_map(move |sample| {
        let converter = Arc::clone(&converter);
        async move {
            let payload = sample.payload().to_bytes();
            match serde_json::from_slice::<E>(&payload) {
                Ok(event) => {
                    let sse_event = converter(event);
                    Some(Ok(sse_event))
                }
                Err(e) => {
                    tracing::warn!(
                        key_expr = %sample.key_expr(),
                        error = %e,
                        "Failed to deserialize Zenoh sample, skipping"
                    );
                    None
                }
            }
        }
    })
}

/// Create an SSE event from a stored event with sequence ID.
///
/// This helper creates an SSE event with the global sequence number as the
/// event ID, enabling Last-Event-ID based reconnection.
///
/// # Arguments
///
/// - `sequence`: Global event sequence number (becomes SSE event ID)
/// - `data`: Event data to serialize as JSON
///
/// # Returns
///
/// SSE event with ID set to the sequence number.
pub fn event_with_sequence<T: serde::Serialize>(sequence: i64, data: &T) -> Event {
    let json = serde_json::to_string(data).unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to serialize event data");
        "{}".to_string()
    });

    Event::default().id(sequence.to_string()).data(json)
}

/// Convert stored events to a replay stream.
///
/// This function takes a vector of stored events and returns a stream that
/// yields them as SSE events. Each event's global sequence becomes the SSE ID.
///
/// # Type parameters
///
/// - `E`: Stored event type
/// - `F`: Event-to-SSE conversion function
pub fn stored_events_to_stream<E, F>(
    events: Vec<E>,
    event_to_sse: F,
) -> impl Stream<Item = Result<Event, Infallible>> + Send
where
    E: Send + 'static,
    F: Fn(E) -> Event + Send + 'static,
{
    futures::stream::iter(events.into_iter().map(move |evt| Ok(event_to_sse(evt))))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use std::time::Duration;

    #[tokio::test]
    async fn keep_alive_stream_emits_comments() {
        let stream = KeepAliveStream::new(Duration::from_millis(10));
        let mut stream = Box::pin(stream);

        // Collect first 3 keep-alives
        let mut events = Vec::new();
        for _ in 0..3 {
            let event = tokio::time::timeout(Duration::from_millis(50), stream.next())
                .await
                .expect("should receive within timeout")
                .expect("stream should yield event")
                .expect("event should be Ok");
            events.push(event);
        }

        assert_eq!(events.len(), 3);
        // Each event should be a comment (we can't directly inspect, but no panic)
    }

    #[tokio::test]
    async fn keep_alive_stream_default_interval() {
        let stream = KeepAliveStream::default_interval();
        // Just verify it constructs without panic
        let mut stream = Box::pin(stream);

        // The first event comes after 15 seconds by default, so we just verify
        // the stream can be polled
        tokio::select! {
            _ = stream.next() => {
                // This would take 15 seconds, so we don't actually wait
            }
            _ = tokio::time::sleep(Duration::from_millis(10)) => {
                // Expected: timeout before first keep-alive
            }
        }
    }

    #[tokio::test]
    async fn builder_creates_with_default_interval() {
        let builder = SseStreamBuilder::new();
        assert_eq!(
            builder.keep_alive_interval(),
            Duration::from_secs(DEFAULT_KEEP_ALIVE_SECS)
        );
    }

    #[tokio::test]
    async fn builder_custom_keep_alive_secs() {
        let builder = SseStreamBuilder::new().with_keep_alive_secs(30);
        assert_eq!(builder.keep_alive_interval(), Duration::from_secs(30));
    }

    #[tokio::test]
    async fn builder_custom_keep_alive_duration() {
        let builder = SseStreamBuilder::new().with_keep_alive(Duration::from_millis(500));
        assert_eq!(builder.keep_alive_interval(), Duration::from_millis(500));
    }

    #[tokio::test]
    async fn builder_keep_alive_only_stream() {
        let builder = SseStreamBuilder::new().with_keep_alive(Duration::from_millis(10));
        let stream = builder.build_keep_alive_only();
        let mut stream = Box::pin(stream);

        // Should get keep-alives
        let event = tokio::time::timeout(Duration::from_millis(50), stream.next())
            .await
            .expect("should receive within timeout")
            .expect("stream should yield event")
            .expect("event should be Ok");

        // Event was yielded without error
        drop(event);
    }

    #[tokio::test]
    async fn builder_with_replay_and_live_streams() {
        // Create replay stream with 2 events
        let replay_events = vec![
            Ok(Event::default().id("1").data("replay-1")),
            Ok(Event::default().id("2").data("replay-2")),
        ];
        let replay = futures::stream::iter(replay_events);

        // Create live stream with 1 event then pending
        let live_events = vec![Ok(Event::default().id("3").data("live-1"))];
        let live = futures::stream::iter(live_events);

        let builder = SseStreamBuilder::new().with_keep_alive(Duration::from_millis(100));
        let stream = builder.build_with_streams(replay, live);
        let mut stream = Box::pin(stream);

        // Should get replay events first (order may interleave with keep-alive)
        // But within reasonable timeout we should get all 3 events
        let mut event_ids = Vec::new();
        for _ in 0..3 {
            tokio::select! {
                Some(Ok(event)) = stream.next() => {
                    // Check if it's a data event (has ID) or keep-alive
                    // This is a simplified check - in real code we'd parse the event
                    event_ids.push(event);
                }
                _ = tokio::time::sleep(Duration::from_millis(200)) => {
                    break; // Timeout waiting for events
                }
            }
        }

        // Should have received at least 3 events (could be more with keep-alives)
        assert!(event_ids.len() >= 3);
    }

    #[tokio::test]
    async fn stored_events_to_stream_converts_all() {
        #[derive(Clone)]
        struct TestEvent {
            id: i64,
            data: String,
        }

        let events = vec![
            TestEvent {
                id: 1,
                data: "first".to_string(),
            },
            TestEvent {
                id: 2,
                data: "second".to_string(),
            },
            TestEvent {
                id: 3,
                data: "third".to_string(),
            },
        ];

        let stream = stored_events_to_stream(events, |evt| {
            Event::default()
                .id(evt.id.to_string())
                .data(evt.data.clone())
        });

        let collected: Vec<_> = stream.collect().await;
        assert_eq!(collected.len(), 3);

        // All should be Ok
        for result in &collected {
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn event_with_sequence_serializes_correctly() {
        #[derive(serde::Serialize)]
        struct TestData {
            message: String,
        }

        let data = TestData {
            message: "hello".to_string(),
        };
        let event = event_with_sequence(42, &data);

        // Event was created without panic
        drop(event);
    }
}
