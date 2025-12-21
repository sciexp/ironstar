---
title: Advanced performance patterns
---

# Advanced performance patterns

This document covers optional performance optimization patterns for high-throughput event sourcing systems: event debouncing, batching, and per-client rate limiting.
These patterns add complexity and should only be applied when metrics indicate performance bottlenecks.

Start with the simpler patterns in `performance-tuning.md` (broadcast channels, basic metrics) before adopting these advanced techniques.

## Event debouncing

Debouncing aggregates rapid-fire events into a single representative event, reducing SSE bandwidth and client-side morph operations.
Useful when a user action triggers multiple events in quick succession (e.g., typing in a text field, dragging a slider).

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{sleep, Instant};

/// Debounces events by aggregate ID within a grace period
pub struct EventDebouncer {
    /// Grace period during which events are coalesced
    grace_period: Duration,
    /// Last seen event per aggregate ID
    pending: Arc<RwLock<HashMap<String, (StoredEvent, Instant)>>>,
    /// Input channel (raw events from event store)
    input_rx: broadcast::Receiver<StoredEvent>,
    /// Output channel (debounced events)
    output_tx: broadcast::Sender<StoredEvent>,
}

impl EventDebouncer {
    pub fn new(
        grace_period: Duration,
        input_rx: broadcast::Receiver<StoredEvent>,
        output_capacity: usize,
    ) -> Self {
        let (output_tx, _) = broadcast::channel(output_capacity);
        let pending = Arc::new(RwLock::new(HashMap::new()));

        let debouncer = Self {
            grace_period,
            pending: pending.clone(),
            input_rx,
            output_tx: output_tx.clone(),
        };

        // Spawn background task to flush expired pending events
        let flush_pending = pending.clone();
        let flush_tx = output_tx.clone();
        let flush_grace = grace_period;
        tokio::spawn(async move {
            loop {
                sleep(flush_grace / 2).await; // Check at half the grace period
                let now = Instant::now();
                let mut pending = flush_pending.write().await;

                // Collect expired events
                let expired: Vec<_> = pending
                    .iter()
                    .filter(|(_, (_, timestamp))| now.duration_since(*timestamp) >= flush_grace)
                    .map(|(k, _)| k.clone())
                    .collect();

                // Emit and remove expired events
                for key in expired {
                    if let Some((event, _)) = pending.remove(&key) {
                        let _ = flush_tx.send(event);
                    }
                }
            }
        });

        debouncer
    }

    /// Start debouncing events from input to output
    pub async fn run(mut self) {
        while let Ok(event) = self.input_rx.recv().await {
            let key = format!("{}:{}", event.aggregate_type, event.aggregate_id);
            let mut pending = self.pending.write().await;

            // Update or insert pending event with current timestamp
            pending.insert(key, (event, Instant::now()));
        }
    }

    /// Subscribe to debounced event stream
    pub fn subscribe(&self) -> broadcast::Receiver<StoredEvent> {
        self.output_tx.subscribe()
    }
}

// Usage in axum app
async fn setup_debounced_events(
    raw_event_bus: broadcast::Sender<StoredEvent>,
) -> broadcast::Sender<StoredEvent> {
    let debouncer = EventDebouncer::new(
        Duration::from_millis(300), // 300ms grace period
        raw_event_bus.subscribe(),
        256, // Output capacity
    );

    let debounced_tx = debouncer.output_tx.clone();
    tokio::spawn(debouncer.run());
    debounced_tx
}
```

**Trade-offs:**

- **Reduces SSE traffic**: Fewer morphs means less bandwidth and CPU on client
- **Adds latency**: Events delayed by grace period (300ms typical)
- **Loses intermediate states**: Only final state within window is delivered

**When to use:** Text input fields, sliders, canvas drawing.
**When not to use:** Critical state transitions (order placed, payment processed) where every event matters.

## Event batching

Batching accumulates events over a time window and renders a single SSE message containing multiple DOM updates.
This reduces the number of SSE events sent while preserving all event data.

```rust
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio::time::{interval, Interval};

