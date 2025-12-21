# Observability decisions

This document covers ironstar's observability architecture: structured logging with the tracing crate, Prometheus metrics for operational visibility, health checks, and development vs production logging configuration.
Observability is critical for debugging CQRS pipelines, diagnosing performance bottlenecks, and maintaining production systems.

## Core principles

**Structured logging over printf debugging**: Use tracing spans and fields for machine-parsable logs instead of formatted strings.

**Metrics-driven tuning**: Instrument key performance indicators (event throughput, SSE connection count, projection lag) to inform configuration decisions.

**Zero-overhead in production**: Use compile-time feature gates (`#[cfg(debug_assertions)]`) to exclude verbose tracing from release builds.

**Context propagation**: Propagate request IDs, session IDs, and aggregate IDs through async spans for end-to-end tracing.

## Structured logging with tracing

Ironstar uses the `tracing` crate for structured logging.
tracing provides span-based context propagation, compile-time field extraction, and pluggable subscribers for different output formats.

### Dependencies

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"

[dev-dependencies]
tracing-test = "0.2"  # For testing with span assertions
```

### Log levels and semantics

| Level | Purpose | Example Use Cases |
|-------|---------|-------------------|
| `TRACE` | Internal implementation details | Function entry/exit, loop iterations |
| `DEBUG` | Development diagnostics | SQL queries, cache hits/misses |
| `INFO` | Operational events | Server started, command processed, event appended |
| `WARN` | Recoverable errors | Slow consumer lag, cache eviction, retry |
| `ERROR` | Unrecoverable errors | Database connection failure, command validation error |

**Ironstar conventions:**

- Use `TRACE` for function-level spans (helps with profiling but excluded from release builds)
- Use `DEBUG` for infrastructure operations (SQL, cache, event bus)
- Use `INFO` for business-level events (command received, event emitted, projection updated)
- Use `WARN` for degraded performance (lag, backpressure, stale cache)
- Use `ERROR` for failures requiring human intervention

### Initialization (development)

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_dev_logging() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    // Default filter: INFO for app, DEBUG for deps, TRACE for internal
                    "ironstar=debug,tower_http=debug,axum=debug,sqlx=warn,zenoh=info".into()
                })
        )
        .with(tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
        )
        .init();
}
```

**Configuration via environment variables:**

```bash
# Override specific modules
RUST_LOG="ironstar::domain=trace,ironstar::infrastructure::event_store=debug" cargo run

# Global verbosity
RUST_LOG="trace" cargo run
```

### Initialization (production)

Production uses JSON output for structured logging, written to rolling log files.

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_prod_logging(log_dir: &str) {
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        log_dir,
        "ironstar.log",
    );

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    // Production default: INFO only
                    "ironstar=info,tower_http=warn,axum=warn,sqlx=error,zenoh=warn".into()
                })
        )
        .with(
            fmt::layer()
                .json()
                .with_file(false)  // Omit file paths in prod
                .with_line_number(false)
                .with_target(true)
                .with_writer(file_appender)
        )
        .init();
}
```

**Log rotation policy:**

- Daily rotation at midnight UTC
- Retain last 30 days of logs
- Compress logs older than 7 days (external logrotate)

### Span context for CQRS pipeline

Propagate context through the command → event → projection pipeline using structured spans.

```rust
use tracing::{info, info_span, warn, Instrument};
use uuid::Uuid;

/// Command handler with full span context
#[tracing::instrument(
    name = "handle_command",
    skip(state, command),
    fields(
        command_id = %Uuid::new_v4(),
        command_type = std::any::type_name::<C>(),
        aggregate_id = tracing::field::Empty,  // Filled later
        user_id = tracing::field::Empty,
    )
)]
pub async fn handle_command<C: Command>(
    state: &AppState,
    command: C,
) -> Result<Vec<Event>, CommandError> {
    let span = tracing::Span::current();

    // Extract aggregate_id from command
    let aggregate_id = command.aggregate_id();
    span.record("aggregate_id", &aggregate_id.to_string());

    info!("Processing command");

    // Load aggregate state
    let events = state.event_store
        .load_events(&aggregate_id)
        .instrument(info_span!("load_events", aggregate_id = %aggregate_id))
        .await?;

    // Apply command to aggregate (pure function)
    let current_state = events.iter().fold(AggregateState::default(), |state, event| {
        state.apply(event)
    });

    let new_events = current_state.handle(command)?;

    // Append new events to event store
    let stored_events = state.event_store
        .append_events(&aggregate_id, &new_events)
        .instrument(info_span!("append_events", event_count = new_events.len()))
        .await?;

    info!(event_count = new_events.len(), "Command processed successfully");

    // Publish to event bus
    for stored_event in &stored_events {
        let _ = state.event_bus.send(stored_event.clone());
    }

    Ok(new_events)
}
```

**Span output example (JSON):**

```json
{
  "timestamp": "2025-12-21T10:30:45.123Z",
  "level": "INFO",
  "target": "ironstar::application::command_handlers",
  "span": {
    "name": "handle_command",
    "command_id": "550e8400-e29b-41d4-a716-446655440000",
    "command_type": "AddTodoCommand",
    "aggregate_id": "todo-123",
    "user_id": "user-456"
  },
  "message": "Command processed successfully",
  "fields": {
    "event_count": 1
  }
}
```

### SSE connection lifecycle spans

Track SSE connections from subscription to disconnection.

```rust
use axum::response::sse::Sse;
use futures::stream::Stream;
use tracing::{debug, info, warn};

