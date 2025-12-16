# Ironstar component selection: algebraic foundations

## Guiding principles

From our development guidelines, the ironstar stack must embody:

> *"Side effects should be explicit in type signatures and isolated at boundaries to preserve compositionality."*

> *"Integration of all codebases would correspond to an indexed monad transformer stack in the category of effects."*

This translates to concrete selection criteria:

| Principle | Concrete Requirement |
|-----------|---------------------|
| Algebraic data types | Sum types for states, product types for data |
| Explicit effects | `Result<T, E>`, `Option<T>`, `Future<Output=T>` in signatures |
| Compositionality | Pure functions, lawful abstractions, referential transparency |
| Boundary isolation | Effects pushed to edges, pure core |
| Type-level guarantees | Invalid states unrepresentable |

---

## Component justifications

### 1. hypertext — HTML as pure functions

**Algebraic justification:**

HTML generation is a *monoid* under concatenation.
Hypertext makes this explicit:

```rust
// Renderable is a monoid: empty element + associative composition
trait Renderable {
    fn render_to(&self, output: &mut String);
}

// Pure function: no allocation, no side effects
fn header() -> impl Renderable { maud! { header { "..." } } }
fn content(items: &[Item]) -> impl Renderable { maud! { main { ... } } }
fn footer() -> impl Renderable { maud! { footer { "..." } } }

// Monoidal composition (associative, identity exists)
fn page(items: &[Item]) -> impl Renderable {
    maud! {
        (header())      // No effect
        (content(items)) // No effect
        (footer())      // No effect
    }
}
// Effect (allocation) deferred to boundary: page.render()
```

**Why not maud directly:**

- Maud's `Markup` type is *eager*—it allocates on construction
- Hypertext's `impl Renderable` is a *thunk*—a description of computation
- This follows the *free monad* pattern: build a description, interpret at the boundary

**Type-safety:**

- Compile-time HTML validation
- No runtime template parsing errors
- Datastar attributes (`data-signals`, `data-on:*`) are stringly-typed at the HTML level but structurally validated by the macro

---

### 2. axum — effect boundaries via extractors

**Algebraic justification:**

Axum's extractor pattern is essentially a *Reader monad* reified as types:

```rust
// Extractors are Reader<Request, Result<T, Rejection>>
async fn handler(
    State(db): State<Pool>,           // Reader effect: access environment
    Path(id): Path<String>,           // Parser effect: extract from path
    Json(cmd): Json<Command>,         // Parser effect: deserialize body
) -> Result<impl IntoResponse, AppError> {  // Error effect explicit
    // Pure business logic here
}
```

**Effect isolation:**

- Extractors handle IO/parsing at the boundary
- Handler body can be pure computation
- `Result` and `impl IntoResponse` make effects explicit in return type

**SSE as a lazy stream:**

```rust
// Sse<S> is essentially Free[Stream, Event] — description of effects
async fn events(State(store): State<EventStore>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = store.subscribe()
        .map(|e| Ok(Event::default().data(e.to_json())));
    Sse::new(stream)  // No events emitted until consumed
}
```

---

### 3. tokio::sync::broadcast — event bus as observable

**Algebraic justification:**

The broadcast channel implements the *Observer pattern* as a pure data flow:

```rust
// Sender<T> + Receiver<T> form a comonadic structure
// - Sender: coalgebraic (produces values)
// - Receiver: algebraic (consumes values)

pub struct EventBus {
    tx: broadcast::Sender<DomainEvent>,
}

impl EventBus {
    // Pure: returns a new receiver, no mutation
    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.tx.subscribe()
    }

    // Effect explicit: Result indicates success/failure
    pub fn publish(&self, event: DomainEvent) -> Result<usize, SendError<DomainEvent>> {
        self.tx.send(event)
    }
}
```

**Replacing NATS KV Watch:**

- NATS Watch is effectful and external
- tokio broadcast is in-process, deterministic, and composable
- No network effects in the notification path

---

### 4. SQLite + sqlx — event store with type-safe queries

**Algebraic justification:**

sqlx provides compile-time query validation—the query is a *type-level proof* that it's valid:

```rust
// query_as! validates SQL at compile time against actual schema
// This is a dependent type approximation: Query<SQL, Schema> -> Result<T, E>
let events = sqlx::query_as!(
    StoredEvent,
    r#"
    SELECT id, aggregate_type, aggregate_id, sequence, event_type, payload, created_at
    FROM events
    WHERE aggregate_id = ?
    ORDER BY sequence
    "#,
    aggregate_id
)
.fetch_all(&pool)
.await?;
```

**Event sourcing as append-only:**

