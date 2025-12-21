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
For complete metric definitions, instrumentation patterns, and alerting rules, see `metrics-reference.md`.

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

## Related documentation

- **Prometheus metrics reference**: `metrics-reference.md` (metric definitions, instrumentation, alerting rules)
- **Zenoh monitoring**: `../infrastructure/zenoh-event-bus.md` (key expression debugging, subscriber health, Zenoh-specific metrics)
- **Performance tuning metrics**: `../cqrs/performance-tuning.md` (broadcast lags, batching, rate limiting)
- **SSE debugging**: `../cqrs/sse-connection-lifecycle.md` (connection phases, Last-Event-ID)
- **Cache invalidation**: `../infrastructure/analytics-cache-architecture.md` (Pattern 4: Zenoh-based invalidation)
- **Event sourcing pipeline**: `../cqrs/event-sourcing-core.md` (command -> event -> projection flow)
