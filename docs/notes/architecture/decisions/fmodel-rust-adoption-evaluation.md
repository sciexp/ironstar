# Evaluation: fmodel-rust adoption for ironstar event sourcing

## Executive summary

**Recommendation: YES, adopt fmodel-rust as ironstar's event sourcing foundation.**

The Decider pattern implemented by fmodel-rust is the minimal algebraic interface for functional event sourcing, directly realizing the theoretical foundations in ironstar's preference documents.
fmodel-rust addresses ironstar's explicit rejection of cqrs-es (mutable `apply(&mut self)` patterns) with pure `evolve(state, event) → state` semantics.

The adoption path is straightforward:
- SQLite is fully compatible with fmodel-rust's EventRepository trait (no PostgreSQL requirement)
- The library is framework-agnostic (zero web dependencies); Axum integration is trivial
- Current ironstar aggregate patterns map 1:1 to Decider functions
- Composition via `combine()` enables future multi-aggregate workflows without event bus complexity

Key factors driving this recommendation:
1. fmodel-rust's pure function enforcement matches ironstar's "effects explicit in type signatures" philosophy
2. The Decider pattern is the canonical implementation of Ghosh's module algebra (Signature → Algebra → Interpreter)
3. SQLite portability is confirmed: schema requires only trivial type mappings, no PostgreSQL-specific features
4. Built-in composition (`combine`, `merge`) provides the multi-aggregate coordination ironstar will need
5. TestSpecification DSL provides given/when/then testing ironstar's architecture documents already specify

---

## Architectural alignment analysis

### Current ironstar patterns → fmodel-rust mapping

| ironstar Current | fmodel-rust Equivalent | Compatibility |
|------------------|------------------------|---------------|
| `Aggregate::handle_command(state, cmd) → Result<Vec<Event>, Error>` | `Decider::decide` | Direct 1:1 |
| `Aggregate::apply_event(state, event) → State` | `Decider::evolve` | Direct 1:1 |
| `Aggregate::State::default()` | `Decider::initial_state` | Direct 1:1 |
| `AggregateRoot<A>` version tracking | `EventRepository::version_provider` | Direct 1:1 |
| Custom projection builders | `View` + `MaterializedView` | Direct 1:1 |
| (Not implemented) Process managers | `Saga` + `SagaManager` | Available |

### Design principle compatibility assessment

**1. Pure function separation (Tao of Datastar principle 1)**

ironstar requires: "Aggregates are pure functions with all I/O at boundaries"

fmodel-rust enforces:
```rust
pub decide: Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + Send + Sync>
pub evolve: Box<dyn Fn(&S, &E) -> S + Send + Sync>
```

Both functions are pure by construction: no async, no mutable state, referentially transparent.
This directly implements Hoffman's Law 7: "Work is a side effect."

**2. Algebraic composition (theoretical foundations)**

ironstar requires: "Deciders form a commutative monoid under composition"

fmodel-rust provides:
```rust
pub fn combine<C2, S2, E2>(self, decider2: Decider<C2, S2, E2>)
    -> Decider<Sum<C, C2>, (S, S2), Sum<E, E2>>
```

Composition properties satisfied:
- Closure: composing two Deciders yields another Decider
- Associativity: `(d1 ⊕ d2) ⊕ d3 = d1 ⊕ (d2 ⊕ d3)`
- Identity: trivial Decider that accepts any state, returns no events
- Commutativity: order of combination doesn't affect behavior

This realizes the category-theoretic foundations: Decider is an algebra (folding events) paired with a coalgebra (unfolding commands).

