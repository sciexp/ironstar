# Performance tuning

This document covers core performance patterns for event sourcing systems: broadcast channel configuration, channel sizing, multiple projections, and observability metrics.
Layer performance controls at channel boundaries with metrics-driven tuning.

For advanced optimization techniques (debouncing, batching, rate limiting), see `performance-advanced-patterns.md`.

## Broadcast channel patterns

**Decision: `tokio::sync::broadcast` with lagged receiver handling and fan-out semantics.**

### Basic implementation

```rust
use std::sync::Arc;
use tokio::sync::broadcast;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    event_store: Arc<EventStore>,
    event_bus: broadcast::Sender<StoredEvent>,
    projections: Arc<Projections>,
}

impl AppState {
    pub fn new(event_store: EventStore, bus_capacity: usize) -> Self {
        let (event_bus, _) = broadcast::channel(bus_capacity);

        Self {
            event_store: Arc::new(event_store),
            event_bus,
            projections: Arc::new(Projections::default()),
        }
    }
}

/// Stored event with sequence number
#[derive(Clone, Debug)]
pub struct StoredEvent {
    pub sequence: i64,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

/// Handling slow consumers (lagged receivers)
async fn sse_stream_with_lag_handling(
    rx: broadcast::Receiver<StoredEvent>,
    event_store: Arc<EventStore>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    use axum::response::sse::Event;
    use datastar::prelude::*;  // ExecuteScript
    use futures::stream::{Stream, StreamExt};
    use std::convert::Infallible;
    use tokio_stream::wrappers::BroadcastStream;
    BroadcastStream::new(rx).filter_map(move |result| {
        let event_store = event_store.clone();
        async move {
            match result {
                Ok(event) => Some(Ok(convert_to_sse(event))),
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    // Slow consumer: replay skipped events from event store
                    // This prevents data loss but adds latency
                    eprintln!("SSE consumer lagged, skipped {} events", skipped);

                    // In practice, you'd fetch the skipped events here
                    // For now, send a signal to reconnect using datastar-rust builder
                    Some(Ok(ExecuteScript::new("window.location.reload()").into()))
                }
                Err(broadcast::error::RecvError::Closed) => None,
            }
        }
    })
}
```

### Channel sizing

**Bus capacity** determines how many events can be buffered before slow consumers are marked as lagged.

```rust
// Conservative: Small buffer, fail fast on slow consumers
broadcast::channel::<StoredEvent>(16)

// Permissive: Large buffer, tolerate slow consumers (uses more memory)
broadcast::channel::<StoredEvent>(1024)

// Ironstar default: 256 events (~1MB assuming 4KB events)
broadcast::channel::<StoredEvent>(256)
```

### Multiple projection types

```rust
use tokio::sync::broadcast;

/// Projections manager supporting multiple projection types
pub struct Projections {
    todo_list: ProjectionManager<TodoListProjection>,
    user_profile: ProjectionManager<UserProfileProjection>,
    analytics: ProjectionManager<AnalyticsProjection>,
}

impl Projections {
    pub async fn init(
        event_store: &EventStore,
        event_bus: broadcast::Sender<StoredEvent>,
    ) -> Result<Self, Error> {
        Ok(Self {
            todo_list: ProjectionManager::init(
                TodoListProjection,
                event_store,
                event_bus.clone(),
            ).await?,
            user_profile: ProjectionManager::init(
                UserProfileProjection,
                event_store,
                event_bus.clone(),
            ).await?,
            analytics: ProjectionManager::init(
                AnalyticsProjection,
                event_store,
                event_bus.clone(),
            ).await?,
        })
    }
}
```

## Advanced performance patterns

For high-throughput scenarios requiring additional optimization beyond broadcast channels and basic metrics, see `performance-advanced-patterns.md` which covers:

- **Event debouncing**: Coalesce rapid-fire events within a grace period to reduce SSE bandwidth
- **Event batching**: Accumulate events over time windows to reduce SSE message count
- **Per-client rate limiting**: Bounded buffers with backpressure strategies (DropOldest, DropNewest, Block)

These patterns add latency and complexity.
Apply them only when metrics indicate performance bottlenecks.

## Metrics for performance monitoring

Observability is critical for tuning performance controls.
Track key metrics to identify bottlenecks and inform configuration changes.

For the complete metrics implementation including Prometheus integration, see the "Metrics implementation" section below.

**Key metrics to monitor:**

| Metric | Type | Purpose | Alert Threshold |
|--------|------|---------|-----------------|
| `events_appended_total` | Counter | Track write throughput | N/A (informational) |
| `sse_connections` | Gauge | Active client count | > 1000 (scale up) |
| `broadcast_lags_total` | Counter | Slow consumer detection | > 10/min (investigate) |
| `append_duration_seconds` | Histogram | Event store performance | p99 > 100ms (SQLite tuning) |
| `sse_emit_duration_seconds` | Histogram | SSE rendering performance | p99 > 50ms (optimize templates) |
| `projection_lag_events` | Gauge | Projection freshness | > 100 (investigate) |
| `rate_limit_drops_total` | Counter | Rate limit effectiveness | High rate (adjust capacity) |
| `batch_sizes` | Histogram | Batching effectiveness | p50 < 2 (disable batching) |

**Dashboard recommendations:**

- Graph `sse_connections` over time to understand load patterns
- Alert on `projection_lag_events` exceeding threshold (stale projections)
- Alert on increasing `broadcast_lags_total` (system-wide slowdown)
- Use `append_duration_seconds` p99 to detect SQLite contention

**Cargo dependencies for metrics:**

```toml
[dependencies]
prometheus = { version = "0.13", features = ["process"] }
```

**Exporting metrics:**

```rust
use axum::{routing::get, Router};
use prometheus::{Encoder, TextEncoder};

async fn metrics_handler(
    State(registry): State<Arc<prometheus::Registry>>,
) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    (
        axum::http::StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        buffer,
    )
}

// Add to router
let app = Router::new()
    .route("/metrics", get(metrics_handler))
    .with_state(Arc::new(prometheus_registry));
```

Expose `/metrics` endpoint for Prometheus scraping, enabling dashboards in Grafana or similar tools.

## Related documentation

- `event-sourcing-core.md`: Master index and architecture overview
- `sse-connection-lifecycle.md`: SSE connection phases and debugging
- `event-replay-consistency.md`: Event replay and consistency boundaries
- `projection-patterns.md`: Projection caching strategies
- `command-write-patterns.md`: Command handlers and write path
- `../infrastructure/zenoh-event-bus.md`: Zenoh integration for distributed event bus
