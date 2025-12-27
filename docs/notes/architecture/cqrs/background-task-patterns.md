---
title: Background task patterns
---

# Background task patterns

In CQRS architectures, command handlers must remain responsive and non-blocking.
Long-running operations like DuckDB analytics queries, report generation, or external API calls would violate this constraint if executed synchronously within the command handler.
The spawn-after-persist pattern separates command acceptance from async work execution, enabling immediate user feedback while background tasks complete asynchronously.

This pattern is essential when:
- DuckDB queries may take seconds to complete over large datasets
- Report generation requires multiple processing steps
- External API calls have unpredictable latency
- Operations require retry logic or timeout handling

## The spawn-after-persist pattern

The core pattern has three phases:

1. **Validate and persist**: Command handler validates the request, emits an event (e.g., `QueryStarted`), appends to event store, returns success immediately
2. **Spawn background task**: After successful persistence, spawn a tokio task to perform the async work
3. **Emit completion events**: Background task publishes `QueryCompleted` or `QueryFailed` events via Zenoh when work finishes

```rust
use uuid::Uuid;
use tokio::task;
use crate::{
    domain::{Command, Event, TodoAggregate},
    infrastructure::{EventStore, ZenohEventBus},
};

async fn handle_execute_query_command(
    cmd: ExecuteQueryCommand,
    event_store: EventStore,
    event_bus: ZenohEventBus,
) -> Result<Uuid, CommandError> {
    // Phase 1: Validate and persist
    let query_id = Uuid::new_v4();
    let event = Event::QueryStarted {
        query_id,
        sql: cmd.sql.clone(),
        requested_by: cmd.user_id,
        timestamp: Utc::now(),
    };

    event_store.append("Query", &query_id.to_string(), &event).await?;
    event_bus.publish(&event).await?;

    // Phase 2: Spawn background task
    task::spawn(async move {
        execute_query_background(query_id, cmd.sql, event_store, event_bus).await
    });

    // Return immediately with query ID for tracking
    Ok(query_id)
}
```

The command handler returns a correlation ID (UUID) that clients use to subscribe to completion events via SSE.

## Task lifecycle management

Background tasks require careful lifecycle management to avoid resource leaks and zombie processes.

### Spawning with context

Spawn tasks with all necessary context cloned into the task closure.
Avoid sharing mutable state across the spawn boundary.

```rust
async fn spawn_query_task(
    query_id: Uuid,
    sql: String,
    event_store: EventStore,
    event_bus: ZenohEventBus,
    duckdb_pool: DuckDbPool,
) -> task::JoinHandle<()> {
    task::spawn(async move {
        match execute_duckdb_query(&duckdb_pool, &sql).await {
            Ok(result) => {
                let event = Event::QueryCompleted {
                    query_id,
                    row_count: result.len(),
                    result_hash: hash_result(&result),
                    timestamp: Utc::now(),
                };

                // Store result in cache
                store_query_result(query_id, result).await;

                // Emit completion event
                event_store.append("Query", &query_id.to_string(), &event).await.ok();
                event_bus.publish(&event).await.ok();
            }
            Err(err) => {
                let event = Event::QueryFailed {
                    query_id,
                    error_message: err.to_string(),
                    timestamp: Utc::now(),
                };

                event_store.append("Query", &query_id.to_string(), &event).await.ok();
                event_bus.publish(&event).await.ok();
            }
        }
    })
}
```

### Cancellation handling

Use tokio's `CancellationToken` to support graceful shutdown and user-initiated cancellation.

```rust
use tokio_util::sync::CancellationToken;

async fn spawn_cancellable_query(
    query_id: Uuid,
    sql: String,
    cancellation_token: CancellationToken,
    event_store: EventStore,
    event_bus: ZenohEventBus,
    duckdb_pool: DuckDbPool,
) -> task::JoinHandle<()> {
    task::spawn(async move {
        tokio::select! {
            result = execute_duckdb_query(&duckdb_pool, &sql) => {
                // Normal completion path
                handle_query_result(query_id, result, event_store, event_bus).await;
            }
            _ = cancellation_token.cancelled() => {
                // Cancellation path
                let event = Event::QueryCancelled {
                    query_id,
                    timestamp: Utc::now(),
                };
                event_store.append("Query", &query_id.to_string(), &event).await.ok();
                event_bus.publish(&event).await.ok();
            }
        }
    })
}
```