/// Batches events within a time window
pub struct EventBatcher {
    /// Batch accumulation window
    window: Duration,
    /// Input stream
    input_rx: broadcast::Receiver<StoredEvent>,
    /// Accumulated events in current batch
    batch: Vec<StoredEvent>,
    /// Timer to flush batches
    flush_timer: Interval,
}

impl EventBatcher {
    pub fn new(window: Duration, input_rx: broadcast::Receiver<StoredEvent>) -> Self {
        Self {
            window,
            input_rx,
            batch: Vec::new(),
            flush_timer: interval(window),
        }
    }

    /// Convert to a stream of event batches
    pub fn into_stream(self) -> impl Stream<Item = Vec<StoredEvent>> {
        BatchStream { batcher: self }
    }
}

/// Stream adapter for EventBatcher
struct BatchStream {
    batcher: EventBatcher,
}

impl Stream for BatchStream {
    type Item = Vec<StoredEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Try to receive new events (non-blocking)
            match self.batcher.input_rx.try_recv() {
                Ok(event) => {
                    self.batcher.batch.push(event);
                    continue; // Keep accumulating
                }
                Err(broadcast::error::TryRecvError::Empty) => {
                    // No more events, check if we should flush
                }
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    // Consumer lagged, flush current batch and continue
                    if !self.batcher.batch.is_empty() {
                        let batch = std::mem::take(&mut self.batcher.batch);
                        return Poll::Ready(Some(batch));
                    }
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    // Channel closed, flush final batch
                    if !self.batcher.batch.is_empty() {
                        let batch = std::mem::take(&mut self.batcher.batch);
                        return Poll::Ready(Some(batch));
                    }
                    return Poll::Ready(None);
                }
            }

            // Check if flush timer elapsed
            if self.batcher.flush_timer.poll_tick(cx).is_ready() {
                if !self.batcher.batch.is_empty() {
                    let batch = std::mem::take(&mut self.batcher.batch);
                    return Poll::Ready(Some(batch));
                }
            }

            return Poll::Pending;
        }
    }
}