```rust
// Events form a monoid (concatenation) with identity (empty stream)
// Append is the only mutation — no update, no delete
pub async fn append(&self, events: Vec<NewEvent>) -> Result<Vec<StoredEvent>> {
    let mut tx = self.pool.begin().await?;

    // Transaction as a bracketed effect: begin -> operations -> commit/rollback
    for event in events {
        sqlx::query!(...).execute(&mut *tx).await?;
    }

    tx.commit().await?; // Effect realized at boundary

    // Publish to observers (pure data flow)
    for event in &stored {
        let _ = self.bus.publish(event.clone());
    }

    Ok(stored)
}
```

**Why SQLite over NATS:**

- SQLite's durability model is *synchronous* by default (WAL + fsync)
- No lazy fsync surprises—effects happen when you commit
- Single-writer semantics prevent split-brain by construction

---

### 5. redb — session state with ACID guarantees

**Algebraic justification:**

redb's transaction model is a *bracket* pattern:

```rust
// WriteTransaction is a linear type (must be committed or dropped)
// This enforces the bracket law: acquire -> use -> release
let txn = db.begin_write()?;  // Acquire
{
    let mut table = txn.open_table(SESSIONS)?;
    table.insert(key, value)?;  // Use (pure within transaction)
}
txn.commit()?;  // Release (effect realized)
```

**Durability as explicit choice:**

```rust
// 1PC+C: single fsync, checksums detect partial writes
// 2PC: two fsyncs, stronger guarantee
// The choice is explicit in the API, not hidden
db.set_two_phase_commit(true);
```

**Why redb for session state:**

- Session state is ephemeral but should survive restarts
- TTL logic is application-level (functional, testable)
- No NATS KV durability surprises

---

### 6. DuckDB — analytics as pure queries

**Algebraic justification:**

DuckDB queries are *referentially transparent*—same input, same output:

```rust
// Analytical query is a pure function: Projection -> Result<DataFrame>
let results = conn.execute(
    "SELECT aggregate_type, COUNT(*) as event_count
     FROM events
     GROUP BY aggregate_type",
    []
)?;
```

**Separation of concerns:**

- SQLite: OLTP (transactional event store)
- DuckDB: OLAP (analytical projections)
- This is the *CQRS* pattern: commands and queries have different algebra

---

### 7. Zenoh — future distribution via unified abstraction

**Algebraic justification:**

Zenoh's key-expression model is a *free monoid* over path segments:

```rust
// Key expressions form a monoid under path concatenation
// Pattern matching is a semilattice (wildcards, unions)
let key = format!("events/{}/{}/{}", aggregate_type, aggregate_id, sequence);

// Put is an effectful operation (IO monad)
session.put(&key, payload).await?;

// Subscribe returns a stream (comonadic, produces values)
let subscriber = session.subscribe("events/**").await?;
```

**Why Zenoh over Apache Iggy:**

- Zenoh has both streaming and storage in one abstraction
- Storage backends (RocksDB, S3) provide durability
- Subscriptions provide the "watch" semantics
- More production-ready (Eclipse Foundation, April 2025 Gozuryū release)

**Migration path:**

```rust
// Trait abstraction allows swapping implementations
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, events: Vec<Event>) -> Result<Vec<StoredEvent>>;
    async fn load(&self, aggregate_id: &str) -> Result<Vec<StoredEvent>>;
    fn subscribe(&self) -> impl Stream<Item = StoredEvent>;
}

// Phase 1: SQLite implementation
pub struct SqliteEventStore { pool: SqlitePool, bus: EventBus }

// Phase 2: Zenoh implementation (same trait)
pub struct ZenohEventStore { session: zenoh::Session }
```

---

### 8. datastar-rust — frontend as signal algebra

**Algebraic justification:**

Datastar's signals are a *reactive graph*—essentially FRP (Functional Reactive Programming):

```rust
// Signals form an applicative functor
// - pure: lift value into signal
// - ap: combine signals
// - map: transform signal values

// PatchSignals is a morphism: JSON -> Signal State
PatchSignals::new(r#"{"count": 0, "loading": false}"#)

// PatchElements is a morphism: HTML -> DOM
PatchElements::new(render_component(state))
```

**SSE as a stream of patches:**

```rust
// The SSE stream is Free[Patch, DOM] — a program describing UI updates
async fn counter_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::unfold(0, |count| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let patch = PatchSignals::new(format!(r#"{{"count": {}}}"#, count + 1));
        Some((Ok(patch.write_as_axum_sse_event()), count + 1))
    });
    Sse::new(stream)
}
```

---

### 9. tailwindcss — styling as data

**Algebraic justification:**

Tailwind classes are a *combinator language* for styles:

```rust
// Classes compose via string concatenation (monoid)
// Each class is a morphism: ClassName -> CSSProperty
maud! {
    div class="flex items-center justify-between p-4 bg-gray-100" {
        // flex: display -> flex
        // items-center: align-items -> center
        // Composition is associative, order-independent for non-conflicting properties
    }
}
```