Clients can cancel queries by sending a `CancelQueryCommand` that triggers the cancellation token.

### Task registry for tracking

Maintain a registry of active background tasks for observability and cleanup.

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct TaskRegistry {
    tasks: RwLock<HashMap<Uuid, task::JoinHandle<()>>>,
}

impl TaskRegistry {
    pub async fn register(&self, id: Uuid, handle: task::JoinHandle<()>) {
        self.tasks.write().await.insert(id, handle);
    }

    pub async fn cancel(&self, id: Uuid) -> Result<(), TaskError> {
        let mut tasks = self.tasks.write().await;
        if let Some(handle) = tasks.remove(&id) {
            handle.abort();
            Ok(())
        } else {
            Err(TaskError::NotFound)
        }
    }

    pub async fn cleanup_finished(&self) {
        let mut tasks = self.tasks.write().await;
        tasks.retain(|_, handle| !handle.is_finished());
    }
}
```

Run periodic cleanup to remove completed task handles and prevent unbounded growth.

## Error handling in background tasks

Background tasks cannot propagate errors via `Result` return types since they execute asynchronously.
Instead, emit error events that flow through the same CQRS pipeline as success events.

### Error categorization

Distinguish between retriable transient errors and permanent failures.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryError {
    /// Transient errors that may succeed on retry
    Transient { message: String, retry_after: Duration },

    /// Permanent errors (invalid SQL, permission denied)
    Permanent { message: String, error_code: String },

    /// Timeout errors
    Timeout { duration: Duration },
}

impl Event {
    pub fn query_failed(query_id: Uuid, error: QueryError) -> Self {
        Event::QueryFailed {
            query_id,
            error,
            timestamp: Utc::now(),
        }
    }
}
```

### Retry logic with exponential backoff

Implement retry logic for transient failures within the background task.

```rust
use tokio::time::{sleep, Duration};

async fn execute_with_retry(
    query_id: Uuid,
    sql: String,
    duckdb_pool: DuckDbPool,
    max_retries: u32,
) -> Result<Vec<Row>, QueryError> {
    let mut retry_count = 0;
    let mut backoff = Duration::from_millis(100);

    loop {
        match execute_duckdb_query(&duckdb_pool, &sql).await {
            Ok(result) => return Ok(result),
            Err(err) if is_retriable(&err) && retry_count < max_retries => {
                retry_count += 1;
                sleep(backoff).await;
                backoff *= 2; // Exponential backoff
            }
            Err(err) => return Err(classify_error(err)),
        }
    }
}

fn is_retriable(err: &DuckDbError) -> bool {
    matches!(err,
        DuckDbError::ConnectionLost |
        DuckDbError::Timeout |
        DuckDbError::ResourceExhausted
    )
}
```

### Error event propagation

Emit detailed error events that preserve error context for debugging while providing user-friendly messages.

```rust
async fn handle_query_error(
    query_id: Uuid,
    error: QueryError,
    event_store: EventStore,
    event_bus: ZenohEventBus,
) {
    // Log detailed error for debugging
    tracing::error!(
        query_id = %query_id,
        error = ?error,
        "Query execution failed"
    );

    // Emit user-facing error event
    let event = Event::QueryFailed {
        query_id,
        error: error.sanitize_for_user(),
        timestamp: Utc::now(),
    };

    if let Err(e) = event_store.append("Query", &query_id.to_string(), &event).await {
        tracing::error!(
            query_id = %query_id,
            error = ?e,
            "Failed to persist QueryFailed event"
        );
    }

    if let Err(e) = event_bus.publish(&event).await {
        tracing::error!(
            query_id = %query_id,
            error = ?e,
            "Failed to publish QueryFailed event"
        );
    }
}
```

Note that errors during event persistence or publishing are logged but not propagated, since there is no caller to receive them.

## Event emission from background context

Background tasks publish events via Zenoh to notify subscribers of progress and completion.

### Event publishing pattern

Clone the event bus handle into the background task and publish events as the task progresses.

