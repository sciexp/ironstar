# Observability architecture

Ironstar uses a layered observability stack combining request correlation, structured tracing, and Prometheus metrics to provide visibility into request flow, domain logic, and event store operations.

## Request correlation

Every HTTP request receives a UUID v7 identifier that flows through the entire request lifecycle.

### Middleware stack

Request correlation is implemented via tower-http layers configured in `crates/ironstar/src/presentation/mod.rs:166-201`:

1. `SetRequestIdLayer` — generates UUID v7 request ID or preserves existing `x-request-id` header
2. `TraceLayer` — creates tracing span per request with method, URI, and request_id
3. `PropagateRequestIdLayer` — copies request ID to response header

The middleware uses `MakeRequestUuidV7` (defined in `crates/ironstar/src/presentation/middleware.rs`) to generate time-ordered identifiers instead of random UUIDs.
UUID v7 provides natural chronological sorting in log analysis tools due to its time-ordered prefix.

### Request ID flow

When a request arrives without an `x-request-id` header:
1. `SetRequestIdLayer` calls `MakeRequestUuidV7::make_request_id()` to generate a new UUID v7
2. The request ID is stored in request extensions as `tower_http::request_id::RequestId`
3. `TraceLayer` extracts the ID from headers and records it in the `http.request` span
4. `PropagateRequestIdLayer` copies the ID to the response `x-request-id` header

When a request arrives with an existing `x-request-id` header (from upstream proxy or gateway), the existing value is preserved and propagated.

### Accessing request IDs in logs

The request ID appears in structured log output as the `request_id` field on the `http.request` span.
All events logged within a request handler automatically inherit this field from the parent span context.

## Handler instrumentation

All HTTP handlers use the `#[instrument]` attribute to create tracing spans with consistent naming.
As of s5j.2, 35 handlers across 5 presentation files are instrumented:

- `crates/ironstar/src/presentation/todo.rs` — 7 handlers
- `crates/ironstar/src/presentation/analytics.rs` — 8 handlers
- `crates/ironstar/src/presentation/chart.rs` — handlers for chart pages and SSE
- `crates/ironstar/src/presentation/workspace.rs` — workspace CRUD and SSE handlers
- `crates/ironstar/src/presentation/health.rs` — health check endpoints

### Span naming convention

Handler spans follow the hierarchical naming pattern `handler.{domain}.{operation}`:

- `handler.todo.create` — create todo operation
- `handler.todo.complete` — complete todo operation
- `handler.analytics.feed` — analytics SSE stream
- `handler.catalog.get` — get catalog state
- `handler.query_session.start` — start query session

This convention enables trace filtering by domain (all `handler.todo.*` spans) or operation type (all `*.create` spans).

### Instrumenting a new handler

Add the `#[instrument]` attribute with appropriate span name and field configuration:

```rust
#[instrument(
    name = "handler.domain.operation",
    skip(state, request_body),
    fields(entity_id = %id)
)]
pub async fn my_handler(
    State(state): State<MyState>,
    Path(id): Path<String>,
    Form(request_body): Form<MyInput>,
) -> Result<impl IntoResponse, AppError> {
    // Handler implementation
}
```

The `skip` parameter excludes complex types (state, large request bodies) from span fields.
The `fields` parameter records primitive values or types with `Display` (via `%` prefix).

Handlers that accept extractors like `State`, `Path`, `Form`, or `Json` should skip those parameters to avoid verbose span output.
Only record business-relevant identifiers (IDs, keys) as span fields.

## Event store instrumentation

The event store (defined in `crates/ironstar-event-store/src/event_store.rs`) instruments all operations with tracing spans.

### Span naming pattern

Event store spans follow the pattern `event_store.{operation}`:

- `event_store.fetch_events` — fetch events for a command (fmodel trait method)
- `event_store.fetch_all_by_type` — fetch events across all instances of an aggregate type
- `event_store.fetch_by_aggregate` — fetch events for specific aggregate instance
- `event_store.append` — append events to store (transaction with optimistic locking)
- `event_store.save` — save events (fmodel trait method, delegates to append)
- `event_store.query_all` — query all events for projection rebuild
- `event_store.query_since` — query events since global sequence (SSE reconnection)
- `event_store.earliest_sequence` — get earliest global sequence
- `event_store.latest_sequence` — get latest global sequence
- `event_store.version_provider` — get latest event_id for optimistic locking

### Recorded span fields