**Nix integration:**

- tailwindcss CLI installed via flake
- Build-time CSS generation (no runtime)
- Deterministic output

---

### 10. process-compose — orchestration as declarative spec

**Algebraic justification:**

Process-compose configurations are *declarative specifications* of system topology:

```yaml
# This is a product type: Process = { command, depends_on, environment, ... }
processes:
  ironstar:
    command: ./result/bin/ironstar
    depends_on:
      tailwind: { condition: process_completed_successfully }
    environment:
      DATABASE_URL: "sqlite:./data/ironstar.db"

  tailwind:
    command: tailwindcss -i input.css -o static/output.css --minify
```

**Why not docker-compose:**

- Nix provides reproducible builds
- process-compose is lighter, no container overhead
- Better for development iteration

---

## Ironstar architecture summary

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Ironstar Template                            │
├─────────────────────────────────────────────────────────────────────┤
│  Boundary Layer (Effects)                                           │
│  ┌─────────────┬─────────────┬─────────────┬─────────────────────┐ │
│  │ axum        │ SSE Stream  │ HTTP Req/Res│ WebSocket (future)  │ │
│  │ extractors  │ (lazy)      │ (bounded)   │                     │ │
│  └─────────────┴─────────────┴─────────────┴─────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│  Application Layer (Pure Functions)                                 │
│  ┌─────────────┬─────────────┬─────────────┬─────────────────────┐ │
│  │ Command     │ Query       │ Event       │ Projection          │ │
│  │ Handlers    │ Handlers    │ Handlers    │ Updaters            │ │
│  │ Cmd -> Evts │ Query -> RM │ Evt -> SSE  │ Evt -> ReadModel    │ │
│  └─────────────┴─────────────┴─────────────┴─────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│  Domain Layer (Algebraic Types)                                     │
│  ┌─────────────┬─────────────┬─────────────┬─────────────────────┐ │
│  │ Aggregates  │ Events      │ Commands    │ Value Objects       │ │
│  │ (Sum types) │ (Sum types) │ (Sum types) │ (Product types)     │ │
│  └─────────────┴─────────────┴─────────────┴─────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│  Infrastructure Layer (Effect Implementations)                      │
│  ┌─────────────┬─────────────┬─────────────┬─────────────────────┐ │
│  │ SQLite/sqlx │ redb        │ DuckDB      │ Zenoh (future)      │ │
│  │ EventStore  │ SessionKV   │ Analytics   │ Distributed         │ │
│  └─────────────┴─────────────┴─────────────┴─────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│  Presentation Layer (Lazy Rendering)                                │
│  ┌─────────────┬─────────────┬─────────────┬─────────────────────┐ │
│  │ hypertext   │ datastar    │ tailwindcss │ Static assets       │ │
│  │ (thunks)    │ (signals)   │ (classes)   │                     │ │
│  └─────────────┴─────────────┴─────────────┴─────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Component selection matrix

| Component | Role | Algebraic Property | Effect Boundary |
|-----------|------|-------------------|-----------------|
| **hypertext** | HTML | Monoid (lazy) | `.render()` |
| **axum** | HTTP | Reader + Error | Handler return |
| **tokio::broadcast** | Event bus | Observable | `.send()` |
| **SQLite/sqlx** | Event store | Append monoid | `.commit()` |
| **redb** | Session KV | Bracket (linear) | `.commit()` |
| **DuckDB** | Analytics | Pure query | `.execute()` |
| **Zenoh** | Distribution | Free monoid | `.put()` / `.subscribe()` |
| **datastar-rust** | Frontend | FRP signals | SSE emit |
| **tailwindcss** | Styling | Class combinators | Build time |
| **process-compose** | Orchestration | Product spec | Process start |

This stack achieves the goal: **effects explicit in types, isolated at boundaries, with a pure functional core**.

---

## Context: why not NATS?

This stack was designed in part as a response to the [Jepsen analysis of NATS 2.12.1](https://jepsen.io/analyses/nats-2.12.1), which identified critical durability failures:

1. **Lazy fsync default** — acknowledged writes lost on crash (2-minute flush window)
2. **Minority corruption propagation** — single-node file corruption caused majority data loss
3. **Split-brain from single OS crash** — persistent replica divergence after power failure

For event sourcing, these issues are catastrophic—you cannot lose events from the middle of a stream without corrupting all downstream state.

The SQLite + tokio broadcast approach avoids all these:

- SQLite's WAL + fsync is synchronous by default
- Single-node means no split-brain by construction
- No corruption propagation between replicas
- tokio broadcast is in-memory notification only (events already durably stored)

When distribution is needed, Zenoh provides safer alternatives with explicit durability controls and production-ready status (Eclipse Foundation, April 2025 Gozuryū release).
