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

**Note**: This 3-layer conceptual model maps to the 7-layer crate architecture: Domain corresponds to Layers 0-1 (Foundation + Domain), Application to Layers 2-3 (Application + Interfaces), and Infrastructure to Layers 4-5 (Infrastructure + Services).
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

## Related documentation

For specific technology choices and component justifications, see `architecture-decisions.md`.