```rust
async fn execute_query_background(
    query_id: Uuid,
    sql: String,
    event_store: EventStore,
    event_bus: ZenohEventBus,
    duckdb_pool: DuckDbPool,
) {
    // Emit progress event (optional)
    let progress_event = Event::QueryProgress {
        query_id,
        stage: "Parsing SQL".to_string(),
        timestamp: Utc::now(),
    };
    event_bus.publish(&progress_event).await.ok();

    // Execute query
    match execute_duckdb_query(&duckdb_pool, &sql).await {
        Ok(result) => {
            // Emit completion event
            let completion_event = Event::QueryCompleted {
                query_id,
                row_count: result.len(),
                result_hash: hash_result(&result),
                timestamp: Utc::now(),
            };

            event_store.append("Query", &query_id.to_string(), &completion_event).await.ok();
            event_bus.publish(&completion_event).await.ok();
        }
        Err(err) => {
            handle_query_error(query_id, err, event_store, event_bus).await;
        }
    }
}
```

### Zenoh key expression for background events

Use hierarchical key expressions to enable fine-grained subscriptions.

```rust
impl ZenohEventBus {
    pub async fn publish_query_event(&self, event: &Event) -> Result<(), ZenohError> {
        let key = match event {
            Event::QueryStarted { query_id, .. } =>
                format!("events/Query/{}/started", query_id),
            Event::QueryProgress { query_id, .. } =>
                format!("events/Query/{}/progress", query_id),
            Event::QueryCompleted { query_id, .. } =>
                format!("events/Query/{}/completed", query_id),
            Event::QueryFailed { query_id, .. } =>
                format!("events/Query/{}/failed", query_id),
            _ => return Err(ZenohError::InvalidEventType),
        };

        let payload = serde_json::to_vec(event)?;
        self.session.put(&key, payload).await?;
        Ok(())
    }
}
```

Clients subscribe to `events/Query/{query_id}/**` to receive all events for a specific query, or `events/Query/*/completed` to track all query completions.

### SSE streaming of background events

Integrate background events into SSE streams using Zenoh subscriptions.

```rust
use axum::response::sse::{Event as SseEvent, Sse};
use futures::stream::{Stream, StreamExt};

async fn query_status_stream(
    query_id: Uuid,
    event_bus: ZenohEventBus,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let key_expr = format!("events/Query/{}/**", query_id);
    let subscriber = event_bus.subscribe(&key_expr).await.unwrap();

    let stream = subscriber.map(move |sample| {
        let event: Event = serde_json::from_slice(&sample.payload.to_bytes()).unwrap();
        let sse_event = SseEvent::default()
            .event(event.event_type())
            .data(serde_json::to_string(&event).unwrap());
        Ok(sse_event)
    });

    Sse::new(stream)
}
```

Clients connect to `/api/queries/{query_id}/stream` and receive real-time updates as the background task progresses.

## Example: DuckDB query execution

Complete example demonstrating the spawn-after-persist pattern for DuckDB analytics queries.

### Command handler

```rust
use axum::{Extension, Json};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ExecuteQueryRequest {
    pub sql: String,
}

#[derive(Debug, Serialize)]
pub struct ExecuteQueryResponse {
    pub query_id: Uuid,
}

pub async fn execute_query_handler(
    Extension(event_store): Extension<EventStore>,
    Extension(event_bus): Extension<ZenohEventBus>,
    Extension(duckdb_pool): Extension<DuckDbPool>,
    Extension(task_registry): Extension<TaskRegistry>,
    Json(req): Json<ExecuteQueryRequest>,
) -> Result<Json<ExecuteQueryResponse>, ApiError> {
    // Validate SQL (basic sanity check)
    validate_sql(&req.sql)?;

    // Generate query ID
    let query_id = Uuid::new_v4();

    // Emit QueryStarted event
    let event = Event::QueryStarted {
        query_id,
        sql: req.sql.clone(),
        timestamp: Utc::now(),
    };

    event_store.append("Query", &query_id.to_string(), &event).await?;
    event_bus.publish(&event).await?;

    // Spawn background task
    let handle = task::spawn({
        let sql = req.sql.clone();
        let event_store = event_store.clone();
        let event_bus = event_bus.clone();
        let duckdb_pool = duckdb_pool.clone();

        async move {
            execute_query_background(
                query_id,
                sql,
                event_store,
                event_bus,
                duckdb_pool,
            ).await
        }
    });

    // Register task for tracking
    task_registry.register(query_id, handle).await;

    // Return immediately
    Ok(Json(ExecuteQueryResponse { query_id }))
}
```

### Background execution logic

