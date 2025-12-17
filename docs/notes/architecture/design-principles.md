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
