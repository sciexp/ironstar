---
title: Event store transactions
---

# Event store transactions

Event sourcing requires transactional guarantees when appending events to the store.
Without proper transaction isolation and concurrency control, concurrent command handlers operating on the same aggregate could produce conflicting event sequences, violating aggregate invariants.

Consider two users simultaneously marking the same todo item as completed.
Each handler reads the current state (incomplete), validates the command (valid), and emits a TodoCompleted event.
If both events append successfully with the same sequence number, the aggregate history becomes corrupted — replaying events would see two completion events for a single todo.

Ironstar uses optimistic locking with version checking to detect and prevent such conflicts while maintaining high concurrency for independent aggregates.

## Optimistic locking

Optimistic locking assumes conflicts are rare and checks for concurrent modifications only at commit time.
Each aggregate maintains a monotonically increasing sequence number.
When a command handler emits events, it specifies the expected current version of the aggregate.
The event store rejects the append if another transaction has already incremented the sequence number.

**Version checking semantics:**

```rust
fn append_events(
    aggregate_type: &str,
    aggregate_id: &str,
    expected_version: u64,
    events: Vec<Event>,
) -> Result<(), AppendError> {
    // Verify current max sequence matches expected_version
    // If mismatch → return Err(AppendError::VersionConflict)
    // Otherwise → append events with sequence = expected_version + 1, +2, ...
}
```

The expected_version represents the aggregate's sequence number before applying the new events.
After a successful append, the aggregate's version becomes `expected_version + events.len()`.

**Example:**

```rust
// Aggregate at version 3 (has events with sequence 1, 2, 3)
// Handler emits 2 new events
append_events("Todo", "123", expected_version: 3, events: [e4, e5])
// → Success: events inserted with sequence 4, 5
// → Aggregate now at version 5

// Concurrent handler with stale version
append_events("Todo", "123", expected_version: 3, events: [e6])
// → Err(VersionConflict): version is now 5, not 3
```

## Batch append API

The event store provides a batch append operation that treats multiple events as an atomic unit.
Either all events in the batch are persisted with consecutive sequence numbers, or none are.

**API signature:**

```rust
pub async fn append_events(
    &self,
    aggregate_type: &str,
    aggregate_id: &str,
    expected_version: u64,
    events: Vec<Event>,
) -> Result<Vec<u64>, AppendError>;
```

**Return value:**

On success, returns the global sequence numbers assigned to each event (used for SSE Last-Event-ID).
On failure, returns an error indicating the conflict or database failure.

**Error cases:**

```rust
pub enum AppendError {
    VersionConflict { expected: u64, actual: u64 },
    DatabaseError(sqlx::Error),
    EmptyEventList,
}
```

**Invariants:**

1. Events in a batch receive consecutive aggregate sequence numbers
2. Events in a batch receive consecutive global sequence numbers
3. If any event fails to insert, the entire batch is rolled back
4. Version checking happens before any events are inserted

## SQLite implementation

SQLite provides ACID transactions with serializable isolation by default.
The event store uses explicit transactions to ensure atomic batch appends with version checking.

**Schema:**

```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,  -- Global sequence
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    sequence INTEGER NOT NULL,             -- Per-aggregate sequence
    event_type TEXT NOT NULL,
    payload JSON NOT NULL,
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(aggregate_type, aggregate_id, sequence)
);

CREATE INDEX idx_aggregate ON events(aggregate_type, aggregate_id, sequence);
CREATE INDEX idx_global_sequence ON events(id);
```

The UNIQUE constraint on `(aggregate_type, aggregate_id, sequence)` provides the mechanical enforcement of optimistic locking.
Attempting to insert a duplicate sequence number returns a constraint violation error.

**Transaction structure:**

```rust
async fn append_events_impl(
    tx: &mut Transaction<'_, Sqlite>,
    aggregate_type: &str,
    aggregate_id: &str,
    expected_version: u64,
    events: Vec<Event>,
) -> Result<Vec<u64>, AppendError> {
    // 1. Query current version
    let current_version: Option<u64> = sqlx::query_scalar(
        "SELECT MAX(sequence) FROM events
         WHERE aggregate_type = ? AND aggregate_id = ?"
    )
    .bind(aggregate_type)
    .bind(aggregate_id)
    .fetch_optional(&mut **tx)
    .await?;

    let current_version = current_version.unwrap_or(0);

    // 2. Check version
    if current_version != expected_version {
        return Err(AppendError::VersionConflict {
            expected: expected_version,
            actual: current_version,
        });
    }

    // 3. Insert events with consecutive sequence numbers
    let mut global_ids = Vec::new();
    for (i, event) in events.iter().enumerate() {
        let sequence = expected_version + 1 + (i as u64);
        let result = sqlx::query(
            "INSERT INTO events
             (aggregate_type, aggregate_id, sequence, event_type, payload, metadata)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(sequence)
        .bind(&event.event_type)
        .bind(&event.payload)
        .bind(&event.metadata)
        .execute(&mut **tx)
        .await?;

        global_ids.push(result.last_insert_rowid() as u64);
    }

    Ok(global_ids)
}
```