Event store spans record the following fields where applicable:

- `aggregate_type` — the aggregate type (e.g., "Todo", "Session")
- `aggregate_id` — the aggregate instance identifier
- `event_count` — number of events fetched or persisted
- `since` — global sequence number for query_since operations

Fields are recorded either at span creation (via `fields()` attribute) or dynamically during execution via `tracing::Span::current().record("field_name", value)`.

### Example trace output

When persisting a todo creation event, the trace contains:

```text
event_store.append aggregate_type="Todo" aggregate_id="todo-abc123" event_count=1
```

When fetching events for a command:

```text
event_store.fetch_events aggregate_type="Todo" aggregate_id="todo-abc123" event_count=3
```

## Domain decider instrumentation

All domain deciders (fmodel-rust Decider pattern) instrument their `decide` and `evolve` functions.

### Span naming pattern

Decider spans follow the pattern `decider.{aggregate}.{function}`:

- `decider.todo.decide` — validate command and emit events
- `decider.todo.evolve` — apply event to state
- `decider.session.decide` — session command validation
- `decider.session.evolve` — session state evolution
- `decider.catalog.decide` — catalog command validation
- `decider.catalog.evolve` — catalog state evolution
- `decider.query_session.decide` — query session command validation
- `decider.query_session.evolve` — query session state evolution

### Log levels

Decider instrumentation uses different log levels for decide and evolve:

- `decide` spans are recorded at **info** level because they represent business decisions
- `evolve` spans are recorded at **trace** level because they fire for every event during replay

This distinction prevents log volume explosion during event replay operations that may process thousands of events.

### Recorded span fields

Decider spans record:

- `command_type` — the command variant being processed (on decide spans)
- `aggregate_type` — the aggregate type (e.g., "Todo")
- `event_count` — number of events produced by decide (recorded after decision)

Example from `crates/ironstar-todo/src/decider.rs:87-94`:

```rust
#[instrument(
    name = "decider.todo.decide",
    skip_all,
    fields(
        command_type = command.command_type(),
        aggregate_type = "Todo",
    )
)]
fn decide(command: &TodoCommand, state: &Option<TodoState>) -> Result<Vec<TodoEvent>, TodoError> {
    // Decision logic
    let result = match (command, state) {
        // ... pattern matching
    };
    if let Ok(ref events) = result {
        tracing::debug!(event_count = events.len(), "decision complete");
    }
    result
}
```

The evolve function at `crates/ironstar-todo/src/decider.rs:186-191` uses `level = "trace"`:

```rust
#[instrument(
    name = "decider.todo.evolve",
    level = "trace",
    skip_all,
    fields(aggregate_type = "Todo")
)]
fn evolve(state: &Option<TodoState>, event: &TodoEvent) -> Option<TodoState> {
    // State evolution logic
}
```

## Prometheus metrics

Ironstar exposes metrics in Prometheus text exposition format via the `/metrics` endpoint.

### Architecture

The metrics stack uses the facade pattern from the `metrics` crate:
- Application code emits metrics via macros (`counter!`, `histogram!`)
- A Prometheus recorder installed at startup accumulates metrics in memory
- The `/metrics` handler renders accumulated metrics on demand

The recorder is initialized in `crates/ironstar/src/infrastructure/metrics.rs:64-72` via `init_prometheus_recorder()`, which returns a `PrometheusHandle`.
This handle is stored in `AppState` and extracted by the metrics handler.

### Metric constants

Metric names are defined as constants in `crates/ironstar/src/infrastructure/metrics.rs:29-45`:

- `HTTP_REQUESTS_TOTAL` — counter with labels: method, path, status
- `HTTP_REQUEST_DURATION_SECONDS` — histogram with labels: method, path
- `EVENTS_PERSISTED_TOTAL` — counter with labels: aggregate_type
- `CACHE_HITS_TOTAL` — counter (analytics cache hits)
- `CACHE_MISSES_TOTAL` — counter (analytics cache misses)
- `QUERY_DURATION_SECONDS` — histogram (query execution duration)

All names follow Prometheus conventions:
- Counters use `_total` suffix
- Histograms use `_seconds` or `_bytes` suffix for units
- Labels use snake_case

### Emitting metrics

Metrics are emitted via the `metrics` crate macros:

```rust
use metrics::counter;

counter!("events_persisted_total", "aggregate_type" => "Todo").increment(1);
```