#[tracing::instrument(
    name = "sse_connection",
    skip(state),
    fields(
        session_id = %session_id,
        remote_addr = tracing::field::Empty,
        last_event_id = tracing::field::Empty,
    )
)]
pub async fn sse_feed(
    State(state): State<AppState>,
    Extension(session_id): Extension<SessionId>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let span = tracing::Span::current();
    span.record("remote_addr", &addr.to_string());

    // Extract Last-Event-ID for reconnection
    if let Some(last_event_id) = headers.get("Last-Event-ID") {
        if let Ok(id) = last_event_id.to_str() {
            span.record("last_event_id", id);
            info!("SSE reconnection detected");
        }
    }

    info!("SSE connection established");

    let rx = state.event_bus.subscribe();
    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            debug!(event_seq = event.sequence, "Emitting SSE event");
            yield Ok(convert_to_sse(event));
        }
        warn!("SSE stream ended (event bus closed)");
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
```

### Event store operation spans

Instrument SQLite operations for performance analysis.

```rust
use sqlx::SqlitePool;
use tracing::{debug, instrument};

pub struct SqliteEventStore {
    pool: SqlitePool,
}

impl SqliteEventStore {
    #[instrument(skip(self), fields(aggregate_id = %aggregate_id))]
    pub async fn load_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let start = std::time::Instant::now();

        let events = sqlx::query_as!(
            StoredEvent,
            r#"
            SELECT id, aggregate_type, aggregate_id, sequence, event_type, payload, metadata, created_at
            FROM events
            WHERE aggregate_id = ?
            ORDER BY sequence ASC
            "#,
            aggregate_id
        )
        .fetch_all(&self.pool)
        .await?;

        let duration = start.elapsed();
        debug!(
            event_count = events.len(),
            duration_ms = duration.as_millis(),
            "Loaded events from store"
        );

        Ok(events)
    }

    #[instrument(skip(self, events), fields(aggregate_id = %aggregate_id, event_count = events.len()))]
    pub async fn append_events(
        &self,
        aggregate_id: &str,
        events: &[Event],
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let start = std::time::Instant::now();

        // Begin transaction
        let mut tx = self.pool.begin().await?;

        let mut stored_events = Vec::new();
        for event in events {
            let stored = sqlx::query_as!(
                StoredEvent,
                r#"
                INSERT INTO events (aggregate_type, aggregate_id, sequence, event_type, payload, metadata)
                VALUES (?, ?, (SELECT COALESCE(MAX(sequence), 0) + 1 FROM events WHERE aggregate_id = ?), ?, ?, ?)
                RETURNING id, aggregate_type, aggregate_id, sequence, event_type, payload, metadata, created_at
                "#,
                event.aggregate_type(),
                aggregate_id,
                aggregate_id,
                event.event_type(),
                event.to_json()?,
                event.metadata_json()?
            )
            .fetch_one(&mut *tx)
            .await?;

            stored_events.push(stored);
        }

        tx.commit().await?;

        let duration = start.elapsed();
        debug!(
            duration_ms = duration.as_millis(),
            "Events appended successfully"
        );

        Ok(stored_events)
    }
}
```

### Conditional debug spans

Use `#[cfg(debug_assertions)]` to exclude verbose tracing from release builds.

```rust
#[cfg(debug_assertions)]
use tracing::trace_span;

pub fn process_batch(events: Vec<Event>) {
    #[cfg(debug_assertions)]
    let _span = trace_span!("process_batch", batch_size = events.len()).entered();

    for event in events {
        process_event(event);
    }
}
```

This pattern ensures zero runtime overhead for internal tracing in release builds.

## Prometheus metrics

