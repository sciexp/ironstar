# CQRS implementation architecture decisions

This document records CQRS and event sourcing implementation decisions for the ironstar stack, covering aggregate patterns, event schema evolution, and framework selection rationale.
For frontend, backend core, and infrastructure decisions, see the related documentation section below.

## fmodel-rust adoption — the Decider pattern for functional event sourcing

Ironstar adopts fmodel-rust's Decider pattern, the minimal algebraic interface for functional event sourcing.
Deciders are pure functions (`decide`, `evolve`, `initial_state`) with no async or side effects, directly implementing ironstar's "effects explicit in type signatures" design principle.

```rust
pub struct Decider<'a, C, S, E, Error = ()> {
    pub decide: Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + 'a + Send + Sync>,
    pub evolve: Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>,
    pub initial_state: Box<dyn Fn() -> S + 'a + Send + Sync>,
}
```

The Decider pattern enforces purity by construction: function signatures prohibit async, mutable state, or hidden I/O.
This is the canonical implementation of Ghosh's module algebra (Signature → Algebra → Interpreter) with category-theoretic grounding: Decider is an algebra (folding events) paired with a coalgebra (unfolding commands).

For complete adoption rationale, see `fmodel-rust-adoption-evaluation.md`.

**Why synchronous Deciders:**

- All I/O isolated at application layer boundaries (command handlers, event store)
- Deciders become pure functions, trivially testable
- No hidden async dependencies inside domain logic
- Aligns with the "effects explicit in types" design principle
- fmodel-rust enforces this via type signatures, preventing accidental violations

External service calls (validation against external APIs, lookups) happen in the command handler *before* calling `Decider::decide`, not inside the Decider.
This is the rationale for choosing fmodel-rust over cqrs-es, which allows mutable `apply(&mut self)` patterns that violate purity.

**Smart constructors for domain validation:**

Validation for domain invariants uses smart constructors that make invalid states unrepresentable in the type system:

```rust
/// Validated Todo text - non-empty, max 200 characters
pub struct TodoText(String);

impl TodoText {
    pub fn new(s: &str) -> Result<Self, ValidationError> {
        if s.is_empty() {
            return Err(ValidationError::Empty);
        }
        if s.len() > 200 {
            return Err(ValidationError::TooLong { max: 200, actual: s.len() });
        }
        Ok(Self(s.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

This pattern shifts validation to construction time.
Once a `TodoText` value exists, the type system guarantees it satisfies the invariants.
Command handlers validate input by constructing these types before passing data to Deciders, keeping Decider logic focused on business rules rather than format validation.

**Test framework DSL (from fmodel-rust):**

The pure Decider pattern enables elegant given/when/then testing via fmodel-rust's `DeciderTestSpecification`:

```rust
DeciderTestSpecification::default()
    .for_decider(todo_decider())
    .given(vec![])  // No prior events
    .when(TodoCommand::Create { id, text })
    .then_expect_events(vec![TodoEvent::Created { id, text, created_at }]);
```

This DSL is possible because `Decider::decide` and `Decider::evolve` are pure functions — no mocking of async dependencies required.
See evaluation document appendix for additional test examples including failure event assertions.

**Framework adoption decision: fmodel-rust**

After evaluating multiple Rust CQRS/ES crates (see `fmodel-rust-adoption-evaluation.md` for complete analysis), ironstar adopts fmodel-rust as its event sourcing foundation.

| Crate | Status | Relationship to ironstar |
|-------|--------|--------------------------|
| **fmodel-rust** | ADOPTED | Primary event sourcing abstraction |
| **cqrs-es** | Studied | Patterns now provided by fmodel-rust (testing DSL, event store abstraction) |
| **esrs** | Studied | Patterns now provided by fmodel-rust (pure sync aggregates) |
| **sqlite-es** | Reference | Event store schema patterns inform SQLite EventRepository implementation |

**Why fmodel-rust was chosen:**

1. **Pure function enforcement**: Type signatures guarantee `decide` and `evolve` are side-effect-free (no async, no mutable state)
2. **Composition primitives**: Built-in `combine()` and `merge()` enable multi-aggregate workflows without custom event bus wiring
3. **Testing DSL**: `DeciderTestSpecification` provides given/when/then testing ironstar's docs specify
4. **SQLite compatibility**: EventRepository trait is database-agnostic; SQLite implementation requires only trivial type mappings (see evaluation doc)
5. **Framework-agnostic**: Zero web dependencies; Axum integration is handler composition, not library lock-in
6. **Category-theoretic grounding**: Decider is the minimal realization of Ghosh's module algebra, satisfying ironstar's theoretical foundations requirements

**What ironstar implements:**

- Custom SQLite EventRepository (fmodel-rust-postgres schema adapted for SQLite)
- Event schema evolution via custom Upcaster pattern (orthogonal to Decider)
- Zenoh event bus for SSE broadcast (fmodel-rust's EventRepository handles persistence, not notification)

**Patterns from studied libraries:**

The evaluation of cqrs-es, esrs, and sqlite-es informed ironstar's architecture, but these patterns are now provided by fmodel-rust:
- Pure synchronous domain logic (esrs) → `Decider::decide` and `Decider::evolve`
- TestFramework DSL (cqrs-es) → `DeciderTestSpecification`
- Event store abstraction (cqrs-es) → `EventRepository` trait
- SQLite schema patterns (sqlite-es) → Adapted for fmodel-rust's EventRepository

---

## Event schema evolution — upcaster pattern

For long-lived systems, event schemas evolve.
Ironstar adopts the Upcaster pattern from esrs:

```rust
pub trait EventUpcaster: Send + Sync {
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool;
    fn upcast(&self, payload: serde_json::Value) -> serde_json::Value;
}
```

Each event stores an `event_version` field.
On deserialization, if the stored version differs from current, upcasters transform the JSON payload.

**Example migration (adding a field):**

```rust
impl EventUpcaster for OrderEventV2Upcaster {
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool {
        event_type == "OrderPlaced" && event_version == "1.0.0"
    }

