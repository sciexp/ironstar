# Ironstar

Ironstar is a Rust + Datastar template for building reactive, event-sourced web applications with hypermedia-driven architecture.
To a first approximation, it is the Rust equivalent of [northstar](https://github.com/zangster300/northstar), whose source code we have a local copy of (see "Datastar ecosystem" table below), adapted for Rust's type system, functional programming idioms, and broader integration of ecosystem components to improve the ability of the template to illustrate how to scale applications.
Documentation follows the typescript-nix-template pattern with Astro Starlight for the production documentation site.
The `docs/` subtree uses symlinks to make Starlight content accessible from the repository root while keeping the actual content in `packages/docs/src/content/docs/`.
Working notes and ephemeral planning documents go in `docs/notes/` (a real directory, not a symlink).
Production documentation that will be published goes in `packages/docs/src/content/docs/` and is accessible via symlinks like `docs/guides/` → `../packages/docs/src/content/docs/guides/`.

## Project status

Early design phase.
Component selection complete, implementation not yet started.

## Template synthesis sources

Ironstar is instantiated as a deliberate synthesis of patterns from multiple template repositories, each contributing specific architectural elements.

### Pattern sources

| Source Repository | Patterns Extracted |
|-------------------|-------------------|
| `~/projects/nix-workspace/typescript-nix-template` | Flake structure with `import-tree`, deferred module composition, category-based CI with content-addressed caching, om CLI instantiation, custom devShell helper scripts |
| `~/projects/rust-workspace/rust-nix-template` | rust-flake integration (crane + rust-overlay), `rust-toolchain.toml` pattern, per-crate `crane.args` configuration, layered devShell composition |
| `~/projects/rust-workspace/rustlings-workspace` | Cargo workspace organization with `resolver = "2"`, `workspace.dependencies` for DRY, `crates/` subdirectory structure, per-crate `crate.nix` files |
| `~/projects/lakescope-workspace/datastar-go-nats-template-northstar` | Datastar SSE architecture, web component integration, single-binary asset embedding, dev/prod mode separation, three-stage build pipeline |

### Template instantiation

Ironstar uses omnix `om` CLI for parameterized instantiation.
Example from `typescript-nix-template` README adapted for ironstar:

```bash
PROJECT_DIRECTORY=my-ironstar-app && \
PARAMS=$(cat <<EOF
{
  "project-name": "$PROJECT_DIRECTORY",
  "crate-name": "my_ironstar_app",
  "github-ci": true,
  "example-todo": true,
  "nix-template": false
}
EOF
) && \
om init github:user/ironstar/main \
  -o "$PROJECT_DIRECTORY" --non-interactive --params "$PARAMS"
```

The template machinery (omnix params, path-conditional includes) follows the pattern from `typescript-nix-template/modules/template.nix`.

### Single-binary asset embedding

Northstar embeds static assets into the Go binary using `//go:embed` + `hashfs` for content-hashed URLs.
Ironstar replicates this pattern for Rust:

| Go (northstar) | Rust (ironstar) |
|----------------|-----------------|
| `//go:embed static` | `rust-embed` crate with `#[derive(RustEmbed)]` |
| `hashfs.NewFS()` content hashing | Rolldown's `[hash]` in output filenames + `manifest.json` |
| Build tags (`!dev` / `dev`) | Conditional compilation (`#[cfg(debug_assertions)]`) |
| `os.DirFS()` for dev | `tower-http::services::ServeDir` for dev |
| `hashfs.FileServer()` for prod | Custom axum handler serving embedded assets |

The build pipeline:

```
web-components/
├── index.ts                    # Entry point
└── styles/main.css             # Open Props imports
        │
        ▼ (Rolldown build)
static/dist/
├── bundle.[hash].js
├── bundle.[hash].css
└── manifest.json               # Maps entry → hashed filename
        │
        ▼ (cargo build --release)
target/release/ironstar          # Single binary with embedded static/dist/
```

Dev mode serves directly from `static/` via `ServeDir` with no caching.
Prod mode embeds `static/dist/` and serves with `Cache-Control: max-age=31536000, immutable`.

### Workspace scaling path

Ironstar starts as a single crate but the workspace structure supports future decomposition:

```
# Initial (single crate)
ironstar/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── domain/
    ├── application/
    ├── infrastructure/
    └── presentation/

# Future (multi-crate workspace)
ironstar/
├── Cargo.toml                    # [workspace] with members
├── crates/
│   ├── ironstar-domain/          # Algebraic types, pure logic
│   ├── ironstar-infra/           # SQLite, DuckDB, moka
│   └── ironstar-web/             # axum, hypertext, datastar
└── ironstar/                     # Main binary, wires crates together
```

The `rustlings-workspace` patterns (workspace.dependencies, per-crate crate.nix) enable this migration without restructuring the Nix configuration.

### Intentional divergences from Northstar

Ironstar adapts Northstar's patterns for Rust's type system and ecosystem conventions.
These divergences are deliberate architectural choices reflecting Rust-native tooling preferences and single-node deployment targets.
The Northstar Go template and datastar-go SDK remain valuable reference implementations for understanding Datastar's SSE streaming architecture and web component integration patterns.

| Northstar Pattern | Ironstar Adaptation | Rationale |
|-------------------|---------------------|-----------|
| Tailwind + DaisyUI | Open Props + Open Props UI | Design tokens over utility classes; better alignment with server-rendered HTML |
| esbuild (Go) | Rolldown (Rust) | Rust-native toolchain consistency |
| Embedded NATS | tokio::sync::broadcast | Single-node deployment target; no external server dependency |
| hashfs runtime hashing | Rolldown content hashing | Hash computed at build time via bundler, not at runtime |
| Templ (Go templates) | hypertext (Rust macros) | Compile-time type-checked HTML with lazy evaluation |
| Air hot reload | cargo-watch + process-compose | Rust ecosystem tooling |
| Task runner (Taskfile) | justfile | Rust ecosystem convention |

## Design philosophy

Effects explicit in type signatures, isolated at boundaries to preserve compositionality.
The stack embodies algebraic data types (sum types for states, product types for data), lawful abstractions, and referential transparency where possible.

See `docs/notes/architecture/design-principles.md` and `docs/notes/architecture/architecture-decisions.md` for detailed design rationale and technology choices.

### The Tao of Datastar (core principles)

Ironstar embodies the principles from the Tao of Datastar (`~/projects/lakescope-workspace/datastar-doc/guide_the_tao_of_datastar.md`):

1. **Backend is source of truth**: No optimistic updates. SSE delivers confirmed state changes.
2. **Patch elements and signals**: Server drives frontend by patching HTML and signals via SSE.
3. **Use signals sparingly**: Signals for UI state (form inputs, visibility); domain state lives in events.
4. **CQRS pattern**: Single long-lived SSE connection for reads, short-lived POST/PUT/DELETE for writes.
5. **In morph we trust**: Default to replacing entire DOM subtrees (fat morph) for resilience.
6. **Loading indicators over deception**: Show progress via `data-indicator`, let SSE confirm completion.

## Stack overview

| Layer | Component | Role |
|-------|-----------|------|
| Web Framework | axum | HTTP, SSE, extractors as Reader monad |
| Async Runtime | tokio | Async effects |
| HTML Templating | hypertext | Lazy rendering (thunks) |
| Frontend Reactivity | datastar-rust | FRP signals via SSE |
| CSS Framework | Open Props + Open Props UI | Design tokens + pure CSS components |
| CSS/JS Build | Rolldown + PostCSS | Rust-native bundler |
| Icons | Lucide (build-time) | Zero-runtime SVG inlining |
| Web Components | Vanilla (when needed) | Thin wrappers for third-party libs |
| Event Store | SQLite + sqlx | Append-only event log |
| Sessions | SQLite + sqlx | Sessions table |
| Analytics | DuckDB | OLAP projections |
| Analytics Cache | moka | In-memory async cache with TTL |
| Event Bus | tokio::sync::broadcast | In-process pub/sub |
| Distribution (future) | Zenoh | Distributed pub/sub + storage |
| Orchestration | process-compose | Declarative process management |
| Environment | Nix flake | Reproducible builds |

### Dependency configuration

Required Cargo feature flags for key dependencies:

```toml
[dependencies]
# axum with SSE support (default features are sufficient)
axum = { version = "0.8", features = ["default"] }

# datastar-rust with axum integration
datastar = { version = "0.3", features = ["axum"] }

# sqlx with SQLite and runtime-tokio
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "json"] }

# DuckDB with bundled build (compiles DuckDB from source)
duckdb = { version = "1.4", features = ["bundled"] }

# rust-embed for asset embedding (conditional)
rust-embed = { version = "8", features = ["include-exclude"] }

# ts-rs for TypeScript type generation
ts-rs = { version = "11.1", features = ["serde-compat", "uuid-impl"] }

# moka for async in-memory caching
moka = { version = "0.12", features = ["future"] }

# rkyv for zero-copy deserialization
rkyv = { version = "0.8", features = ["validation"] }
```

**Feature notes:**

- `datastar` feature `axum` is required for the ReadSignals extractor and Event conversion
- `sqlx` features must match your async runtime (tokio) and database (sqlite)
- `duckdb` feature `bundled` is strongly recommended to avoid system DuckDB version mismatches

## Local dependency paths

All dependencies with local source code available for reference.

### Core Rust dependencies

| Dependency | Local Path | Description |
|------------|------------|-------------|
| axum | `~/projects/rust-workspace/axum` | Web framework with SSE support |
| tokio | `~/projects/rust-workspace/tokio` | Async runtime |
| hypertext | `~/projects/rust-workspace/hypertext` | Lazy HTML templating (maud-compatible syntax) |
| sqlx | `~/projects/rust-workspace/sqlx` | Async SQL with compile-time validation |
| moka | `~/projects/rust-workspace/moka-caching` | Async in-memory cache with TTL for analytics |
| rkyv | `~/projects/rust-workspace/rkyv-deserialization` | Zero-copy deserialization for cache serialization |
| duckdb-rs | `~/projects/omicslake-workspace/duckdb-rs` | DuckDB Rust bindings |
| ts-rs | `~/projects/rust-workspace/ts-rs` | TypeScript type generation from Rust structs |
| cqrs-es | `~/projects/rust-workspace/cqrs-es` | CQRS/ES framework (reference patterns, not dependency) |
| sqlite-es | `~/projects/rust-workspace/sqlite-es` | SQLite backend for cqrs-es (reference for event store schema) |

### CQRS/Event sourcing references

| Reference | Local Path | Patterns to Study |
|-----------|------------|-------------------|
| cqrs-es | `~/projects/rust-workspace/cqrs-es` | Aggregate trait, EventStore abstraction, GenericQuery projections, TestFramework DSL, event upcasting |
| sqlite-es | `~/projects/rust-workspace/sqlite-es` | SQLite event table schema, optimistic locking, stream-based replay |
| esrs (event_sourcing.rs) | `~/projects/rust-workspace/event_sourcing.rs` | Pure sync aggregates, Schema/Upcaster pattern, TransactionalEventHandler vs EventHandler |

These crates are *reference implementations only* — ironstar implements its own CQRS layer following their patterns but adapted for hypertext + datastar integration.
The key adopted patterns are:
- Pure synchronous aggregates (from esrs): `handle_command(state, cmd) -> Result<Vec<Event>, Error>` with no async/side effects
- Event schema evolution via Upcaster pattern (from esrs)
- TestFramework DSL for aggregate testing (from cqrs-es)
- SQLite event store schema with global sequence for SSE Last-Event-ID (adapted from sqlite-es)

### Datastar ecosystem

The Datastar core developers are primarily Go developers, so the Go SDK and templates represent the most mature, fleshed-out examples of high-performance Datastar integration patterns.
When implementing ironstar, study the Go examples (especially northstar) as primary references for patterns like web component integration and SSE streaming architecture, then adapt them for Rust's type system and functional idioms.
Note: northstar uses Tailwind+DaisyUI while ironstar uses Open Props + Open Props UI, so CSS styling patterns will differ.

| Dependency | Local Path | Description |
|------------|------------|-------------|
| datastar | `~/projects/lakescope-workspace/datastar` | Main datastar repository (frontend JS + SDK specs) |
| datastar-doc | `~/projects/lakescope-workspace/datastar-doc` | Local markdown copy of datastar documentation |
| datastar-rust | `~/projects/rust-workspace/datastar-rust` | Rust SDK for SSE generation |
| datastar-rust-lince | `~/projects/rust-workspace/datastar-rust-lince` | Real-world usage example |
| datastar-go | `~/projects/lakescope-workspace/datastar-go` | Go SDK (reference implementation) |
| datastar-go-template-minimal | `~/projects/lakescope-workspace/datastar-go-template-minimal` | Minimal Go template (SSE basics without NATS) |
| northstar | `~/projects/lakescope-workspace/datastar-go-nats-template-northstar` | Full-featured Go template with NATS, Templ, web components |

**Canonical SDK specification**: `~/projects/lakescope-workspace/datastar/sdk/ADR.md`

All ironstar SSE implementations must conform to this specification.
Key types: `PatchElements`, `PatchSignals`, `ReadSignals<T>`.

### Alternative Datastar implementations (reference)

| Implementation | Local Path | Description |
|----------------|------------|-------------|
| http-nu | `~/projects/rust-workspace/http-nu` | Nushell-scriptable HTTP server with Datastar SDK |
| xs | `~/projects/rust-workspace/xs` | Event store with http-nu + Datastar examples |

These projects take a different architectural approach (Nushell scripting vs compiled Rust) but contain useful Datastar integration patterns and edge case handling.
See their TodoMVC implementations for SSE formatting and signal parsing patterns.

### Infrastructure and tooling

| Dependency | Local Path | Description |
|------------|------------|-------------|
| process-compose | `~/projects/nix-workspace/process-compose` | Process orchestration |
| process-compose-flake | `~/projects/nix-workspace/process-compose-flake` | Nix flake integration |
| rolldown | `~/projects/rust-workspace/rolldown` | Rust-native JS/CSS bundler |
| zenoh | `~/projects/rust-workspace/zenoh` | Distributed pub/sub + storage (future) |

### CSS and styling

| Dependency | Local Path | Description |
|------------|------------|-------------|
| open-props | `~/projects/lakescope-workspace/open-props` | CSS design tokens library |
| open-props-ui | `~/projects/lakescope-workspace/open-props-ui` | Pure CSS component library (copy-paste model) |

### Reference implementations (alternative approaches)

| Dependency | Local Path | Description |
|------------|------------|-------------|
| maud | `~/projects/rust-workspace/maud` | Alternative HTML templating (eager evaluation) |
| askama | `~/projects/rust-workspace/askama` | Alternative HTML templating (file-based) |
| nats.rs | `~/projects/rust-workspace/nats.rs` | NATS Rust client (for reference, not used) |
| nats-server | `~/projects/lakescope-workspace/nats-server` | NATS server (Go, for reference) |

### Template references

| Template | Local Path | Description |
|----------|------------|-------------|
| rust-nix-template | `~/projects/rust-workspace/rust-nix-template` | Rust + Nix template pattern |
| typescript-nix-template | `~/projects/nix-workspace/typescript-nix-template` | TypeScript + Nix template pattern |
| python-nix-template | `~/projects/nix-workspace/python-nix-template` | Python + Nix template pattern |
| hypertext-typst-nix-youwen5-web | `~/projects/rust-workspace/hypertext-typst-nix-youwen5-web` | Hypertext + Nix pattern (uses Tailwind, not datastar) |

### Integration pattern references

| Pattern | Local Path | Description |
|---------|------------|-------------|
| rust-duckdb-huggingface-ducklake-query | `~/projects/rust-workspace/rust-duckdb-huggingface-ducklake-query` | DuckDB + DuckLake + HuggingFace query pattern (hf:// protocol) |
| marhar-frozen | `~/projects/omicslake-workspace/marhar-frozen` | DuckLake fixture data creation tools |
| marhar-duckdb-tools | `~/projects/omicslake-workspace/marhar-duckdb-tools` | DuckDB tooling for data lake operations |

These patterns enable the axum backend to query remote datasets (HuggingFace Hub, S3-compatible storage) via DuckDB's httpfs extension, serving data for ECharts/Vega visualizations without local data ingestion.
See "Remote data sources via httpfs" in `docs/notes/architecture/architecture-decisions.md` section 6 for implementation details.

### Visualization libraries

| Library | Local Path | Description |
|---------|------------|-------------|
| echarts | `~/projects/lakescope-workspace/echarts` | Apache ECharts (see northstar ds-echarts Lit component for reference) |
| vega-embed | `~/projects/lakescope-workspace/vega-embed` | Vega-Lite chart embedding (wrap in web component) |
| mosaic | `~/projects/lakescope-workspace/mosaic` | Grammar of graphics for large datasets (candidate for integration) |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Ironstar Template                            │
├─────────────────────────────────────────────────────────────────────┤
│  Frontend (Browser)                                                 │
│    datastar.js (signals), Open Props + Open Props UI (CSS), WC      │
├─────────────────────────────────────────────────────────────────────┤
│  Build Pipeline                                                     │
│    Rolldown (bundler), PostCSS (Open Props imports), TypeScript     │
├─────────────────────────────────────────────────────────────────────┤
│  Boundary Layer (Effects)                                           │
│    axum extractors, SSE streams, HTTP request/response              │
├─────────────────────────────────────────────────────────────────────┤
│  Application Layer (Pure Functions)                                 │
│    Command handlers, Query handlers, Event handlers, Projections    │
├─────────────────────────────────────────────────────────────────────┤
│  Domain Layer (Algebraic Types)                                     │
│    Aggregates (sum), Events (sum), Commands (sum), Values (product) │
├─────────────────────────────────────────────────────────────────────┤
│  Infrastructure Layer (Effect Implementations)                      │
│    SQLite/sqlx (events + sessions), DuckDB (analytics), moka (cache)│
├─────────────────────────────────────────────────────────────────────┤
│  Presentation Layer (Lazy Rendering)                                │
│    hypertext (HTML thunks), datastar-rust (SSE generation)          │
└─────────────────────────────────────────────────────────────────────┘
```

## Frontend tooling philosophy

Datastar's core principle is **server-driven UI**: the server sends HTML fragments and signal updates via SSE.
This means most reactivity lives in datastar signals, not client-side frameworks.

**What we use:**

| Tool | Why |
|------|-----|
| Open Props | CSS design tokens, zero build complexity |
| Open Props UI | Pure CSS components, copy-paste ownership model |
| Rolldown | Rust-native bundler (over esbuild which is Go-based) |
| Vanilla Web Components | Thin wrappers when encapsulating third-party libs |
| Lucide | Build-time SVG icons, zero runtime |
| TypeScript | Type safety for the minimal JS we write |

**Modern CSS features:**

Open Props + Open Props UI leverage modern CSS capabilities that require recent browsers:
- **OKLch color space**: Perceptually uniform colors with consistent lightness
- **light-dark() function**: Native theme switching without class toggles
- **Container queries**: Component-level responsive design
- **:has() selector**: Parent-aware styling

**Browser requirements**: Chrome 111+, Firefox 119+, Safari 17+ (all released 2023)

**What we avoid:**

| Tool | Why Not |
|------|---------|
| Lit | Redundant reactivity (datastar already provides this) |
| React/Vue/Svelte | SPA frameworks contradict hypermedia philosophy |
| Leptos/Dioxus | Would duplicate datastar's role |
| esbuild | Go-based; prefer Rust-native Rolldown |

**Web component pattern for datastar:**

```typescript
// Thin wrapper around third-party library
class SortableList extends HTMLElement {
  connectedCallback() {
    Sortable.create(this, {
      onEnd: (evt) => {
        // Dispatch event for datastar to handle
        this.dispatchEvent(new CustomEvent('reorder', { detail: evt }));
      }
    });
  }
}
customElements.define('sortable-list', SortableList);
```

```html
<!-- Datastar handles all state and reactivity -->
<sortable-list
  data-on:reorder="@post('/api/reorder', {body: evt.detail})"
>
</sortable-list>
```

The component is a thin wrapper. All state flows through datastar signals.

## Vega-Lite integration pattern

Vega-Lite charts via vega-embed require special handling because Vega manages its own DOM.

**Key requirements:**

- Use `data-ignore-morph` on the container (prevents datastar from morphing Vega's DOM)
- Store the Vega `View` instance for reactive updates
- Updates happen via Vega's View API, not re-embedding
- Call `finalize()` on disconnect to prevent memory leaks

**Web component pattern:**

```typescript
// web-components/components/vega-chart.ts
import embed, { Result } from 'vega-embed';
import { View } from 'vega';

class VegaChart extends HTMLElement {
  private result: Result | null = null;
  private view: View | null = null;

  static observedAttributes = ['spec-url', 'data-url', 'signal-values'];

  async connectedCallback() {
    await this.render();
  }

  disconnectedCallback() {
    this.result?.finalize(); // Critical: prevent memory leaks
  }

  async attributeChangedCallback(name: string, oldValue: string, newValue: string) {
    if (!this.view || oldValue === newValue) return;

    switch (name) {
      case 'data-url':
        const data = await fetch(newValue).then(r => r.json());
        this.view.data('source', data).run();
        break;
      case 'signal-values':
        const signals = JSON.parse(newValue);
        Object.entries(signals).forEach(([k, v]) => this.view!.signal(k, v));
        this.view.run();
        break;
      case 'spec-url':
        await this.render(); // Full re-render for spec changes
        break;
    }
  }

  private async render() {
    const specUrl = this.getAttribute('spec-url');
    if (!specUrl) return;

    this.result?.finalize();
    const spec = await fetch(specUrl).then(r => r.json());
    this.result = await embed(this, spec, { renderer: 'svg', actions: false });
    this.view = this.result.view;

    // Bridge Vega selections to datastar custom events
    this.view.addSignalListener('select', (name, value) => {
      this.dispatchEvent(new CustomEvent('vega-select', {
        detail: { name, value }, bubbles: true
      }));
    });
  }
}
customElements.define('vega-chart', VegaChart);
```

**Datastar usage:**

```html
<vega-chart
  data-ignore-morph
  data-attr:spec-url="$chartSpec"
  data-attr:data-url="$dataEndpoint"
  data-on:vega-select="$selection = evt.detail.value"
></vega-chart>
```

**Mosaic consideration:**

Mosaic (`~/projects/lakescope-workspace/mosaic`) provides a higher-level grammar of graphics optimized for large datasets with coordinated views.
Integration pattern TBD — may benefit from similar web component wrapper or direct integration depending on how it manages state.

## Event sourcing model

Ironstar uses event sourcing with CQRS separation:

**Write side (commands):**

```
Command → Validate → Emit Events → Append to SQLite → Publish to broadcast
```

**Read side (queries):**

```
Subscribe to broadcast → Update projections → Serve via SSE
```

**Event store schema:**

```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    payload JSON NOT NULL,
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(aggregate_type, aggregate_id, sequence)
);
```

## Architectural context: embedded vs. external services

This stack prioritizes embedded Rust-native solutions over external server dependencies.
NATS is an excellent choice for teams willing to run an external server — it provides streaming, key-value storage, and pub/sub in a unified abstraction.

For Ironstar, the embedded approach (SQLite + tokio broadcast + moka) was chosen because the template targets single-node deployments where a separate server is unnecessary.
The [Jepsen analysis of NATS 2.12.1](https://jepsen.io/analyses/nats-2.12.1) also reinforced confidence in SQLite's durability model, though NATS can be configured appropriately for many use cases.

Sessions are stored in SQLite alongside the event store (in a separate table), simplifying the stack by using a single embedded database for all persistent state.
Analytics query results are cached in moka, an async-native in-memory cache with TTL-based eviction.
Cache entries are invalidated via subscription to the tokio broadcast channel when relevant events arrive.
See `docs/notes/architecture/analytics-cache-architecture.md` for detailed cache design.

When distribution is needed, Zenoh provides Rust-native pub/sub with storage backends.

**CQRS/ES framework decision:**

After evaluating cqrs-es, sqlite-es, and esrs (Prima.it's event_sourcing.rs), the decision is to implement a custom CQRS layer rather than adopt these frameworks as dependencies.
The rationale:
- cqrs-es adds abstraction overhead that may conflict with hypertext lazy rendering and datastar SSE integration
- esrs is PostgreSQL-only with no SQLite backend
- sqlite-es is a thin adapter; the patterns are more valuable than the library itself
- Rust's type system enforces CQRS discipline without framework magic

Key patterns adopted from these references:
- Pure synchronous aggregates (esrs): keeps side effects at boundaries, improves testability
- Schema/Upcaster for event evolution (esrs): enables backward-compatible schema changes
- TestFramework DSL (cqrs-es): elegant given/when/then testing for aggregates
- Event store schema (sqlite-es): compound primary key, JSON payload, optimistic locking

## Build commands

*To be implemented with nix flake.*

```bash
# Development
nix develop                    # Enter dev shell
just dev                       # Run with hot reload

# Build
nix build                      # Build release binary

# Test
cargo test                     # Run tests
just test                      # Run all tests including integration
```

## Project structure (planned)

```
ironstar/
├── CLAUDE.md                           # This file
├── flake.nix                           # Nix flake definition
├── flake.lock
├── Cargo.toml
├── Cargo.lock
├── justfile                            # Task runner
├── process-compose.yaml                # Process orchestration
├── docs/
│   └── notes/
│       └── architecture/
│           ├── design-principles.md
│           └── architecture-decisions.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── domain/                         # Algebraic types
│   │   ├── mod.rs
│   │   ├── aggregates/
│   │   ├── events/
│   │   ├── commands/
│   │   ├── values/
│   │   └── signals.rs                  # Datastar signal types (derive TS for TypeScript)
│   ├── application/                    # Pure business logic
│   │   ├── mod.rs
│   │   ├── command_handlers.rs
│   │   ├── query_handlers.rs
│   │   └── projections.rs
│   ├── infrastructure/                 # Effect implementations
│   │   ├── mod.rs
│   │   ├── event_store.rs              # SQLite events
│   │   ├── session_store.rs            # SQLite sessions
│   │   ├── analytics.rs                # DuckDB queries
│   │   ├── analytics_cache.rs          # moka cache with rkyv serialization
│   │   └── event_bus.rs                # tokio broadcast
│   └── presentation/                   # HTTP + HTML
│       ├── mod.rs
│       ├── routes.rs
│       ├── handlers/
│       │   ├── mod.rs
│       │   ├── sse.rs
│       │   └── commands.rs
│       └── templates/                  # hypertext components
├── web-components/                     # Frontend assets (separate build)
│   ├── package.json
│   ├── pnpm-lock.yaml
│   ├── rolldown.config.ts              # Bundler config
│   ├── tsconfig.json
│   ├── index.ts                        # CSS entry point
│   ├── icons.ts                        # Lucide icons (build-time)
│   ├── types/                          # Generated TypeScript types (from ts-rs)
│   │   └── *.ts                        # Auto-generated, do not edit
│   ├── components/                     # Vanilla web components
│   │   ├── sortable-list.ts
│   │   └── vega-chart.ts               # Vega-Lite chart wrapper
│   └── styles/
│       ├── main.css                    # Open Props imports + theme + components
│       ├── theme.css                   # Custom theme tokens
│       └── components/                 # Copied Open Props UI CSS
│           ├── button.css
│           ├── card.css
│           └── *.css                   # Other component styles
├── static/
│   ├── dist/                           # Built assets (from web-components)
│   │   ├── bundle.css
│   │   └── components.js
│   └── datastar/
│       └── datastar.js                 # Datastar runtime
└── examples/
    └── todo/                           # Todo app demo (like northstar)
```

## Related documentation

### Ironstar architecture docs

- Design principles: `docs/notes/architecture/design-principles.md`
- Architecture decisions: `docs/notes/architecture/architecture-decisions.md`
  - Open Props design tokens rationale
  - Open Props UI component library rationale
  - Asset embedding (rust-embed)
  - Module organization and scaling path
- Development workflow: `docs/notes/architecture/development-workflow.md`
  - Process orchestration (process-compose)
  - Hot reload pattern
  - Asset serving modes (dev/prod)
  - TypeScript type generation (ts-rs)
- Event sourcing + SSE pipeline: `docs/notes/architecture/event-sourcing-sse-pipeline.md`
- Third-party library integration: `docs/notes/architecture/integration-patterns.md`
  - Pattern 1: Web component thin wrapper (vanilla)
  - Pattern 1.5: When Lit is appropriate (complex lifecycle)
  - Pattern 2: Vega-Lite chart integration
  - Pattern 3: Apache ECharts integration via Lit (ds-echarts component)
- TypeScript signal contracts: `docs/notes/architecture/signal-contracts.md`
- Frontend build pipeline: `docs/notes/architecture/frontend-build-pipeline.md`
  - Includes Lit component bundling options (Rolldown vs esbuild)

### External references

- Datastar SDK specification: `~/projects/lakescope-workspace/datastar/sdk/ADR.md`
- Datastar documentation: `~/projects/lakescope-workspace/datastar-doc/`
- Tao of Datastar: `~/projects/lakescope-workspace/datastar-doc/guide_the_tao_of_datastar.md`
- Northstar (Go template): `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/`
- Open Props design tokens: `~/projects/lakescope-workspace/open-props/`
- Open Props UI components: `~/projects/lakescope-workspace/open-props-ui/`
- vega-embed API: `~/projects/lakescope-workspace/vega-embed/src/embed.ts`
- Mosaic documentation: `~/projects/lakescope-workspace/mosaic/docs/`