**3. Event schema immutability (Hoffman's Law 2)**

ironstar requires: "Event schemas are immutable; evolution uses versioned types with upcasters"

fmodel-rust is compatible: The library doesn't mandate schema handling, allowing ironstar's existing `EventUpcaster` pattern to work unchanged.
Events store in JSONwith `event_version` metadata; upcasting happens at deserialization time.

**4. Effect boundaries (axum integration)**

ironstar requires: "The async/sync boundary IS the effect boundary"

fmodel-rust enforces this perfectly:
- Domain layer: Pure `Decider` (no async, no I/O)
- Application layer: `EventSourcedAggregate` handles async I/O
- Infrastructure: `EventRepository` implementer handles SQL

The `EventSourcedAggregate::handle()` method:
```rust
pub async fn handle(&self, command: &C) -> Result<Vec<(E, Version)>, Error> {
    let events = self.fetch_events(command).await?;  // I/O
    let state = events.fold(initial_state(), evolve);  // Pure
    let new_events = (decider.decide)(command, &state)?;  // Pure
    self.save(&new_events).await?;  // I/O
    Ok(new_events)
}
```

### What ironstar gains

1. **Composition primitives**: `combine()` and `merge()` enable multi-aggregate workflows without custom event bus wiring
2. **Saga/process managers**: Built-in `Saga<Event, Command>` pattern for event-driven choreography
3. **Testing DSL**: `DeciderTestSpecification` provides given/when/then testing ironstar's docs already specify
4. **Send/!Send flexibility**: Feature flag chooses between multi-threaded and single-threaded async

### What ironstar retains

- Full control over event store schema (custom SQLite implementation)
- Upcaster pattern for event evolution
- Hypertext + datastar integration (fmodel-rust has no web opinions)
- Zenoh event bus for broadcast (fmodel-rust's EventRepository is persistence, not notification)

---

## Database stack recommendation

**Recommendation: SQLite with custom EventRepository implementation**

### Feasibility assessment

The fmodel-rust-postgres schema was analyzed for SQLite portability:

| PostgreSQL Feature | SQLite Equivalent | Complexity |
|--------------------|-------------------|------------|
| `JSONB` | `TEXT CHECK(json_valid(data))` | Trivial |
| `BIGSERIAL` | `INTEGER PRIMARY KEY AUTOINCREMENT` | Trivial |
| `UUID` | `TEXT CHECK(length(value)=36)` | Trivial |
| `TIMESTAMP WITH TIME ZONE` | `TEXT DEFAULT(datetime('now', 'utc'))` | Trivial |
| PL/pgSQL triggers | SQLite trigger syntax | Minor |
| `UNIQUE(previous_id)` NULL handling | Identical behavior | None |

**No PostgreSQL-specific features are required.**

### Recommended SQLite schema

```sql
CREATE TABLE deciders (
    decider TEXT NOT NULL,
    event TEXT NOT NULL,
    PRIMARY KEY (decider, event)
) STRICT;

CREATE TABLE events (
    event TEXT NOT NULL,
    event_id TEXT NOT NULL UNIQUE CHECK(length(event_id) = 36),
    decider TEXT NOT NULL,
    decider_id TEXT NOT NULL,
    data TEXT NOT NULL CHECK(json_valid(data)),
    command_id TEXT,
    previous_id TEXT UNIQUE REFERENCES events(event_id),
    final INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT(datetime('now', 'utc')),
    offset INTEGER PRIMARY KEY AUTOINCREMENT,
    FOREIGN KEY (decider, event) REFERENCES deciders (decider, event)
) STRICT;

CREATE INDEX idx_events_decider ON events (decider_id, offset);

-- Trigger: first event must have NULL previous_id
CREATE TRIGGER t_check_first_event BEFORE INSERT ON events
FOR EACH ROW
WHEN (NEW.previous_id IS NULL AND EXISTS(
    SELECT 1 FROM events WHERE decider_id = NEW.decider_id AND decider = NEW.decider
))
BEGIN
    SELECT RAISE(ABORT, 'previous_id can only be null for first decider event');
END;

-- Trigger: stream must not be finalized
CREATE TRIGGER t_check_final_event BEFORE INSERT ON events
FOR EACH ROW
WHEN EXISTS(
    SELECT 1 FROM events
    WHERE decider_id = NEW.decider_id AND decider = NEW.decider AND final = 1
)
BEGIN
    SELECT RAISE(ABORT, 'last event for this decider stream is already final');
END;

-- Trigger: previous_id must reference same stream
CREATE TRIGGER t_check_previous_id BEFORE INSERT ON events
FOR EACH ROW
WHEN (NEW.previous_id IS NOT NULL AND NOT EXISTS(
    SELECT 1 FROM events
    WHERE event_id = NEW.previous_id
    AND decider_id = NEW.decider_id
    AND decider = NEW.decider
))
BEGIN
    SELECT RAISE(ABORT, 'previous_id must reference event in same stream');
END;

-- Immutability enforcement
CREATE TRIGGER t_prevent_delete BEFORE DELETE ON events
BEGIN SELECT RAISE(ABORT, 'events are immutable'); END;

CREATE TRIGGER t_prevent_update BEFORE UPDATE ON events
BEGIN SELECT RAISE(ABORT, 'events are immutable'); END;
```

### Optimistic concurrency

The `previous_id` field implements optimistic locking:
- First event in stream: `previous_id = NULL`
- Subsequent events: `previous_id = event_id` of previous event
- `UNIQUE(previous_id)` constraint prevents concurrent appends to same stream
- Conflict detection: second writer violates unique constraint → retry with fresh state

### Global sequence for SSE Last-Event-ID

The `offset INTEGER PRIMARY KEY AUTOINCREMENT` provides monotonic ordering:
```sql
SELECT * FROM events WHERE offset > ? ORDER BY offset ASC
```

This supports SSE reconnection with Last-Event-ID header.

---

## Migration path

### Phase 1: Add fmodel-rust dependency (immediate)

```toml
# Cargo.toml workspace dependencies
fmodel-rust = { version = "0.9" }
```

No code changes required. Library is purely additive.

### Phase 2: Implement SQLite EventRepository

Create `crates/ironstar/src/infrastructure/event_repository.rs`:

```rust
use fmodel_rust::aggregate::EventRepository;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub struct SqliteEventRepository {
    pool: Pool<Sqlite>,
}

impl<C, E> EventRepository<C, E, Uuid, EventStoreError> for SqliteEventRepository
where
    C: Identifier + DeciderType + Sync,
    E: Identifier + EventType + DeciderType + IsFinal + Serialize + DeserializeOwned + Clone + Sync,
{
    async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Uuid)>, EventStoreError> {
        let rows = sqlx::query_as::<_, EventRow>(
            "SELECT event_id, data FROM events
             WHERE decider_id = ? AND decider = ?
             ORDER BY offset"
        )
        .bind(command.identifier())
        .bind(command.decider_type())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let event: E = serde_json::from_str(&row.data)?;
                Ok((event, row.event_id.parse()?))
            })
            .collect()
    }

    async fn save(&self, events: &[E]) -> Result<Vec<(E, Uuid)>, EventStoreError> {
        let mut tx = self.pool.begin().await?;
        let mut results = Vec::with_capacity(events.len());
        let mut previous_id: Option<Uuid> = None;

        for event in events {
            // Get latest version if this is first event
            if previous_id.is_none() {
                previous_id = self.version_provider(event).await?;
            }

            let event_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO events (event, event_id, decider, decider_id, data, previous_id, final)
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(event.event_type())
            .bind(event_id.to_string())
            .bind(event.decider_type())
            .bind(event.identifier())
            .bind(serde_json::to_string(event)?)
            .bind(previous_id.map(|id| id.to_string()))
            .bind(event.is_final())
            .execute(&mut *tx)
            .await?;

            results.push((event.clone(), event_id));
            previous_id = Some(event_id);
        }

        tx.commit().await?;
        Ok(results)
    }

    async fn version_provider(&self, event: &E) -> Result<Option<Uuid>, EventStoreError> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT event_id FROM events
             WHERE decider_id = ? AND decider = ?
             ORDER BY offset DESC LIMIT 1"
        )
        .bind(event.identifier())
        .bind(event.decider_type())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|id| id.parse().map_err(EventStoreError::from)).transpose()
    }
}
```

### Phase 3: Convert TodoAggregate to Decider

**Current implementation** (`crates/ironstar/src/domain/todo/aggregate.rs`):

```rust
impl Aggregate for TodoAggregate {
    fn handle_command(state: &TodoState, cmd: TodoCommand)
        -> Result<Vec<TodoEvent>, TodoError> {
        match cmd {
            TodoCommand::Create { id, text } => {
                if state.id.is_some() {
                    return Err(TodoError::AlreadyExists);
                }
                Ok(vec![TodoEvent::Created { id, text, created_at: Utc::now() }])
            }
            // ... other commands
        }
    }

    fn apply_event(mut state: TodoState, event: TodoEvent) -> TodoState {
        match event {
            TodoEvent::Created { id, text, created_at } => {
                state.id = Some(id);
                state.text = Some(text);
                // ...
            }
            // ...
        }
        state
    }
}
```

**fmodel-rust implementation**:

```rust
use fmodel_rust::decider::Decider;

pub fn todo_decider<'a>() -> Decider<'a, TodoCommand, Option<TodoState>, TodoEvent, TodoError> {
    Decider {
        decide: Box::new(|command, state| match command {
            TodoCommand::Create { id, text } => {
                if state.is_some() {
                    // Failure event, not error - preserves event log completeness
                    Ok(vec![TodoEvent::NotCreated {
                        id: id.clone(),
                        reason: "Todo already exists".into()
                    }])
                } else {
                    Ok(vec![TodoEvent::Created {
                        id: id.clone(),
                        text: text.clone(),
                        created_at: Utc::now()
                    }])
                }
            }
            TodoCommand::UpdateText { text } => {
                match state {
                    Some(s) if s.status != TodoStatus::Deleted => {
                        Ok(vec![TodoEvent::TextUpdated { text: text.clone() }])
                    }
                    _ => Ok(vec![TodoEvent::NotUpdated {
                        reason: "Todo does not exist or is deleted".into()
                    }])
                }
            }
            TodoCommand::Complete => {
                match state {
                    Some(s) if s.status == TodoStatus::Active => {
                        Ok(vec![TodoEvent::Completed { completed_at: Utc::now() }])
                    }
                    _ => Ok(vec![TodoEvent::NotCompleted {
                        reason: "Cannot complete: invalid state".into()
                    }])
                }
            }
            // ... other commands
        }),

        evolve: Box::new(|state, event| match event {
            TodoEvent::Created { id, text, created_at } => Some(TodoState {
                id: id.clone(),
                text: text.clone(),
                created_at: *created_at,
                status: TodoStatus::Active,
                completed_at: None,
            }),
            TodoEvent::NotCreated { .. } => state.clone(),  // Failure events preserve state
            TodoEvent::TextUpdated { text } => state.clone().map(|mut s| {
                s.text = text.clone();
                s
            }),
            TodoEvent::NotUpdated { .. } => state.clone(),
            TodoEvent::Completed { completed_at } => state.clone().map(|mut s| {
                s.status = TodoStatus::Completed;
                s.completed_at = Some(*completed_at);
                s
            }),
            TodoEvent::NotCompleted { .. } => state.clone(),
            TodoEvent::Deleted { .. } => state.clone().map(|mut s| {
                s.status = TodoStatus::Deleted;
                s
            }),
            TodoEvent::NotDeleted { .. } => state.clone(),
        }),

        initial_state: Box::new(|| None),
    }
}
```

### Phase 4: Wire to Axum handlers

```rust
use fmodel_rust::aggregate::EventSourcedAggregate;

pub async fn handle_todo_command(
    State(app_state): State<AppState>,
    Json(command): Json<TodoCommand>,
) -> impl IntoResponse {
    let aggregate = EventSourcedAggregate::new(
        app_state.event_repository.clone(),
        todo_decider(),
    );

    match aggregate.handle(&command).await {
        Ok(events) => {
            // Publish to Zenoh for SSE broadcast
            for (event, _version) in &events {
                app_state.zenoh.put(
                    format!("events/Todo/{}", event.identifier()),
                    serde_json::to_vec(event).unwrap(),
                ).await.ok();
            }
            (StatusCode::ACCEPTED, Json(events)).into_response()
        }
        Err(err) => {
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}
```

### Phase 5: Add testing with DeciderTestSpecification

```rust
#[cfg(test)]
mod tests {
    use fmodel_rust::decider::DeciderTestSpecification;
    use super::*;

    #[test]
    fn test_create_todo() {
        let id = TodoId::new();
        let text = TodoText::new("Buy groceries").unwrap();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![])  // No prior events
            .when(TodoCommand::Create { id: id.clone(), text: text.clone() })
            .then(vec![TodoEvent::Created {
                id: id.clone(),
                text: text.clone(),
                created_at: /* captured */
            }]);
    }

    #[test]
    fn test_create_existing_todo_emits_failure_event() {
        let id = TodoId::new();
        let text = TodoText::new("Buy groceries").unwrap();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![TodoEvent::Created { id: id.clone(), text: text.clone(), created_at: Utc::now() }])
            .when(TodoCommand::Create { id: id.clone(), text: text.clone() })
            .then(vec![TodoEvent::NotCreated {
                id: id.clone(),
                reason: "Todo already exists".into(),
            }]);
    }

    #[test]
    fn test_complete_active_todo() {
        let id = TodoId::new();
        let text = TodoText::new("Buy groceries").unwrap();

        DeciderTestSpecification::default()
            .for_decider(todo_decider())
            .given(vec![TodoEvent::Created { id: id.clone(), text, created_at: Utc::now() }])
            .when(TodoCommand::Complete)
            .then_state(Some(TodoState {
                id,
                text: /* ... */,
                status: TodoStatus::Completed,
                completed_at: Some(/* captured */),
                // ...
            }));
    }
}
```

### Phase 6: Deferred - Saga for multi-aggregate coordination

When ironstar needs Order → Payment → Shipping coordination:

```rust
use fmodel_rust::saga::Saga;

pub fn order_payment_saga<'a>() -> Saga<'a, OrderEvent, PaymentCommand> {
    Saga {
        react: Box::new(|event| match event {
            OrderEvent::Placed { order_id, amount, .. } => {
                vec![PaymentCommand::InitiatePayment {
                    order_id: order_id.clone(),
                    amount: amount.clone(),
                }]
            }
            OrderEvent::Cancelled { order_id, .. } => {
                vec![PaymentCommand::RefundPayment {
                    order_id: order_id.clone(),
                }]
            }
            _ => vec![],
        }),
    }
}
```

---

## Decision record

### Decision: Adopt fmodel-rust

**Status**: RECOMMENDED

**Context**: ironstar requires a functional event sourcing foundation that enforces pure aggregates, supports composition, and integrates with SQLite + Axum + datastar.

**Decision**: Adopt fmodel-rust as the primary ES abstraction, implementing a custom SQLite EventRepository.

**Key factors**:

1. **Algebraic purity enforced**: fmodel-rust's Decider pattern guarantees pure `decide`/`evolve` functions via type signatures (no async, no mutable state)

2. **Composition built-in**: `combine()` enables multi-aggregate workflows; `Saga` enables process managers - both patterns ironstar's architecture documents anticipate

3. **SQLite fully compatible**: The EventRepository trait is database-agnostic; PostgreSQL schema ports to SQLite with trivial type mappings

4. **Framework-agnostic**: Zero web dependencies; Axum integration is handler composition, not library lock-in

5. **Testing DSL provided**: `DeciderTestSpecification` implements given/when/then pattern ironstar's docs already specify

6. **Category-theoretic grounding**: Decider is the minimal realization of Ghosh's module algebra, satisfying ironstar's theoretical foundations requirements

**Alternatives considered**:

| Alternative | Why Not Chosen |
|-------------|----------------|
| cqrs-es | Mutable `apply(&mut self)` pattern violates purity requirements |
| esrs | PostgreSQL-only; no SQLite backend |
| Custom implementation (current) | Works but lacks composition primitives; reinvents DeciderTestSpecification |
| No ES abstraction | Would require manual fold/evolve across all aggregates |

**Consequences**:

- Current `Aggregate` trait implementations will migrate to `Decider` functions
- `AggregateRoot<A>` wrapper replaced by `EventSourcedAggregate<Decider, Repository>`
- Test code simplifies with `DeciderTestSpecification::given().when().then()`
- Future multi-aggregate coordination uses `Saga` instead of custom event bus wiring

**Risk assessment**:

| Risk | Mitigation |
|------|------------|
| Type safety loss (Box<dyn Fn> vs generics) | Rust's type system still enforces C, S, E consistency per Decider |
| Learning curve | Decider pattern maps 1:1 to existing Aggregate trait |
| Library abandonment | fmodel has active Kotlin, TypeScript, Java ports; Rust port is maintained by Fraktalio |
| SQLite concurrency limits | Optimistic locking via `previous_id` + WAL mode handles typical workloads |

---

## Appendix: fmodel-rust type signatures

```rust
// Core Decider (pure domain)
pub struct Decider<'a, C, S, E, Error = ()> {
    pub decide: Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + 'a + Send + Sync>,
    pub evolve: Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>,
    pub initial_state: Box<dyn Fn() -> S + 'a + Send + Sync>,
}

// Composition
impl Decider<C, S, E, Error> {
    pub fn combine<C2, S2, E2>(self, other: Decider<C2, S2, E2, Error>)
        -> Decider<Sum<C, C2>, (S, S2), Sum<E, E2>, Error>;
}

// EventRepository (infrastructure boundary)
pub trait EventRepository<C, E, Version, Error> {
    fn fetch_events(&self, command: &C) -> impl Future<Output = Result<Vec<(E, Version)>, Error>> + Send;
    fn save(&self, events: &[E]) -> impl Future<Output = Result<Vec<(E, Version)>, Error>> + Send;
    fn version_provider(&self, event: &E) -> impl Future<Output = Result<Option<Version>, Error>> + Send;
}

// Application layer (wires pure + effects)
pub struct EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error> { ... }

impl EventSourcedAggregate {
    pub async fn handle(&self, command: &C) -> Result<Vec<(E, Version)>, Error>;
}

// View (pure projection)
pub struct View<'a, S, E> {
    pub evolve: Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>,
    pub initial_state: Box<dyn Fn() -> S + 'a + Send + Sync>,
}

// Saga (pure choreography)
pub struct Saga<'a, AR, A> {
    pub react: Box<dyn Fn(&AR) -> Vec<A> + 'a + Send + Sync>,
}
```