    fn upcast(&self, mut payload: Value) -> Value {
        // V1 → V2: Add default customer_id field
        if let Value::Object(ref mut obj) = payload {
            obj.entry("customer_id").or_insert(Value::Null);
        }
        payload
    }
}
```

Upcasters are applied in sequence during event loading, enabling incremental schema evolution without data migrations.

**Algebraic interpretation:**

Upcasters form a *category* where:

- Objects are event schema versions
- Morphisms are upcaster functions between versions
- Composition is sequential upcaster application
- Identity is the no-op upcaster for current version

This structure guarantees that any historical event can be loaded into the current domain model, preserving the append-only property of the event store.

**Event versioning strategy:**

Events include a `version` field in metadata:

```rust
#[derive(Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_type: String,
    pub event_version: String,  // Semantic version: "1.0.0"
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub sequence: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: i64,
    pub metadata: EventMetadata,
    pub payload: serde_json::Value,
}
```

When loading events, the event store checks `event_version` against the current schema version.
If they differ, it applies the upcaster chain to bring the payload up to date.

**Migration workflow:**

1. Define new event variant with `#[serde(rename = "EventName")]` to preserve type identifier
2. Increment version in event metadata: `"1.0.0"` → `"2.0.0"`
3. Write upcaster from old version to new
4. Register upcaster with event store
5. Old events are automatically migrated on load

**Why this approach:**

- No database schema migrations required (events are JSON)
- Upcasters are testable pure functions
- Can replay historical events through current domain logic
- Supports gradual schema evolution without downtime

---

## CQRS architecture overview

Ironstar's CQRS implementation separates write operations (commands) from read operations (queries) at the handler level, with events as the single source of truth.

**Write side (command processing):**

```
HTTP POST → Command Handler → Decider.decide → Events
    ↓
Append to SQLite event store via EventRepository
    ↓
Publish to Zenoh for SSE broadcast
    ↓
Return HTTP 202 Accepted
```

**Read side (query processing and projections):**

```
Events from broadcast → Event Handlers → Update Read Models
    ↓
HTTP GET/SSE → Query Handler → Read from Read Model → Return data
```

**Key architectural properties:**

| Property | Implementation | Benefit |
|----------|----------------|---------|
| **Separation of concerns** | Commands write events; queries read projections | Optimized data models for each use case |
| **Event-driven updates** | Projections subscribe to event bus | Eventually consistent read models |
| **Pure domain logic** | Deciders are sync pure functions | Testable without mocking |
| **SSE integration** | Events convert directly to PatchElements | Real-time UI updates |
| **Audit trail** | SQLite append-only event log | Complete history for debugging/replay |

**Read model patterns:**

Read models are optimized for specific query patterns:

```rust
pub struct TodoReadModel {
    todos: HashMap<Uuid, Todo>,
    by_status: BTreeMap<Status, Vec<Uuid>>,
    by_user: BTreeMap<UserId, Vec<Uuid>>,
}

impl TodoReadModel {
    // Efficient lookups for common queries
    pub fn list_by_status(&self, status: Status) -> Vec<&Todo> {
        self.by_status.get(&status)
            .map(|ids| ids.iter().filter_map(|id| self.todos.get(id)).collect())
            .unwrap_or_default()
    }

    // Apply events to maintain consistency
    pub fn apply(&mut self, event: TodoEvent) {
        match event {
            TodoEvent::Created { id, text, user_id } => {
                self.todos.insert(id, Todo { id, text, status: Status::Active, user_id });
                self.by_status.entry(Status::Active).or_default().push(id);
                self.by_user.entry(user_id).or_default().push(id);
            }
            TodoEvent::StatusChanged { id, new_status } => {
                if let Some(todo) = self.todos.get_mut(&id) {
                    // Remove from old status index
                    if let Some(ids) = self.by_status.get_mut(&todo.status) {
                        ids.retain(|&x| x != id);
                    }
                    // Update todo and add to new status index
                    todo.status = new_status;
                    self.by_status.entry(new_status).or_default().push(id);
                }
            }
            // ... other events
        }
    }
}
```

