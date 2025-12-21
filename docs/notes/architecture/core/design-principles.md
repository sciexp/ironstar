# Ironstar design principles

This document establishes foundational design principles for ironstar.

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

## Domain layer principles

The domain layer contains pure business logic with no infrastructure dependencies.
This purity enables local reasoning, trivial testing, and clean separation of concerns.

### Pure aggregates

Aggregates are pure synchronous functions with no side effects.
The `handle_command` function receives immutable state and a command, returning either events or an error.
No I/O, no async, no hidden dependencies.

```rust
impl Order {
    pub fn handle_command(
        state: &OrderState,
        cmd: OrderCommand,
    ) -> Result<Vec<OrderEvent>, OrderError> {
        match cmd {
            OrderCommand::Pay { amount } => {
                if state.status != OrderStatus::Pending {
                    return Err(OrderError::InvalidTransition);
                }
                if amount != state.total {
                    return Err(OrderError::AmountMismatch);
                }
                Ok(vec![OrderEvent::Paid { amount }])
            }
            OrderCommand::Ship => {
                if state.status != OrderStatus::Paid {
                    return Err(OrderError::NotPaid);
                }
                Ok(vec![OrderEvent::Shipped])
            }
        }
    }

    pub fn apply_event(state: OrderState, event: &OrderEvent) -> OrderState {
        match event {
            OrderEvent::Paid { .. } => OrderState {
                status: OrderStatus::Paid,
                ..state
            },
            OrderEvent::Shipped => OrderState {
                status: OrderStatus::Shipped,
                ..state
            },
        }
    }
}
```

This purity has algebraic significance: aggregates form a *coalgebra* over the state type, with events as the coproduct.
The `apply_event` function is the unfold operation, reconstructing state from the event stream.

External service calls (API lookups, validation against external data) happen in the command handler *before* calling the aggregate.
The aggregate receives pre-validated, pre-enriched data.

### Effect boundaries

The design separates pure computation from effectful operations across three conceptual layers:

| Layer | Responsibility | Effects |
|-------|----------------|---------|
| Domain | Algebraic types, pure validation, state machines | None |
| Application | Command/query handlers at the async boundary | I/O coordination |
| Infrastructure | Effect implementations (database, network, pub/sub) | All I/O |

The async/sync boundary is the effect boundary: if a function is `async`, it performs I/O; if sync, it is pure.

**Note**: This 3-layer conceptual model maps to the 8-layer crate architecture (layers 0-7): Domain corresponds to Layers 0-1 (Foundation + Domain), Application to Layers 2-3 (Application + Interfaces), and Infrastructure to Layers 4-5 (Infrastructure + Services).
See `crate-architecture.md` for the complete layering and multi-crate decomposition plan.

```
Domain Layer (Pure)
    Aggregate::handle_command — sync, pure, no I/O
    Aggregate::apply_event — sync, pure, deterministic

Application Layer (Effect Boundary)
    handle_command handler — async, I/O for loading/saving
    Projection updaters — async, I/O for read models

Infrastructure Layer (Effect Implementation)
    EventStore::append — async, database I/O
    EventBus::publish — async, channel send
```

This separation enables:

- Testing domain logic without mocks or fixtures
- Swapping infrastructure implementations without touching domain code
- Reasoning about business rules independent of I/O concerns

### Testing pure aggregates

Because aggregates are pure functions, testing requires no infrastructure.

```rust
#[test]
fn aggregate_rejects_shipping_unpaid_order() {
    let state = OrderState {
        status: OrderStatus::Pending,
        total: 100,
    };
    let cmd = OrderCommand::Ship;

    let result = Order::handle_command(&state, cmd);

    assert!(matches!(result, Err(OrderError::NotPaid)));
}

#[test]
fn aggregate_accepts_valid_payment() {
    let state = OrderState {
        status: OrderStatus::Pending,
        total: 100,
    };
    let cmd = OrderCommand::Pay { amount: 100 };

    let result = Order::handle_command(&state, cmd);

    assert!(matches!(
        result.as_deref(),
        Ok(&[OrderEvent::Paid { amount: 100 }])
    ));
}
```

