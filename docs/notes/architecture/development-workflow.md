# Development workflow

This document covers ironstar's development workflow including process orchestration, hot reload patterns, and build commands.

## Process orchestration

Ironstar uses process-compose to coordinate multiple development processes.
The configuration lives in `process-compose.yaml` at the repository root.

### Process dependency graph

```
                    ┌─────────────┐
                    │   db-init   │
                    │  (one-shot) │
                    └──────┬──────┘
                           │ process_completed_successfully
                           ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  frontend   │     │   backend   │     │   typegen   │
│  (watcher)  │────▶│   (server)  │     │  (watcher)  │
└─────────────┘     └──────┬──────┘     └─────────────┘
   process_started         │ process_healthy
                           ▼
                    ┌─────────────┐
                    │  hotreload  │
                    │  (watcher)  │
                    └─────────────┘
```

### Process descriptions

| Process | Command | Purpose |
|---------|---------|---------|
| `db-init` | `sqlite3` schema creation | One-shot database initialization |
| `frontend` | `pnpm dev` (Rolldown watch) | Rebuild CSS/JS on file changes |
| `typegen` | `cargo watch` + `cargo test` | Regenerate TypeScript types from Rust signals |
| `backend` | `cargo watch -x run` | Rebuild and restart server on Rust changes |
| `hotreload` | `cargo watch` + curl | Trigger browser reload when builds complete |
| `test` | `curl /health` | Integration test for CI (creates flake check) |

### Running development mode

With Nix (preferred for reproducibility):

```bash
nix develop
nix run .#dev
```

Without Nix (requires process-compose in PATH):

```bash
process-compose up
```

Individual processes can be run in isolation:

```bash
process-compose up backend frontend  # Run only these two
process-compose up -d                # Detached mode
process-compose logs backend         # View specific process logs
```

---

## Hot reload pattern

Ironstar implements server-driven hot reload following the northstar pattern.
The browser maintains an SSE connection to `/dev/reload` that triggers page refresh when the server signals a rebuild completion.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Browser                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ <div data-init="@get('/dev/reload',                   │  │
│  │      {retryMaxCount: 1000, retryInterval: 20})">      │  │
│  │ </div>                                                │  │
│  └───────────────────────────────────────────────────────┘  │
│            │ SSE connection (long-lived)                    │
└────────────┼────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────┐
│                    Axum Server                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ GET /hotreload                                        │    │
│  │   - Blocks on reload channel                         │    │
│  │   - On signal: ExecuteScript("window.location.reload()") │
│  │   - Client reconnects automatically via Datastar      │    │
│  └─────────────────────────────────────────────────────┘    │
│                          ▲                                   │
│                          │ broadcast::Sender                 │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ POST /hotreload/trigger                              │    │
│  │   - Sends signal to reload channel                   │    │
│  │   - Called by hotreload process on build complete    │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Implementation

The hot reload endpoint is conditionally compiled (development only):

```rust
#[cfg(debug_assertions)]
mod dev {
    use axum::{
        extract::State,
        response::sse::{Event, Sse},
        routing::{get, post},
        Router,
    };
    use datastar::prelude::ExecuteScript;
    use std::convert::Infallible;
    use tokio::sync::broadcast;
    use tokio_stream::{wrappers::BroadcastStream, StreamExt};

    pub fn routes(reload_tx: broadcast::Sender<()>) -> Router {
        Router::new()
            .route("/hotreload", get(reload_sse))
            .route("/hotreload/trigger", post(trigger_reload))
            .with_state(reload_tx)
    }

    async fn reload_sse(
        State(reload_tx): State<broadcast::Sender<()>>,
    ) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
        let rx = reload_tx.subscribe();
        let stream = BroadcastStream::new(rx).filter_map(|_| async {
            Some(Ok(ExecuteScript::new("window.location.reload()")
                .write_as_axum_sse_event()))
        });
        Sse::new(stream)
    }

    async fn trigger_reload(State(reload_tx): State<broadcast::Sender<()>>) {
        let _ = reload_tx.send(());
    }
}
```

The base template conditionally includes the reload hook:

