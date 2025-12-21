# Ironstar architecture decisions index

This document serves as the index for ironstar's architecture decision records (ADRs).
Detailed technology selection decisions are organized into focused topic documents.
For foundational principles guiding these decisions, see `design-principles.md`.

## Decision documents

### Frontend stack decisions

**Document**: `frontend-stack-decisions.md`

Covers frontend technology choices including:
- datastar-rust for reactive SSE-driven UI
- Open Props design tokens (CSS custom properties)
- Open Props UI component library (pure CSS, copy-paste model)
- Lucide icons (build-time SVG inlining)
- Web component integration patterns

**Key decisions**:
- Pure CSS with runtime variables over utility-first frameworks (Tailwind)
- Vanilla web components for third-party library wrappers
- Server-driven reactivity via Datastar signals and SSE
- Zero-runtime icon embedding

### Backend core decisions

**Document**: `backend-core-decisions.md`

Covers backend core technology choices including:
- hypertext for lazy HTML templating (compile-time validation)
- axum web framework (extractors as Reader monad)
- SQLite + sqlx for event store (compile-time query validation)
- DuckDB for analytics (OLAP, remote data via httpfs)

**Key decisions**:
- Lazy HTML generation via hypertext thunks over eager maud
- SQLite for both events (append-only) and sessions (colocated)
- DuckDB for analytics with support for remote parquet datasets (HuggingFace, S3)

### Infrastructure decisions

**Document**: `infrastructure-decisions.md`

Covers infrastructure technology choices including:
- tokio::sync::broadcast for in-process event bus
- SQLite sessions colocated with event store
- moka for analytics cache with rkyv serialization
- Zenoh for future distributed deployment
- rust-embed for static asset embedding
- process-compose for development orchestration

**Key decisions**:
- Embedded components over external services (single-node deployment target)
- SQLite sessions instead of separate redb database
- moka cache with TTL-based eviction for analytics results
- Zenoh as migration path for distributed deployment

### CQRS implementation decisions

**Document**: `cqrs-implementation-decisions.md`

Covers CQRS and event sourcing implementation including:
- Pure synchronous aggregate pattern (inspired by esrs)
- Event schema evolution via Upcaster pattern
- Framework evaluation rationale (why custom over cqrs-es/esrs)

**Key decisions**:
- Pure aggregates with no async effects
- Custom CQRS implementation adopting patterns from cqrs-es, esrs, and sqlite-es
- Upcaster pattern for backward-compatible event schema evolution

### Build tooling decisions

**Document**: `build-tooling-decisions.md`

Covers build tooling technology choices including:
- Rolldown for JavaScript/CSS bundling (Rust-native)
- PostCSS for CSS transforms (imports, autoprefixing, minification)
- Build pipeline architecture (dev vs. prod modes)

**Key decisions**:
- Rolldown over esbuild (Rust-native, better tree-shaking)
- PostCSS for CSS processing without JIT scanning
- Content hashing via bundler, not runtime

---

## Architecture summary

This diagram shows the 5-layer operational view of ironstar's architecture.
For the complete 7-layer crate decomposition plan, see `crate-architecture.md`.

**Layer mapping**: Boundary Layer → Layer 6 (Presentation), Application Layer → Layers 2-3 (Application + Interfaces), Domain Layer → Layers 0-1 (Foundation + Domain), Infrastructure Layer → Layers 4-5 (Infrastructure + Services), Presentation Layer → Layer 6 (Presentation, frontend components).

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
│  │ SQLite/sqlx │ DuckDB      │ moka        │ Zenoh (future)      │ │
│  │ Events+Sess │ Analytics   │ Cache       │ Distributed         │ │
│  └─────────────┴─────────────┴─────────────┴─────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│  Presentation Layer (Lazy Rendering)                                │
│  ┌─────────────┬─────────────┬─────────────┬─────────────────────┐ │
│  │ hypertext   │ datastar    │ open-props  │ open-props-ui       │ │
│  │ (thunks)    │ (signals)   │ (tokens)    │ (components)        │ │
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
| **SQLite/sqlx** | Event store + sessions | Append monoid | `.commit()` |
| **moka** | Analytics cache | TTL-based eviction | Cache hit/miss |
| **rkyv** | Cache serialization | Zero-copy deserialize | Serialize/deserialize boundary |
| **DuckDB** | Analytics | Pure query | `.execute()` |
| **Zenoh** | Distribution | Free monoid | `.put()` / `.subscribe()` |
| **datastar-rust** | Frontend | FRP signals | SSE emit |
| **open-props** | CSS Tokens | Constants (map/dictionary) | CSS `var()` resolution |
| **open-props-ui** | CSS Components | Three-layer composition | Style application |
| **process-compose** | Orchestration | Product spec | Process start |
| **Rolldown** | JS/CSS bundler | Pure morphism (deterministic) | `rolldown build` |
| **Lucide** | Icons | Constants (Yoneda embedding) | Build time |
| **rust-embed** | Asset embedding | Compile-time constants | `cargo build --release` |
| **Pure Aggregate** | Domain logic | State machine (pure function) | None (pure) |
| **Upcaster** | Schema evolution | Category of versions | Event load |