```rust
async fn execute_query_background(
    query_id: Uuid,
    sql: String,
    event_store: EventStore,
    event_bus: ZenohEventBus,
    duckdb_pool: DuckDbPool,
) {
    tracing::info!(query_id = %query_id, "Starting DuckDB query execution");

    // Execute with timeout
    let timeout_duration = Duration::from_secs(30);
    let result = tokio::time::timeout(
        timeout_duration,
        execute_duckdb_query(&duckdb_pool, &sql),
    ).await;

    match result {
        Ok(Ok(rows)) => {
            tracing::info!(
                query_id = %query_id,
                row_count = rows.len(),
                "Query completed successfully"
            );

            // Store result in cache
            let result_hash = store_query_result(query_id, &rows).await;

            // Emit completion event
            let event = Event::QueryCompleted {
                query_id,
                row_count: rows.len(),
                result_hash,
                timestamp: Utc::now(),
            };

            persist_and_publish(event, &event_store, &event_bus).await;
        }
        Ok(Err(err)) => {
            tracing::error!(
                query_id = %query_id,
                error = ?err,
                "Query execution failed"
            );

            let event = Event::QueryFailed {
                query_id,
                error: QueryError::Permanent {
                    message: err.to_string(),
                    error_code: err.error_code(),
                },
                timestamp: Utc::now(),
            };

            persist_and_publish(event, &event_store, &event_bus).await;
        }
        Err(_) => {
            tracing::error!(
                query_id = %query_id,
                timeout = ?timeout_duration,
                "Query timed out"
            );

            let event = Event::QueryFailed {
                query_id,
                error: QueryError::Timeout {
                    duration: timeout_duration,
                },
                timestamp: Utc::now(),
            };

            persist_and_publish(event, &event_store, &event_bus).await;
        }
    }
}

async fn persist_and_publish(
    event: Event,
    event_store: &EventStore,
    event_bus: &ZenohEventBus,
) {
    let query_id = event.query_id();

    if let Err(e) = event_store.append("Query", &query_id.to_string(), &event).await {
        tracing::error!(
            query_id = %query_id,
            error = ?e,
            "Failed to persist event"
        );
    }

    if let Err(e) = event_bus.publish(&event).await {
        tracing::error!(
            query_id = %query_id,
            error = ?e,
            "Failed to publish event"
        );
    }
}
```

### Frontend integration

Client subscribes to query events via SSE and updates UI as events arrive.

```typescript
async function executeQuery(sql: string) {
    // Submit query command
    const response = await fetch('/api/queries', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sql }),
    });

    const { query_id } = await response.json();

    // Subscribe to query events
    const eventSource = new EventSource(`/api/queries/${query_id}/stream`);

    eventSource.addEventListener('QueryProgress', (e) => {
        const data = JSON.parse(e.data);
        updateProgressUI(data.stage);
    });

    eventSource.addEventListener('QueryCompleted', (e) => {
        const data = JSON.parse(e.data);
        displayResults(query_id, data.row_count);
        eventSource.close();
    });

    eventSource.addEventListener('QueryFailed', (e) => {
        const data = JSON.parse(e.data);
        displayError(data.error.message);
        eventSource.close();
    });
}
```

This pattern ensures the UI remains responsive during long-running queries while providing real-time feedback through the CQRS event pipeline.

## Metrics and observability

Instrument background tasks with metrics for monitoring and debugging.

```rust
use prometheus::{register_histogram, register_counter, Histogram, Counter};

lazy_static! {
    static ref QUERY_DURATION: Histogram = register_histogram!(
        "query_duration_seconds",
        "Time spent executing queries"
    ).unwrap();

    static ref QUERY_FAILURES: Counter = register_counter!(
        "query_failures_total",
        "Total number of failed queries"
    ).unwrap();

    static ref ACTIVE_QUERIES: prometheus::IntGauge = register_int_gauge!(
        "active_queries",
        "Number of currently executing queries"
    ).unwrap();
}

async fn execute_query_background_instrumented(
    query_id: Uuid,
    sql: String,
    event_store: EventStore,
    event_bus: ZenohEventBus,
    duckdb_pool: DuckDbPool,
) {
    let _timer = QUERY_DURATION.start_timer();
    ACTIVE_QUERIES.inc();

    let result = execute_duckdb_query(&duckdb_pool, &sql).await;

    ACTIVE_QUERIES.dec();

    match result {
        Ok(rows) => {
            // Emit completion event
        }
        Err(err) => {
            QUERY_FAILURES.inc();
            // Emit failure event
        }
    }
}
```

Monitor these metrics to detect slow queries, resource exhaustion, and failure patterns.