**Projection initialization:**

On application startup, read models are rebuilt by replaying all events from the event store:

```rust
pub async fn init_projections(event_store: &SqliteEventStore) -> Result<Projections> {
    let mut todo_model = TodoReadModel::default();

    // Replay all events to rebuild projection
    let events = event_store.load_all().await?;
    for event in events {
        if let Ok(todo_event) = serde_json::from_value::<TodoEvent>(event.payload) {
            todo_model.apply(todo_event);
        }
    }

    Ok(Projections { todo: Arc::new(RwLock::new(todo_model)) })
}
```

**Consistency guarantees:**

- **Write side**: Strong consistency via SQLite transactions (ACID)
- **Read side**: Eventually consistent via event broadcast
- **Latency**: Typically <10ms from event commit to projection update in single-node deployment

For guaranteed consistency on read-after-write, handlers can explicitly wait for projection update or read directly from event store (slower but consistent).

---

## Deferred patterns: Process managers and sagas

Ironstar v1 targets single-aggregate analytics workflows.
Multi-aggregate coordination patterns are deferred:

### What's deferred

- **Process managers** (Hoffman Laws 8-9): Stateful coordinators that consume events and emit commands across aggregate boundaries
- **Sagas**: Long-running transactions with compensation logic
- **Choreography patterns**: Event-driven coordination without central orchestrator

fmodel-rust provides the `Saga` abstraction for future implementation when multi-aggregate workflows are needed.
The `Saga` pattern enables event-to-command choreography (reacting to events by emitting commands to other aggregates) without custom event bus wiring.

### Why deferred

1. **Scope focus**: v1 demonstrates CQRS/ES patterns with QuerySession and Todo aggregates independently
2. **Complexity budget**: Process managers add significant operational complexity
3. **Single-node deployment**: Distribution patterns aren't needed for embedded architecture

### When to introduce

Consider process managers when:
- Workflows span multiple aggregates (Order → Payment → Shipping)
- Compensation logic is required (rollback on partial failure)
- Long-running processes need durable state (days/weeks)

### Reference

See `~/.claude/commands/preferences/event-sourcing.md` for:
- Law 8: Never manage more than one flow per process manager
- Law 9: Process managers consume events and emit commands
- Saga implementation patterns with compensation

See `fmodel-rust-adoption-evaluation.md` appendix (Phase 6) for example Saga implementation with fmodel-rust.

---

## References

The fmodel-rust adoption decision was informed by both theoretical principles and practical pattern study.

**Primary sources:**

| Source | Contribution |
|--------|--------------|
| Kevin Hoffman, *Real World Event Sourcing* | Laws 7, 2, 10 directly inform pure Deciders, schema immutability, and testing patterns |
| Scott Wlaschin, *Domain Modeling Made Functional* | Aggregates as consistency boundaries, smart constructor pattern |
| Debasish Ghosh, *Functional and Reactive Domain Modeling* | Module algebra (Signature → Algebra → Interpreter), theoretical grounding for Decider pattern |
| `~/.claude/commands/preferences/event-sourcing.md` | Theoretical synthesis and decision frameworks |

Hoffman's **Law 7** (work is a side effect) is the central principle: Deciders contain no I/O, enabling pure functional domain logic.
**Law 2** (event schemas are immutable) drives the upcaster pattern for schema evolution.
**Law 10** (aggregates own event streams) enables the DeciderTestSpecification DSL pattern where tests assert events, not Decider internals.

**Rust event sourcing libraries:**

| Library | Status | Maturity | Notes |
|---------|--------|----------|-------|
| **fmodel-rust** | ADOPTED | Production | Primary event sourcing abstraction; Decider pattern enforces purity |
| **cqrs-es** | Studied | Production | TestFramework DSL pattern now provided by fmodel-rust's DeciderTestSpecification |
| **esrs** | Studied | Production | Pure sync aggregate pattern now provided by fmodel-rust's Decider |
| **sqlite-es** | Reference | Production | Schema patterns inform SQLite EventRepository implementation |
| **kameo_es** | Studied | Alpha | Actor patterns orthogonal to ironstar; causation tracking worth noting |
| **SierraDB** | Reference | Pre-production | Future reference for multi-node scaling; not suitable for current embedded approach |

For complete adoption rationale including SQLite compatibility analysis and migration path, see `fmodel-rust-adoption-evaluation.md`.

---

## Related documentation

- fmodel-rust adoption evaluation: `fmodel-rust-adoption-evaluation.md` (complete rationale, SQLite compatibility, migration path)
- Design principles: `../core/design-principles.md`
- Frontend stack decisions: `frontend-stack-decisions.md`
- Backend core decisions: `backend-core-decisions.md`
- Infrastructure decisions: `infrastructure-decisions.md`
- Build tooling decisions: `build-tooling-decisions.md`
- Event sourcing core concepts: `../cqrs/event-sourcing-core.md`
- Command write patterns: `../cqrs/command-write-patterns.md`
- Crate architecture: `../core/crate-architecture.md`
- Module organization: `../core/architecture-decisions.md` (see Module organization section)