No database setup, no async runtime, no mocking.
This is the payoff of pure aggregates: business logic becomes as testable as a calculator.

---

## Frontend architecture philosophy

Datastar's core principle is *server-driven UI*: the server sends HTML fragments and signal updates via SSE, and the browser renders them.
This inverts the SPA model where the client manages state and the server provides JSON APIs.
The Tao of Datastar states: "Most state should live in the backend. Since the frontend is exposed to the user, the backend should be the source of truth."

This architectural constraint dramatically simplifies frontend tooling requirements.
When the server drives state, client-side reactivity frameworks become redundant rather than complementary.
We need only thin presentation layers that compose with Datastar's signal system rather than competing with it.

### Anti-pattern: client-side reactivity duplication

Lit, React, Vue, and Svelte each provide their own reactivity systems.
When paired with Datastar, you would have two competing reactivity graphs: the framework's internal state and Datastar's signals.
This leads to state synchronization bugs, increased bundle size, and architectural incoherence.

Datastar already provides:

- Signals (`$foo`) — reactive variables with automatic change propagation
- Computed signals (`data-computed`) — derived state
- Two-way binding (`data-bind`) — form inputs synchronized with signals
- Conditional rendering (`data-show`) — declarative visibility
- Class binding (`data-class`) — reactive styling

These capabilities cover 90% of what React hooks or Vue reactivity provide, but with server-driven truth.
The remaining 10% (complex client-only interactions like drag-and-drop, rich text editing, or data visualization) are handled via thin vanilla web components that emit events for Datastar to process.

### Anti-pattern: Rust WASM frameworks

Leptos and Dioxus are excellent frameworks for building SPAs in Rust with WASM.
However, they embody the SPA philosophy: compile Rust to WASM, run a reactive framework in the browser, manage state on the client.

This directly contradicts the hypermedia-driven architecture:

```
SPA Model:      Server → JSON → Client (WASM/JS) → Virtual DOM → Render
Hypermedia:     Server → HTML → Browser native DOM
```

Using Leptos or Dioxus alongside Datastar would create two parallel systems for the same job.
Ironstar commits to the hypermedia approach fully, using the browser's native capabilities augmented by Datastar's signal system.

---

## Glossary

This glossary defines key architectural terms used throughout ironstar's documentation.
For detailed implementation patterns, see the linked documents.

### Port trait

A port trait defines the interface contract for an infrastructure capability without specifying its implementation.
Port traits live in the `ironstar-interfaces` crate and use `async_trait` for async methods.

```rust
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, event: NewEvent) -> Result<StoredEvent, EventStoreError>;
    async fn load_all(&self) -> Result<Vec<StoredEvent>, EventStoreError>;
}
```

Port traits enable dependency inversion: domain and application layers depend on abstract interfaces, not concrete implementations.
This allows swapping implementations for testing (in-memory mock) or production (SQLite, PostgreSQL) without changing domain logic.
See `crate-services-composition.md` for port trait organization patterns.

### Adapter

An adapter is a concrete implementation of a port trait, translating between the domain's interface expectations and a specific infrastructure technology.

```rust
pub struct SqliteEventStore {
    pool: SqlitePool,
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn append(&self, event: NewEvent) -> Result<StoredEvent, EventStoreError> {
        // SQLite-specific implementation
    }
}
```

Adapters live in the `ironstar-adapters` crate and contain all infrastructure-specific code (SQL queries, HTTP clients, file I/O).
Multiple adapters can implement the same port trait, enabling runtime backend selection via configuration.
See `crate-services-composition.md` section "Configuration-driven adapter selection" for the tagged enum pattern.

### HasXxx trait

A HasXxx trait provides access to a specific capability through a getter method.
This pattern enables fine-grained dependency injection where functions declare exactly which services they need.

```rust
pub trait HasEventStore {
    fn event_store(&self) -> Arc<dyn EventStore>;
}

async fn handle_command<S: HasEventStore>(services: &S, cmd: Command) {
    services.event_store().append(event).await?;
}
```