```rust
fn base_layout(content: impl Renderable) -> impl Renderable {
    maud! {
        html {
            head { /* ... */ }
            body {
                @if cfg!(debug_assertions) {
                    div "data-init"="@get('/hotreload', {retryMaxCount: 1000, retryInterval: 20})" {}
                }
                (content)
            }
        }
    }
}
```

### Process coordination

The `hotreload` process in process-compose watches for build artifacts and triggers reload:

```yaml
hotreload:
  command: |
    cargo watch -w src -w static/dist -s 'curl -s http://localhost:3000/hotreload/trigger || true'
  depends_on:
    backend:
      condition: process_healthy
```

This creates the feedback loop:
1. Developer edits Rust or frontend source
2. `cargo watch` or Rolldown rebuilds artifacts
3. `hotreload` process detects change and curls trigger endpoint
4. Server broadcasts to all connected SSE clients
5. Browsers execute `window.location.reload()`

---

## Asset serving modes

Ironstar serves static assets differently in development and production.

### Development mode

Assets are served directly from the filesystem with no-cache headers:

```rust
#[cfg(debug_assertions)]
fn static_routes() -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("static/dist"))
        .layer(SetResponseHeaderLayer::if_not_present(
            CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
}
```

Benefits:
- Instant refresh on file changes
- No compilation overhead for asset updates
- Rolldown watch mode output is immediately available

### Production mode

Assets are embedded in the binary with immutable cache headers:

```rust
#[cfg(not(debug_assertions))]
fn static_routes() -> Router {
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "static/dist"]
    struct Assets;

    // Serve embedded assets with long-lived cache
    Router::new()
        .nest_service("/static", /* embedded asset handler */)
        .layer(SetResponseHeaderLayer::overriding(
            CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        ))
}
```

Content-hashed filenames from Rolldown enable infinite caching.

See `architecture-decisions.md` for detailed asset embedding rationale.

---

## TypeScript type generation

Signal types defined in Rust are exported to TypeScript via ts-rs.

### Workflow

1. Define signal types with `#[derive(TS)]`:

```rust
// src/domain/signals.rs
#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "web-components/types/")]
pub struct TodoSignals {
    pub input: Option<String>,
    pub filter: TodoFilter,
}
```

2. The `typegen` process watches for changes and regenerates types:

```yaml
typegen:
  command: |
    cargo watch -w src/domain/signals.rs -s 'cargo test --lib export_bindings'
  environment:
    TS_RS_EXPORT_DIR: web-components/types
```

3. Generated TypeScript appears in `web-components/types/`:

```typescript
// Auto-generated by ts-rs
export type TodoSignals = {
  input?: string;
  filter: TodoFilter;
};
```

4. Frontend code imports the types for compile-time safety:

```typescript
import type { TodoSignals } from '@types/TodoSignals';
```

### Manual generation

Run type generation manually:

```bash
TS_RS_EXPORT_DIR=web-components/types cargo test --lib
```

---

## Build commands

### justfile integration

```justfile
# Start full development environment
dev:
    process-compose up

# Run development environment in background
dev-bg:
    process-compose up -d

# Generate TypeScript types from Rust
gen-types:
    TS_RS_EXPORT_DIR=web-components/types cargo test --lib

# Build frontend assets (production)
build-frontend:
    cd web-components && pnpm build

# Build backend (production)
build-backend:
    cargo build --release

# Full production build
build: gen-types build-frontend build-backend

# Run tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean
    rm -rf static/dist/
    rm -rf web-components/dist/
```

### Nix flake integration

The flake exposes process-compose as a runnable package:

```nix
# In perSystem
process-compose."dev" = {
  settings.processes = {
    # Process definitions imported from process-compose.yaml
    # or defined inline using lib.getExe for reproducible paths
  };
};

packages.dev = config.process-compose."dev".package;
```

Run with:

```bash
nix run .#dev
```

---

## Related documentation

- Process orchestration details: `process-compose.yaml` (repository root)
- Asset embedding decision: `docs/notes/architecture/architecture-decisions.md`
- Signal type contracts: `docs/notes/architecture/signal-contracts.md`
- Frontend build pipeline: `docs/notes/architecture/frontend-build-pipeline.md`