**Transaction lifecycle:**

```rust
pub async fn append_events(
    &self,
    aggregate_type: &str,
    aggregate_id: &str,
    expected_version: u64,
    events: Vec<Event>,
) -> Result<Vec<u64>, AppendError> {
    let mut tx = self.pool.begin().await?;
    let global_ids = append_events_impl(
        &mut tx,
        aggregate_type,
        aggregate_id,
        expected_version,
        events
    ).await?;
    tx.commit().await?;
    Ok(global_ids)
}
```

SQLite's default isolation level (serializable) ensures that concurrent transactions cannot observe intermediate states.
If two transactions check the version simultaneously, only one will successfully commit — the other will fail with a UNIQUE constraint violation.

## Handling conflicts

When a version conflict occurs, the command handler must decide whether to retry or fail.

**Retry strategy:**

Retrying is appropriate when the command is idempotent or when the handler can safely re-validate against the updated aggregate state.

```rust
async fn handle_command_with_retry(
    cmd: Command,
    max_retries: u32,
) -> Result<(), CommandError> {
    for attempt in 0..max_retries {
        // Load current aggregate state
        let events = event_store.load_events(&cmd.aggregate_id).await?;
        let state = Aggregate::replay(events)?;

        // Validate command against current state
        let new_events = state.handle_command(cmd.clone())?;

        // Attempt append with current version
        match event_store.append_events(
            "Todo",
            &cmd.aggregate_id,
            state.version,
            new_events,
        ).await {
            Ok(_) => return Ok(()),
            Err(AppendError::VersionConflict { .. }) if attempt < max_retries - 1 => {
                // Retry with updated state
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Err(CommandError::MaxRetriesExceeded)
}
```

**Fail-fast strategy:**

Some commands should not be retried automatically, particularly when user intent matters.
For example, if a user clicks "Mark Complete" but another user has already deleted the todo, retrying could produce confusing behavior.

```rust
async fn handle_command_fail_fast(cmd: Command) -> Result<(), CommandError> {
    let events = event_store.load_events(&cmd.aggregate_id).await?;
    let state = Aggregate::replay(events)?;
    let new_events = state.handle_command(cmd)?;

    event_store.append_events(
        "Todo",
        &cmd.aggregate_id,
        state.version,
        new_events,
    ).await?;

    Ok(())
}
```

On conflict, the handler returns an error to the user indicating the aggregate has been modified concurrently, allowing them to refresh and retry manually.

**Trade-offs:**

- Automatic retry: Higher success rate, risk of stale validation logic
- Fail-fast: Clear error signals, requires user intervention

Ironstar defaults to fail-fast for user-initiated commands and automatic retry (with bounded attempts) for system-initiated background processes.

## Hybrid sequence schema

The event store maintains two sequence numbers per event:

1. **Per-aggregate sequence** (`sequence` column): Monotonic per aggregate, used for optimistic locking
2. **Global sequence** (`id` column): Monotonic across all aggregates, used for SSE Last-Event-ID

**Rationale:**

Per-aggregate sequence is essential for version checking — each aggregate's sequence space is independent.
Global sequence is essential for SSE replay — clients must be able to resume from an arbitrary point in the global event stream without knowing which aggregates changed.

**Example event table state:**

```
id  | aggregate_type | aggregate_id | sequence | event_type
----|----------------|--------------|----------|------------------
1   | Todo           | abc          | 1        | TodoCreated
2   | Todo           | abc          | 2        | TodoCompleted
3   | User           | alice        | 1        | UserRegistered
4   | Todo           | xyz          | 1        | TodoCreated
5   | Todo           | abc          | 3        | TodoDeleted
```

Aggregate `Todo:abc` has version 3 (sequence 1, 2, 3).
Aggregate `Todo:xyz` has version 1 (sequence 1).
Aggregate `User:alice` has version 1 (sequence 1).

SSE clients track global sequence via Last-Event-ID.
A client with Last-Event-ID=2 would receive events with id >= 3.

**Implementation detail:**

SQLite's `AUTOINCREMENT` on the `id` column guarantees monotonically increasing global sequence numbers even across transactions.
The per-aggregate sequence is manually computed by the append implementation based on `MAX(sequence) + 1`.

This hybrid approach enables both local correctness (optimistic locking per aggregate) and global ordering (SSE replay across aggregates).