HasXxx traits are composable: a struct implementing multiple HasXxx traits can be passed to any function requiring a subset of those capabilities.
This avoids the God object anti-pattern where every handler receives the entire AppState.
See `crate-services-composition.md` section "HasXxx capability trait pattern" for complete examples.

### All composition root

The All struct implements all HasXxx traits, serving as the central service registry that wires up dependencies.
It is constructed once at application startup and cloned into each request context.

```rust
#[derive(Clone)]
pub struct All {
    event_store: Arc<dyn EventStore>,
    session_store: Arc<dyn SessionStore>,
    // ... all services
}

impl HasEventStore for All {
    fn event_store(&self) -> Arc<dyn EventStore> { self.event_store.clone() }
}
```

The name "All" comes from Golem's composition pattern, reflecting its role as the universal provider of capabilities.
Handlers that need everything can require `impl HasAll`, while specialized handlers can require specific trait combinations.
See `crate-services-composition.md` section "Composition root" for initialization patterns.

### Upcaster

An Upcaster transforms events from old schema versions to the current schema during event loading, enabling backward-compatible schema evolution without data migrations.

```rust
pub trait EventUpcaster {
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool;
    fn upcast(&self, payload: Value) -> Value;
}
```

Upcasters are registered in an UpcasterChain and applied lazily when loading events from the event store.
This keeps the event store immutable (events are historical facts that never change) while allowing the domain model to evolve.
See `../cqrs/event-sourcing-core.md` section "Event schema evolution with upcasters" for complete implementation.

### Projection

A projection is a read model derived from events, optimized for specific query patterns.
Projections subscribe to the event bus and update their state in response to events.

```rust
pub struct TodoListProjection {
    todos: HashMap<String, TodoState>,
}

impl TodoListProjection {
    pub fn handle_event(&mut self, event: &TodoEvent) {
        match event {
            TodoEvent::Created { id, text } => {
                self.todos.insert(id.clone(), TodoState { text: text.clone(), completed: false });
            }
            TodoEvent::Completed { id } => {
                if let Some(todo) = self.todos.get_mut(id) {
                    todo.completed = true;
                }
            }
        }
    }
}
```

Projections are ephemeral; they can be deleted and rebuilt from the event store at any time.
This enables schema evolution by replaying events through new projection logic.
See `../cqrs/projection-patterns.md` for caching strategies and DuckDB integration.

### Observable

An Observable represents a stream of values over time that can be subscribed to.
In ironstar, the event bus (`tokio::broadcast::Sender`) is an observable, emitting events to multiple subscribers.

The Observable pattern comes from functional reactive programming (FRP) and can be understood categorically as a coalgebra over the temporal functor.
Concretely, subscribers receive each event exactly once in the order emitted (per Rust's broadcast semantics with buffering).

See `../infrastructure/zenoh-event-bus.md` for the migration path from `tokio::broadcast` to Zenoh's distributed pub/sub when scaling beyond single-node deployments.

### Effect boundary

An effect boundary is the architectural layer where pure computation transitions to effectful I/O operations.
In ironstar, the async/sync boundary marks the effect boundary: async functions perform I/O, sync functions are pure.

```rust
// Pure domain logic (sync, no effects)
impl Order {
    pub fn handle_command(state: &OrderState, cmd: OrderCommand) -> Result<Vec<OrderEvent>, OrderError> {
        // Pure computation
    }
}

// Effect boundary (async, performs I/O)
async fn execute_command(cmd: OrderCommand, store: Arc<dyn EventStore>) -> Result<(), AppError> {
    let state = load_state(&store).await?; // Effect: database read
    let events = Order::handle_command(&state, cmd)?; // Pure computation
    for event in events {
        store.append(event).await?; // Effect: database write
    }
    Ok(())
}
```

Isolating effects at boundaries enables local reasoning about pure functions, testability without mocks, and principled composition.
See `design-principles.md` section "Effect boundaries" for the complete layer model.

---

## Related documentation

For specific technology choices and component justifications, see `architecture-decisions.md`.