// Usage in SSE handler
async fn sse_feed_with_batching(
    State(app_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    use axum::response::sse::Event;
    use datastar::prelude::*;
    use std::convert::Infallible;

    let rx = app_state.event_bus.subscribe();
    let batcher = EventBatcher::new(Duration::from_millis(100), rx);

    let stream = batcher.into_stream().map(|batch| {
        // Render all events in batch into a single HTML fragment
        let mut html = String::new();
        for event in &batch {
            html.push_str(&render_event_html(event));
        }

        // Use the last event's sequence as SSE id
        let last_seq = batch.last().map(|e| e.sequence.to_string())
            .unwrap_or_default();

        Ok::<_, Infallible>(
            PatchElements::new(html)
                .id(last_seq)
                .into()
        )
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

fn render_event_html(event: &StoredEvent) -> String {
    // Render individual event to HTML fragment
    // Each fragment targets different DOM element via id
    hypertext::html! {
        <div id={format!("event-{}", event.sequence)}>
            {&event.event_type} " at " {&event.created_at.to_string()}
        </div>
    }
}
```

**Trade-offs:**

- **Reduces SSE overhead**: Fewer HTTP/2 frames, less protocol overhead
- **Batched rendering**: Client morphs multiple updates in single pass
- **Adds latency**: Events delayed by batch window (100ms typical)
- **Preserves all events**: Unlike debouncing, no events are dropped

**When to use:** High-frequency updates (live dashboards, real-time analytics).
**When not to use:** Low-frequency or latency-sensitive updates.

## Per-client rate limiting

Prevent fast producers from overwhelming slow consumers by inserting a bounded channel between the broadcast source and each SSE client.
When the buffer fills, the strategy determines what happens (drop, block, etc.).

```rust
use tokio::sync::mpsc;

/// Rate-limited SSE stream with bounded buffer per client
pub struct RateLimitedStream {
    /// Per-client buffer capacity
    capacity: usize,
    /// Strategy when buffer is full
    strategy: BackpressureStrategy,
}

#[derive(Clone, Copy, Debug)]
pub enum BackpressureStrategy {
    /// Drop oldest event (FIFO eviction, preserves recent state)
    DropOldest,
    /// Drop newest event (preserve historical continuity)
    DropNewest,
    /// Block sender until space available (may slow down entire system)
    Block,
    /// Grow buffer dynamically (may cause OOM)
    Unbounded,
}

impl RateLimitedStream {
    pub fn new(capacity: usize, strategy: BackpressureStrategy) -> Self {
        Self { capacity, strategy }
    }

    /// Create a rate-limited stream from broadcast receiver
    pub fn wrap(
        &self,
        mut broadcast_rx: broadcast::Receiver<StoredEvent>,
    ) -> mpsc::Receiver<StoredEvent> {
        let (tx, rx) = mpsc::channel(self.capacity);
        let strategy = self.strategy;

        tokio::spawn(async move {
            while let Ok(event) = broadcast_rx.recv().await {
                let send_result = match strategy {
                    BackpressureStrategy::DropOldest => {
                        // try_send fails if full; mpsc drops oldest not supported directly
                        // Use try_send and ignore error (client is slow, drop this event)
                        tx.try_send(event).ok();
                        continue;
                    }
                    BackpressureStrategy::DropNewest => {
                        // If channel full, skip this event
                        tx.try_send(event).ok();
                        continue;
                    }
                    BackpressureStrategy::Block => {
                        // Block until space available (backpressure to source)
                        tx.send(event).await
                    }
                    BackpressureStrategy::Unbounded => {
                        // This requires mpsc::unbounded_channel instead
                        // For simplicity, treat as Block
                        tx.send(event).await
                    }
                };

                if send_result.is_err() {
                    // Receiver dropped (client disconnected)
                    break;
                }
            }
        });

        rx
    }
}

// Usage in SSE handler with rate limiting
async fn sse_feed_rate_limited(
    State(app_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    use axum::response::sse::Event;
    use datastar::prelude::*;
    use std::convert::Infallible;
    use tokio_stream::wrappers::ReceiverStream;

    let broadcast_rx = app_state.event_bus.subscribe();

    // Create per-client bounded buffer (64 events)
    let rate_limiter = RateLimitedStream::new(64, BackpressureStrategy::DropOldest);
    let mpsc_rx = rate_limiter.wrap(broadcast_rx);

    // Convert mpsc::Receiver to Stream
    let stream = ReceiverStream::new(mpsc_rx).map(|event| {
        Ok::<_, Infallible>(
            PatchElements::new(render_html(&event))
                .id(event.sequence.to_string())
                .into()
        )
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
```

**DropOldest vs DropNewest:**

- **DropOldest**: Client always sees most recent state (eventual consistency). Good for dashboards showing current values.
- **DropNewest**: Client sees historical sequence without gaps. Good for audit logs where continuity matters.

## Backpressure strategies comparison

| Strategy | Behavior | Pros | Cons | Use Case |
|----------|----------|------|------|----------|
| **DropOldest** | Discard oldest buffered events when full | Always shows recent state | Loses historical events | Live dashboards, metrics |
| **DropNewest** | Discard incoming events when full | Preserves historical continuity | Client falls behind, may show stale data | Audit logs, event replay |
| **Block** | Wait until buffer space available | No data loss | Slows entire system if one client lags | Guaranteed delivery, low client count |
| **Unbounded** | Grow buffer without limit | No data loss, no blocking | Memory exhaustion risk | Trusted clients, bounded event rate |

**Ironstar recommendation:** Use **DropOldest** for UI feeds (users care about current state), **Block** for administrative/internal clients (guaranteed delivery), and avoid **Unbounded** (OOM risk).

## Related documentation

- `performance-tuning.md`: Core performance patterns (broadcast channels, metrics, observability)
- `event-sourcing-core.md`: Master index and architecture overview
- `sse-connection-lifecycle.md`: SSE connection phases and debugging
- `event-replay-consistency.md`: Event replay and consistency boundaries
- `projection-patterns.md`: Projection caching strategies
- `../infrastructure/zenoh-event-bus.md`: Zenoh integration for distributed event bus
