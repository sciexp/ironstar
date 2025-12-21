# CQRS implementation architecture decisions

This document records CQRS and event sourcing implementation decisions for the ironstar stack, covering aggregate patterns, event schema evolution, and framework selection rationale.
For frontend, backend core, and infrastructure decisions, see the related documentation section below.

## Pure aggregate pattern — domain logic as state machines

Ironstar adopts the pure synchronous aggregate pattern from esrs (Prima.it's event_sourcing.rs).
Aggregates are side-effect-free state machines:

```rust
pub trait Aggregate: Default + Send + Sync {
    const TYPE: &'static str;
    type State: Clone + Send + Sync;
    type Command;
    type Event: DomainEvent;
    type Error: std::error::Error;

    // Pure function: State × Command → Result<Vec<Event>, Error>
    fn handle_command(state: &Self::State, cmd: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;

    // Pure function: State × Event → State
    fn apply_event(state: Self::State, event: Self::Event) -> Self::State;
}
```

**Why synchronous aggregates:**

- All I/O isolated at application layer boundaries (command handlers, event store)
- Aggregates become pure functions, trivially testable
- No hidden async dependencies inside domain logic
- Aligns with the "effects explicit in types" design principle

External service calls (validation against external APIs, lookups) happen in the command handler *before* calling `handle_command`, not inside the aggregate.

**Test framework DSL (inspired by cqrs-es):**

The pure aggregate pattern enables elegant given/when/then testing:

```rust
TestFramework::with(service)
    .given(vec![PreviousEvent::Created { ... }])
    .when(Command::Update { ... })
    .then_expect_events(vec![Event::Updated { ... }]);
```

This DSL is possible because `handle_command` is a pure function — no mocking of async dependencies required.

**Framework vs. custom implementation decision:**

Ironstar evaluated three Rust CQRS/ES crates:

| Crate | Strengths | Limitations |
|-------|-----------|-------------|
| **cqrs-es** | Mature, elegant test DSL | Abstractions may conflict with hypertext + datastar integration |
| **esrs** | Pure aggregates, upcaster pattern | PostgreSQL-only |
| **sqlite-es** | SQLite adapter for cqrs-es | Thin wrapper, patterns valuable but library unnecessary |

Decision: implement custom CQRS layer adopting patterns from these libraries, not the frameworks themselves.
This preserves flexibility for tight integration with hypertext templates and datastar SSE generation.

**Patterns adopted:**

From **esrs**:
- Pure synchronous aggregates: `handle_command(state, cmd) -> Result<Vec<Event>, Error>` with no async/side effects
- Schema/Upcaster pattern for event evolution
- Clear separation between domain logic and infrastructure

From **cqrs-es**:
- TestFramework DSL for given/when/then testing
- GenericQuery pattern for projections
- Event store trait abstraction

From **sqlite-es**:
- Event store schema with compound primary key
- JSON payload for event data
- Optimistic locking via sequence numbers

**Rationale for custom implementation:**

While these frameworks provide excellent patterns, direct dependency on them introduces constraints:

1. **Integration complexity**: cqrs-es's abstractions (GenericQuery, EventEnvelope) require adapters to work with hypertext templates and datastar SSE
2. **Backend limitations**: esrs only supports PostgreSQL; porting to SQLite would require rewriting the storage layer
3. **Unnecessary indirection**: sqlite-es is a thin adapter over cqrs-es; the value is in understanding the patterns, not the library code

The custom approach allows:
- Direct conversion from domain events to SSE patches via hypertext templates
- SQLite event store optimized for SSE Last-Event-ID support (global sequence numbers)
- Lighter dependency footprint without sacrificing pattern quality

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
HTTP POST → Command Handler → Aggregate.handle_command → Events
    ↓
Append to SQLite event store
    ↓
Publish to tokio::broadcast
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
| **Pure domain logic** | Aggregates are sync pure functions | Testable without mocking |
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

## Related documentation

- Design principles: `design-principles.md`
- Frontend stack decisions: `frontend-stack-decisions.md`
- Backend core decisions: `backend-core-decisions.md`
- Infrastructure decisions: `infrastructure-decisions.md`
- Build tooling decisions: `build-tooling-decisions.md`
- Event sourcing patterns: `event-sourcing-sse-pipeline.md`
- Crate architecture: `crate-architecture.md`
- Module organization: `architecture-decisions.md` (see Module organization section)