This stack achieves the goal: **effects explicit in types, isolated at boundaries, with a pure functional core**.

---

## Architectural context: embedded vs. external services

This stack prioritizes embedded Rust-native solutions over external server dependencies.

**Why embedded:**

- Single binary deployment (no orchestration of multiple services)
- No network effects in the critical path (in-process communication)
- Rust-native dependencies align with the stack's language choice
- Simpler operational model for single-node deployments

**NATS as a valid alternative:**

NATS is an excellent choice for teams willing to run an external server.
It provides streaming, key-value storage, and pub/sub in a unified abstraction, and the Rust client (nats.rs) is production-ready.

For Ironstar, the embedded approach was chosen because the template targets single-node deployments where the operational complexity of a separate server is unnecessary.
The [Jepsen analysis of NATS 2.12.1](https://jepsen.io/analyses/nats-2.12.1) also reinforced confidence in SQLite's well-understood durability model for the event store, though NATS's durability can be configured appropriately for many use cases.

**Future distribution:**

When distributed deployment is needed, Zenoh provides Rust-native pub/sub with storage backends (RocksDB, S3), offering a migration path that maintains the embedded philosophy per node while enabling cross-node communication.

---

## Module organization and scaling path

Ironstar's module structure follows CQRS boundaries with clear separation between domain, application, infrastructure, and presentation layers.

### Initial structure (single crate)

```
src/
├── main.rs                      # Entry point, router composition
├── lib.rs                       # Public API, re-exports
├── config.rs                    # Environment configuration
├── domain/                      # Algebraic types (pure)
│   ├── mod.rs
│   ├── aggregates/              # State machines (sum types)
│   ├── events.rs                # Domain events (sum type)
│   ├── commands.rs              # Command types (product types)
│   └── signals.rs               # Datastar signal types (ts-rs derives)
├── application/                 # Business logic (pure functions)
│   ├── mod.rs
│   ├── command_handlers.rs      # Command → Events
│   ├── query_handlers.rs        # Query → ReadModel
│   └── projections.rs           # Event → ReadModel updates
├── infrastructure/              # Effect implementations
│   ├── mod.rs
│   ├── event_store.rs           # SQLite event persistence
│   ├── session_store.rs         # SQLite session storage
│   ├── analytics.rs             # DuckDB queries
│   ├── analytics_cache.rs       # moka cache with rkyv serialization
│   └── event_bus.rs             # tokio::broadcast coordination
├── presentation/                # HTTP + HTML (effects at boundary)
│   ├── mod.rs
│   ├── routes.rs                # Router composition
│   ├── handlers/                # Axum handlers
│   │   ├── sse.rs               # SSE feed handlers
│   │   └── commands.rs          # POST command handlers
│   ├── templates/               # hypertext components
│   │   ├── layouts.rs           # Base layouts
│   │   ├── pages/               # Full page templates
│   │   └── components/          # Reusable fragments
│   └── assets.rs                # Static asset serving (rust-embed)
└── features/                    # Optional: feature-based grouping
    └── todos/                   # Self-contained feature module
        ├── mod.rs
        ├── routes.rs
        ├── handlers.rs
        └── templates.rs
```

### Layer responsibilities

| Layer | Purity | Dependencies | Responsibility |
|-------|--------|--------------|----------------|
| **Domain** | Pure | None | Types, validation, business rules |
| **Application** | Pure | Domain | Command/query handling, projections |
| **Infrastructure** | Effectful | Domain, Application | Persistence, external services |
| **Presentation** | Effectful | All | HTTP, HTML, routing |

### Feature module pattern

For larger applications, self-contained feature modules group related functionality:

```rust
// src/features/todos/mod.rs
pub mod routes;
pub mod handlers;
pub mod templates;

// src/features/todos/routes.rs
use axum::{routing::{get, post}, Router};
use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::index_page))
        .route("/api/todos", get(handlers::todos_sse))
        .route("/api/todos", post(handlers::add_todo))
        .route("/api/todos/:id/toggle", post(handlers::toggle_todo))
}

// src/main.rs - Compose feature routers
let app = Router::new()
    .merge(features::todos::routes())
    .merge(features::counter::routes())
    .with_state(app_state);
```

This pattern mirrors northstar's `features/*/routes.go` organization.

### Workspace scaling path

When the codebase grows, extract layers into separate crates following the multi-crate architecture documented in `crate-architecture.md`.
The structure draws from patterns in Golem (~25 crates) and Hyperswitch (~40 crates).

**Key patterns adopted:**

| Pattern | Source | Purpose |
|---------|--------|---------|
| HasXxx capability traits | Golem | Fine-grained dependency injection |
| All composition root | Golem | Central service wiring |
| Three commons (enums/types/utils) | Hyperswitch | Foundation layer separation |
| Interfaces crate | Hyperswitch | Port trait definitions |
| Configuration-driven adapters | Golem | Runtime backend selection |
| Workspace lints | Hyperswitch | Consistent code quality |

**Layered crate structure:**

```
Layer 0 (Foundation): common-enums, common-types, common-utils
Layer 1 (Domain): ironstar-domain, ironstar-commands, ironstar-events
Layer 2 (Application): ironstar-app
Layer 3 (Interfaces): ironstar-interfaces
Layer 4 (Infrastructure): ironstar-adapters, ironstar-analytics, ironstar-projections, ironstar-config
Layer 5 (Services): ironstar-services
Layer 6 (Presentation): ironstar-web
Layer 7 (Binary): ironstar
```

Crate names use kebab-case following crates.io convention.
Rust normalizes these to snake_case for `use` statements.

Each layer can only depend on layers below it.
See `crate-architecture.md` for detailed directory structure, trait definitions, and migration strategy.

**Per-crate Nix configuration:**

Each crate can have a `crate.nix` file for customized build requirements (e.g., additional build inputs for DuckDB or OpenSSL).
This pattern is used by rust-flake for automatic workspace discovery and per-crate crane configuration.

### AppState composition

Dependency injection uses axum's `State` extractor with a shared state struct:

```rust
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub event_store: Arc<SqliteEventStore>,
    pub session_store: Arc<SessionStore>,       // SQLite-based
    pub analytics: Arc<DuckDbAnalytics>,
    pub analytics_cache: AnalyticsCache,        // moka-based
    pub event_bus: broadcast::Sender<StoredEvent>,
    pub projections: Arc<Projections>,
    #[cfg(debug_assertions)]
    pub reload_tx: broadcast::Sender<()>,
}

impl AppState {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let pool = SqlitePool::connect(&config.database_url).await?;
        let event_store = SqliteEventStore::new(pool.clone()).await?;
        let session_store = SessionStore::new(pool, 30); // 30-day TTL
        let analytics = Arc::new(DuckDbAnalytics::new(&config.analytics_path)?);
        let analytics_cache = AnalyticsCache::new(analytics.clone());
        let (event_bus, _) = broadcast::channel(256);

        // Initialize projections by replaying events
        let projections = Projections::init(&event_store, event_bus.clone()).await?;

        // Spawn cache invalidation task
        tokio::spawn(run_cache_invalidator(
            analytics_cache.clone(),
            event_bus.subscribe(),
        ));

        Ok(Self {
            event_store: Arc::new(event_store),
            session_store: Arc::new(session_store),
            analytics,
            analytics_cache,
            event_bus,
            projections: Arc::new(projections),
            #[cfg(debug_assertions)]
            reload_tx: broadcast::channel(16).0,
        })
    }
    // Note: Error type is application-specific, see event-sourcing-core.md appendix
}
```

Handlers extract what they need:

```rust
use axum::extract::State;
use axum::response::IntoResponse;
use datastar::axum::ReadSignals;

async fn add_todo(
    State(state): State<AppState>,
    ReadSignals(signals): ReadSignals<TodoSignals>,
) -> impl IntoResponse {
    // Access state.event_store, state.event_bus, etc.
}
```

---

## Related documentation

### Architecture decision documents

- **Frontend stack**: `frontend-stack-decisions.md`
- **Backend core**: `backend-core-decisions.md`
- **Infrastructure**: `infrastructure-decisions.md`
- **CQRS implementation**: `cqrs-implementation-decisions.md`
- **Build tooling**: `build-tooling-decisions.md`

### Design and patterns

- **Design principles**: `design-principles.md`
- **Crate architecture**: `crate-architecture.md`
- **Event sourcing core concepts**: `event-sourcing-core.md`
- **SSE connection lifecycle**: `sse-connection-lifecycle.md`
- **Command write patterns**: `command-write-patterns.md`
- **Analytics cache design**: `analytics-cache-architecture.md`
- **Third-party integration**: `integration-patterns.md`

### Implementation guides

- **Development workflow**: `development-workflow.md`
- **Signal contracts**: `signal-contracts.md`
- **Build pipeline**: `frontend-build-pipeline.md`
- **Session management**: `session-management.md`
