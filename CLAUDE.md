# Ironstar

Ironstar is a Rust + Datastar template for building reactive, event-sourced web applications with hypermedia-driven architecture.
To a first approximation, it is the Rust equivalent of [northstar](https://github.com/zangster300/northstar), whose source code we have a local copy of (see "Datastar ecosystem" table below), adapted for Rust's type system, functional programming idioms, and broader integration of ecosystem components to improve the ability of the template to illustrate how to scale applications.
For now we will not build a documentation site and keep documentation work in the docs/ subtree. Any documents that are not eventually intended to be
retained in production quality docs for users or developers should go in an aptly named subfolder within the docs/notes/ subtree.

## Project status

Early design phase.
Component selection complete, implementation not yet started.

## Design philosophy

Effects explicit in type signatures, isolated at boundaries to preserve compositionality.
The stack embodies algebraic data types (sum types for states, product types for data), lawful abstractions, and referential transparency where possible.

See `docs/notes/architecture/stack-component-selection.md` for detailed component rationale.

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
| HTML Templating | hypertext | Lazy monoid (thunks) |
| Frontend Reactivity | datastar-rust | FRP signals via SSE |
| CSS Framework | Tailwind v4 + DaisyUI | Utility classes + component combinators |
| CSS/JS Build | Rolldown + PostCSS | Rust-native bundler |
| Icons | Lucide (build-time) | Zero-runtime SVG inlining |
| Web Components | Vanilla (when needed) | Thin wrappers for third-party libs |
| Event Store | SQLite + sqlx | Append-only event log |
| Session KV | redb | ACID embedded KV |
| Analytics | DuckDB | OLAP projections |
| Event Bus | tokio::sync::broadcast | In-process pub/sub |
| Distribution (future) | Zenoh | Distributed pub/sub + storage |
| Orchestration | process-compose | Declarative process management |
| Environment | Nix flake | Reproducible builds |

## Local dependency paths

All dependencies with local source code available for reference.

### Core Rust dependencies

| Dependency | Local Path | Description |
|------------|------------|-------------|
| axum | `~/projects/rust-workspace/axum` | Web framework with SSE support |
| tokio | `~/projects/rust-workspace/tokio` | Async runtime |
| hypertext | `~/projects/rust-workspace/hypertext` | Lazy HTML templating (maud-compatible syntax) |
| sqlx | `~/projects/rust-workspace/sqlx` | Async SQL with compile-time validation |
| redb | `~/projects/rust-workspace/redb` | Embedded ACID KV store |
| duckdb-rs | `~/projects/omicslake-workspace/duckdb-rs` | DuckDB Rust bindings |
| ts-rs | `~/projects/rust-workspace/ts-rs` | TypeScript type generation from Rust structs |

### Datastar ecosystem

| Dependency | Local Path | Description |
|------------|------------|-------------|
| datastar | `~/projects/lakescope-workspace/datastar` | Main datastar repository (frontend JS + SDK specs) |
| datastar-doc | `~/projects/lakescope-workspace/datastar-doc` | Local markdown copy of datastar documentation |
| datastar-rust | `~/projects/rust-workspace/datastar-rust` | Rust SDK for SSE generation |
| datastar-rust-lince | `~/projects/rust-workspace/datastar-rust-lince` | Real-world usage example |
| datastar-go | `~/projects/lakescope-workspace/datastar-go` | Go SDK (reference implementation) |
| northstar | `~/projects/lakescope-workspace/datastar-go-nats-template-northstar` | Go template we're porting from |

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
| tailwindcss | `~/projects/lakescope-workspace/tailwindcss` | CSS framework |

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
| hypertext-typst-nix-youwen5-web | `~/projects/rust-workspace/hypertext-typst-nix-youwen5-web` | Hypertext + Tailwind + Nix pattern (no datastar) |

### Integration pattern references

| Pattern | Local Path | Description |
|---------|------------|-------------|
| rust-duckdb-huggingface-ducklake-query | `~/projects/rust-workspace/rust-duckdb-huggingface-ducklake-query` | DuckDB + DuckLake + HuggingFace query pattern |

### Visualization libraries

| Library | Local Path | Description |
|---------|------------|-------------|
| vega-embed | `~/projects/lakescope-workspace/vega-embed` | Vega-Lite chart embedding (wrap in web component) |
| mosaic | `~/projects/lakescope-workspace/mosaic` | Grammar of graphics for large datasets (candidate for integration) |

### Missing dependencies (need to clone)

| Dependency | Clone URL | Description |
|------------|-----------|-------------|
| zenoh | `https://github.com/eclipse-zenoh/zenoh` | Future distributed pub/sub + storage |
| rolldown | `https://github.com/rolldown/rolldown` | Rust-native JS/CSS bundler |

Clone with:

```bash
cd ~/projects/rust-workspace
git clone https://github.com/eclipse-zenoh/zenoh.git
git clone https://github.com/rolldown/rolldown.git
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Ironstar Template                            │
├─────────────────────────────────────────────────────────────────────┤
│  Frontend (Browser)                                                 │
│    datastar.js (signals), Tailwind + DaisyUI (CSS), Vanilla WC      │
├─────────────────────────────────────────────────────────────────────┤
│  Build Pipeline                                                     │
│    Rolldown (bundler), PostCSS (Tailwind), TypeScript (WC only)     │
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
│    SQLite/sqlx (events), redb (sessions), DuckDB (analytics)        │
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
| DaisyUI | Pure CSS components, zero runtime, Tailwind plugin |
| Rolldown | Rust-native bundler (over esbuild which is Go-based) |
| Vanilla Web Components | Thin wrappers when encapsulating third-party libs |
| Lucide | Build-time SVG icons, zero runtime |
| TypeScript | Type safety for the minimal JS we write |

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

For Ironstar, the embedded approach (SQLite + tokio broadcast + redb) was chosen because the template targets single-node deployments where a separate server is unnecessary.
The [Jepsen analysis of NATS 2.12.1](https://jepsen.io/analyses/nats-2.12.1) also reinforced confidence in SQLite's durability model, though NATS can be configured appropriately for many use cases.

When distribution is needed, Zenoh provides Rust-native pub/sub with storage backends.

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
│           └── stack-component-selection.md
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
│   │   ├── session_store.rs            # redb sessions
│   │   ├── analytics.rs                # DuckDB queries
│   │   └── event_bus.rs                # tokio broadcast
│   └── presentation/                   # HTTP + HTML
│       ├── mod.rs
│       ├── routes.rs
│       ├── handlers.rs
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
│       ├── main.css                    # Tailwind import + @source
│       └── daisyui-theme.js            # Custom theme
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

- Component selection rationale: `docs/notes/architecture/stack-component-selection.md`
- Event sourcing + SSE pipeline: `docs/notes/architecture/event-sourcing-sse-pipeline.md`
- Third-party library integration: `docs/notes/architecture/integration-patterns.md`
- TypeScript signal contracts: `docs/notes/architecture/signal-contracts.md`
- Frontend build pipeline: `docs/notes/architecture/frontend-build-pipeline.md`

### External references

- Datastar SDK specification: `~/projects/lakescope-workspace/datastar/sdk/ADR.md`
- Datastar documentation: `~/projects/lakescope-workspace/datastar-doc/`
- Tao of Datastar: `~/projects/lakescope-workspace/datastar-doc/guide_the_tao_of_datastar.md`
- Northstar (Go template): `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/`
- Hypertext + Tailwind + Nix patterns: `~/projects/rust-workspace/hypertext-typst-nix-youwen5-web/`
- redb design: `~/projects/rust-workspace/redb/docs/design.md`
- vega-embed API: `~/projects/lakescope-workspace/vega-embed/src/embed.ts`
- Mosaic documentation: `~/projects/lakescope-workspace/mosaic/docs/`