Ironstar exposes a `/metrics` endpoint for Prometheus scraping.
Metrics inform capacity planning, alert thresholds, and performance tuning.

### Dependencies

```toml
[dependencies]
prometheus = { version = "0.13", features = ["process"] }
once_cell = "1.20"
```

### Metrics registry

```rust
use once_cell::sync::Lazy;
use prometheus::{
    CounterVec, GaugeVec, HistogramVec, IntCounter, IntGauge, Registry, Opts, HistogramOpts,
};

/// Global Prometheus registry
pub static METRICS_REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

/// Event store metrics
pub static EVENTS_APPENDED_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new(
        "events_appended_total",
        "Total number of events appended to event store"
    );
    let counter = CounterVec::new(opts, &["aggregate_type"]).unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

pub static APPEND_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "append_duration_seconds",
        "Duration of event append operations"
    )
    .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]);
    let histogram = HistogramVec::new(opts, &["aggregate_type"]).unwrap();
    METRICS_REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

/// SSE connection metrics
pub static SSE_CONNECTIONS: Lazy<IntGauge> = Lazy::new(|| {
    let opts = Opts::new(
        "sse_connections",
        "Current number of active SSE connections"
    );
    let gauge = IntGauge::with_opts(opts).unwrap();
    METRICS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub static SSE_EMIT_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "sse_emit_duration_seconds",
        "Duration of SSE event rendering and emission"
    )
    .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5]);
    let histogram = HistogramVec::new(opts, &["event_type"]).unwrap();
    METRICS_REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

pub static BROADCAST_LAGS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let opts = Opts::new(
        "broadcast_lags_total",
        "Total number of broadcast lag events (slow consumers)"
    );
    let counter = IntCounter::with_opts(opts).unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

/// Projection metrics
pub static PROJECTION_LAG_SECONDS: Lazy<GaugeVec> = Lazy::new(|| {
    let opts = Opts::new(
        "projection_lag_seconds",
        "Time lag between event creation and projection update"
    );
    let gauge = GaugeVec::new(opts, &["projection_name"]).unwrap();
    METRICS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub static PROJECTION_EVENTS_PROCESSED: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new(
        "projection_events_processed_total",
        "Total events processed by each projection"
    );
    let counter = CounterVec::new(opts, &["projection_name"]).unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

/// Cache metrics
pub static CACHE_HIT_RATIO: Lazy<GaugeVec> = Lazy::new(|| {
    let opts = Opts::new(
        "cache_hit_ratio",
        "Cache hit ratio (0.0 to 1.0)"
    );
    let gauge = GaugeVec::new(opts, &["cache_name"]).unwrap();
    METRICS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub static CACHE_EVICTIONS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new(
        "cache_evictions_total",
        "Total number of cache evictions"
    );
    let counter = CounterVec::new(opts, &["cache_name", "reason"]).unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});
```

### Instrumented event store

```rust
use prometheus::Histogram;

impl SqliteEventStore {
    pub async fn append_events(
        &self,
        aggregate_id: &str,
        events: &[Event],
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let timer = APPEND_DURATION_SECONDS
            .with_label_values(&[events.first().unwrap().aggregate_type()])
            .start_timer();

        // Perform append operation
        let stored_events = self.append_events_inner(aggregate_id, events).await?;

        timer.observe_duration();
        EVENTS_APPENDED_TOTAL
            .with_label_values(&[events.first().unwrap().aggregate_type()])
            .inc_by(events.len() as f64);

        Ok(stored_events)
    }
}
```

### Instrumented SSE handler

```rust
use axum::response::sse::Event;

pub async fn sse_feed(
    State(state): State<AppState>,
    Extension(session_id): Extension<SessionId>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    SSE_CONNECTIONS.inc();

    let rx = state.event_bus.subscribe();
    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            let timer = SSE_EMIT_DURATION_SECONDS
                .with_label_values(&[&event.event_type])
                .start_timer();

            yield Ok(convert_to_sse(event));

            timer.observe_duration();
        }

        SSE_CONNECTIONS.dec();
    };

    Sse::new(stream)
}
```

### Instrumented cache