Histograms use `histogram!`:

```rust
use metrics::histogram;
use std::time::Instant;

let start = Instant::now();
// ... operation
histogram!("query_duration_seconds").record(start.elapsed().as_secs_f64());
```

### Current metric emission sites

As of s5j.5, metrics are emitted at:

- `crates/ironstar-event-store/src/event_store.rs:408-418` — `events_persisted_total` counter incremented after successful event persistence
- Additional metrics (HTTP request counters, cache metrics) are planned but not yet implemented

### Adding a new metric

1. Define the metric constant in `crates/ironstar/src/infrastructure/metrics.rs`
2. Add a description in `describe_metrics()` function
3. Emit the metric at appropriate call sites using `counter!` or `histogram!` macros

Example:

```rust
// In crates/ironstar/src/infrastructure/metrics.rs
pub const COMMAND_PROCESSED_TOTAL: &str = "command_processed_total";

// In describe_metrics()
metrics::describe_counter!(
    COMMAND_PROCESSED_TOTAL,
    metrics::Unit::Count,
    "Total number of commands processed"
);

// At emission site
counter!("command_processed_total", "command_type" => "CreateTodo").increment(1);
```

### Testing with PrometheusHandle

Tests that verify metric behavior use `test_prometheus_handle()` to create a non-global recorder instance.
This prevents test pollution when tests run in parallel:

```rust
use crate::infrastructure::metrics::test_prometheus_handle;

let handle = test_prometheus_handle();
// Emit metrics via global macros
counter!("test_metric").increment(1);
// Render output
let output = handle.render();
```

The test handle captures metrics registered explicitly but does not capture global macro emissions unless the recorder is installed via `init_prometheus_recorder()`.

### Metrics endpoint

The `/metrics` endpoint is defined in `crates/ironstar/src/presentation/metrics.rs:47-58`.
It returns the Prometheus text exposition format with correct `Content-Type: text/plain; version=0.0.4; charset=utf-8` header.

The endpoint handler extracts `MetricsState` (containing `PrometheusHandle`) via axum's `State` extractor using `FromRef<AppState>`.

## Log level guidelines

Use the following log levels to maintain signal-to-noise ratio:

- **error** — unrecoverable failures requiring operator intervention, data corruption, invariant violations
- **warn** — degraded operation that may recover, retryable failures, unexpected but handled conditions
- **info** — request lifecycle events, business events (command accepted, event persisted), service startup/shutdown
- **debug** — detailed flow through application layers, intermediate state, function entry/exit for non-hot-path code
- **trace** — per-event detail during replay, evolve function calls, verbose diagnostics for performance-sensitive code

Handler instrumentation uses info level by default.
Event store operations use info for fetch/append and debug for detailed event counts.
Decider decide uses info, evolve uses trace.

## Development workflow

### Viewing traces locally

Run the process-compose stack (f8b integration) to start all services including Prometheus and Grafana:

```bash
nix run .#dev
```

This starts:
- Ironstar HTTP server on port 3000
- Prometheus scraping `/metrics` every 15 seconds
- Grafana with pre-configured Prometheus datasource

Access points:
- Application: http://localhost:3000
- Metrics endpoint: http://localhost:3000/metrics
- Prometheus UI: http://localhost:9090 (check scrape targets, query metrics)
- Grafana dashboards: http://localhost:3001 (admin:admin credentials for dev)

### Filtering traces

Use `RUST_LOG` environment variable to control trace output:

```bash
# Show all info and above
RUST_LOG=info cargo run

# Show debug for specific crate
RUST_LOG=ironstar_event_store=debug cargo run

# Show trace for decider evolve, info for everything else
RUST_LOG=info,decider=trace cargo run

# Show only spans matching a pattern
RUST_LOG=handler.todo=debug cargo run
```

The hierarchical span naming convention enables precise filtering:
- `handler.*` — all handler spans
- `handler.todo.*` — all todo domain handlers
- `event_store.*` — all event store operations
- `decider.*.decide` — all decide functions (business decisions)

### Debugging request flow

To trace a request end-to-end:

1. Set `RUST_LOG=debug` to see all layers
2. Identify the request ID from the `http.request` span in logs
3. Filter logs by request_id to see the complete request lifecycle
4. Follow the span hierarchy: `http.request` → `handler.domain.operation` → `event_store.append` → `decider.aggregate.decide`

All events within a request context automatically carry the request_id field from the parent span.