```rust
use moka::future::Cache;

pub struct AnalyticsCache {
    cache: Cache<String, Vec<u8>>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl AnalyticsCache {
    pub async fn get_or_compute<F, Fut>(
        &self,
        key: String,
        compute: F,
    ) -> Result<Vec<u8>, Error>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<u8>, Error>>,
    {
        if let Some(value) = self.cache.get(&key).await {
            self.hits.fetch_add(1, Ordering::Relaxed);
            self.update_hit_ratio();
            return Ok(value);
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        self.update_hit_ratio();

        let value = compute().await?;
        self.cache.insert(key, value.clone()).await;
        Ok(value)
    }

    fn update_hit_ratio(&self) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total > 0 {
            let ratio = hits as f64 / total as f64;
            CACHE_HIT_RATIO
                .with_label_values(&["analytics"])
                .set(ratio);
        }
    }

    pub fn on_eviction(&self, reason: &str) {
        CACHE_EVICTIONS_TOTAL
            .with_label_values(&["analytics", reason])
            .inc();
    }
}
```

### Metrics endpoint

```rust
use axum::{routing::get, Router};
use prometheus::{Encoder, TextEncoder};

async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = METRICS_REGISTRY.gather();
    let mut buffer = Vec::new();

    encoder.encode(&metric_families, &mut buffer).unwrap();

    (
        axum::http::StatusCode::OK,
        [("content-type", encoder.format_type())],
        buffer,
    )
}

pub fn metrics_routes() -> Router<AppState> {
    Router::new().route("/metrics", get(metrics_handler))
}
```

### Key metrics summary

| Metric | Type | Purpose | Alert Threshold |
|--------|------|---------|-----------------|
| `events_appended_total` | Counter | Track write throughput | N/A (informational) |
| `append_duration_seconds` | Histogram | Event store performance | p99 > 100ms (SQLite tuning) |
| `sse_connections` | Gauge | Active client count | > 1000 (scale up) |
| `sse_emit_duration_seconds` | Histogram | SSE rendering performance | p99 > 50ms (optimize templates) |
| `broadcast_lags_total` | Counter | Slow consumer detection | > 10/min (investigate) |
| `projection_lag_seconds` | Gauge | Projection freshness | > 60s (stale projections) |
| `projection_events_processed_total` | Counter | Projection throughput | Decreasing (backlog growing) |
| `cache_hit_ratio` | Gauge | Cache effectiveness | < 0.5 (poor cache utilization) |
| `cache_evictions_total` | Counter | Cache pressure | High rate (increase capacity) |

## Health check endpoint

Health checks enable orchestration systems (Kubernetes, Docker Compose) to detect service readiness and liveness.

### Health check implementation

```rust
use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthCheck {
    status: &'static str,
    version: &'static str,
    uptime_seconds: u64,
    checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    database: CheckStatus,
    event_bus: CheckStatus,
    cache: CheckStatus,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

pub async fn health_check_handler(
    State(state): State<AppState>,
) -> Result<Json<HealthCheck>, StatusCode> {
    let uptime_seconds = state.start_time.elapsed().as_secs();

    // Check database connectivity
    let database_status = match sqlx::query("SELECT 1")
        .fetch_one(&state.event_store.pool)
        .await
    {
        Ok(_) => CheckStatus::Healthy,
        Err(_) => CheckStatus::Unhealthy,
    };

    // Check event bus (subscriber count)
    let event_bus_status = if state.event_bus.receiver_count() > 0 {
        CheckStatus::Healthy
    } else {
        CheckStatus::Degraded  // No subscribers, still operational
    };

    // Check cache (simple ping)
    let cache_status = CheckStatus::Healthy;  // moka has no explicit health check

    let overall_status = match (&database_status, &event_bus_status, &cache_status) {
        (CheckStatus::Unhealthy, _, _) => return Err(StatusCode::SERVICE_UNAVAILABLE),
        _ => "healthy",
    };

    Ok(Json(HealthCheck {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds,
        checks: HealthChecks {
            database: database_status,
            event_bus: event_bus_status,
            cache: cache_status,
        },
    }))
}

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
}

/// Readiness: Can the service accept traffic?
async fn readiness_check(State(state): State<AppState>) -> StatusCode {
    match sqlx::query("SELECT 1").fetch_one(&state.event_store.pool).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

/// Liveness: Is the service running?
async fn liveness_check() -> StatusCode {
    StatusCode::OK
}
```

**Health check responses:**

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": {
    "database": "healthy",
    "event_bus": "healthy",
    "cache": "healthy"
  }
}
```

**Docker Compose integration:**

```yaml
services:
  ironstar:
    image: ironstar:latest
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health/ready"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 10s
```

## Development vs production configuration

### Development configuration

```rust
#[cfg(debug_assertions)]
pub fn init_observability() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ironstar=debug,tower_http=debug,sqlx=warn".into())
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .pretty()
        )
        .init();

    tracing::info!("Development logging initialized");
}
```

### Production configuration

```rust
#[cfg(not(debug_assertions))]
pub fn init_observability(log_dir: &str) {
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        log_dir,
        "ironstar.log",
    );

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ironstar=info,tower_http=warn,sqlx=error".into())
        )
        .with(
            fmt::layer()
                .json()
                .with_file(false)
                .with_line_number(false)
                .with_target(true)
                .with_writer(file_appender)
        )
        .init();

    tracing::info!("Production logging initialized");
}
```

### Unified initialization

```rust
pub fn init_observability_with_config(config: &Config) {
    #[cfg(debug_assertions)]
    init_observability();

    #[cfg(not(debug_assertions))]
    init_observability(&config.log_dir);
}
```

## Zenoh monitoring patterns

When using Zenoh for distributed event bus, additional metrics track pub/sub health.

### Zenoh-specific metrics

```rust
pub static ZENOH_PUBLICATIONS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new(
        "zenoh_publications_total",
        "Total number of Zenoh publications"
    );
    let counter = CounterVec::new(opts, &["key_expr_prefix"]).unwrap();
    METRICS_REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

pub static ZENOH_SUBSCRIBER_COUNT: Lazy<GaugeVec> = Lazy::new(|| {
    let opts = Opts::new(
        "zenoh_subscriber_count",
        "Number of active Zenoh subscribers"
    );
    let gauge = GaugeVec::new(opts, &["key_expr_pattern"]).unwrap();
    METRICS_REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub static ZENOH_SAMPLE_LATENCY_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new(
        "zenoh_sample_latency_seconds",
        "Latency between publication and subscriber receipt"
    )
    .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5]);
    let histogram = HistogramVec::new(opts, &["key_expr_prefix"]).unwrap();
    METRICS_REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});
```

### Instrumented Zenoh publication

```rust
pub async fn publish_event(
    session: &Session,
    event: &StoredEvent,
) -> Result<(), Error> {
    let key_expr = format!("events/{}/{}", event.aggregate_type, event.aggregate_id);
    let prefix = format!("events/{}", event.aggregate_type);

    let timer = ZENOH_SAMPLE_LATENCY_SECONDS
        .with_label_values(&[&prefix])
        .start_timer();

    session.put(&key_expr, serde_json::to_vec(event)?).await?;

    timer.observe_duration();
    ZENOH_PUBLICATIONS_TOTAL
        .with_label_values(&[&prefix])
        .inc();

    Ok(())
}
```

### Monitoring Zenoh subscriber health

| Metric | Healthy | Investigate |
|--------|---------|-------------|
| SSE connection count | Stable | Sudden drops |
| Event latency | <200ms | >500ms |
| Zenoh subscriber count | Matches SSE connections | Mismatch indicates leak |
| Error rate | <0.1% | >1% |
| Memory usage | <50MB per 1000 subscribers | >100MB |

See `../infrastructure/zenoh-event-bus.md` for complete monitoring and debugging patterns.

## Alerting thresholds

Recommended Prometheus alerting rules for production deployments.

### Critical alerts

```yaml
groups:
  - name: ironstar_critical
    interval: 30s
    rules:
      - alert: DatabaseDown
        expr: up{job="ironstar"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Ironstar database is down"
          description: "Database health check failed for more than 1 minute"

      - alert: HighSSEConnectionDrop
        expr: rate(sse_connections[5m]) < -10
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High rate of SSE disconnections"
          description: "More than 10 SSE clients disconnecting per minute"
```

### Warning alerts

```yaml
      - alert: HighProjectionLag
        expr: projection_lag_seconds > 60
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Projection lag exceeds 60 seconds"
          description: "Projection {{ $labels.projection_name }} is lagging behind event stream"

      - alert: LowCacheHitRatio
        expr: cache_hit_ratio < 0.5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Cache hit ratio below 50%"
          description: "Cache {{ $labels.cache_name }} has poor hit ratio, consider tuning"

      - alert: FrequentBroadcastLags
        expr: rate(broadcast_lags_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Frequent broadcast lag events"
          description: "Slow consumers detected, consider rate limiting or capacity increase"
```

## Related documentation

- **Performance tuning metrics**: `../cqrs/performance-tuning.md` (broadcast lags, batching, rate limiting)
- **Zenoh monitoring**: `../infrastructure/zenoh-event-bus.md` (key expression debugging, subscriber health)
- **SSE debugging**: `../cqrs/sse-connection-lifecycle.md` (connection phases, Last-Event-ID)
- **Cache invalidation**: `../infrastructure/analytics-cache-architecture.md` (Pattern 4: Zenoh-based invalidation)
- **Event sourcing pipeline**: `../cqrs/event-sourcing-core.md` (command → event → projection flow)
