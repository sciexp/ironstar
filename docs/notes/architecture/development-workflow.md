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

### Frontend asset build

The frontend process runs Rolldown in watch mode, rebuilding on TypeScript/CSS changes.
It depends on typegen completing first to ensure signal types are available.

```yaml
# process-compose.yaml addition
processes:
  frontend:
    command: pnpm --dir web-components dev
    availability:
      restart: on_failure
    depends_on:
      typegen:
        condition: process_completed_successfully
```

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
The browser maintains an SSE connection to `/hotreload` that triggers page refresh when the server signals a rebuild completion.

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
    backend = {
      command = lib.getExe' pkgs.cargo-watch "cargo-watch" + " -x run";
      depends_on."db-init".condition = "process_completed_successfully";
      readiness_probe.http_get = {
        host = "localhost";
        port = 3000;
        path = "/health";
      };
    };
    # ... other processes
  };
};

packages.dev = config.process-compose."dev".package;
```

Run with:

```bash
nix run .#dev
```

---

## Database schema management

Ironstar uses SQLite for the event store with a simplified schema management workflow optimized for single-developer environments.

### Schema location

Database schema files live in `migrations/schema.sql` at the repository root:

```sql
-- migrations/schema.sql
CREATE TABLE IF NOT EXISTS events (
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

CREATE INDEX IF NOT EXISTS idx_events_aggregate
    ON events(aggregate_type, aggregate_id, sequence);
```

### Schema initialization

The `db-init` process in process-compose.yaml applies the schema on startup:

```yaml
db-init:
  command: |
    sqlite3 dev.db < migrations/schema.sql
  disable_on_success: true
```

This ensures the database exists and has the correct schema before other processes start.

### Development workflow

For schema changes during development:

1. Edit `migrations/schema.sql` with new DDL
2. Restart process-compose (or run `db-init` manually):

```bash
sqlite3 dev.db < migrations/schema.sql
```

The `IF NOT EXISTS` and `IF NOT EXISTS` clauses make schema application idempotent.

### Migration tooling consideration

This simplified approach works well for single-developer workflows where the database can be recreated from events.
For team environments or production systems requiring schema versioning, consider adding sqlx migrations:

```bash
# Future migration workflow (not implemented yet)
sqlx migrate add create_events_table
sqlx migrate run
```

The current pattern prioritizes development velocity over migration history.
Event sourcing provides data durability independent of schema versions.

---

## Code quality commands

Ironstar uses standard Rust and TypeScript tooling for formatting, linting, and type checking.

### justfile targets

Add these commands to `justfile`:

```justfile
# Format code
fmt:
    cargo fmt
    cd web-components && pnpm format

# Lint
lint:
    cargo clippy -- -D warnings
    cd web-components && pnpm lint

# Type check (no compilation)
check:
    cargo check
    cd web-components && pnpm typecheck

# All quality checks
ci: fmt lint check test
```

### Pre-commit workflow

Run before committing:

```bash
just fmt    # Auto-format Rust and TypeScript
just lint   # Check for common issues
just check  # Verify types without building
```

### CI integration

The `ci` target runs all checks in sequence, suitable for pre-push hooks or GitHub Actions:

```bash
just ci
```

Failures in any step will halt the pipeline.

### Tool configuration

Rust tools use workspace defaults:

- `cargo fmt`: Follows `rustfmt.toml` (if present)
- `cargo clippy`: Uses pedantic lints defined in `Cargo.toml`:

```toml
[workspace.lints.clippy]
pedantic = "warn"
```

TypeScript tools use `web-components/` configuration:

- `pnpm format`: Runs Prettier via `package.json` script
- `pnpm lint`: Runs ESLint with TypeScript rules
- `pnpm typecheck`: Invokes `tsc --noEmit`

---

## Environment configuration

Ironstar uses environment variables for development configuration.
Production deployments use Nix-managed secrets (see `~/.claude/commands/preferences/secrets.md`).

### Development environment file

Create `.env.development` at the repository root:

```bash
# .env.development
DATABASE_URL=dev.db
LOG_LEVEL=debug
SERVER_PORT=3000
RELOAD_ENABLED=true

# Asset paths (development only)
STATIC_DIR=static/dist
FRONTEND_DEV_URL=http://localhost:5173
```

### Loading environment variables

process-compose auto-loads `.env` from the working directory by default (disable with `is_dotenv_disabled: true`).
For development, either rename `.env.development` to `.env` or use `env_cmds` for dynamic generation.

```yaml
processes:
  backend:
    command: cargo watch -x run
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - LOG_LEVEL=${LOG_LEVEL}
    # Or use env_cmds for dynamic env generation:
    # env_cmds:
    #   - "cat .env.development"
```

The `environment` field accepts a list of `KEY=VALUE` strings, not file paths.
Variables are interpolated at process startup from the auto-loaded `.env` file.

### Git ignore pattern

Never commit secrets to version control.
Ensure `.gitignore` contains:

```gitignore
# Environment files
.env
.env.development
.env.production
*.env.local

# Development database
dev.db
dev.db-shm
dev.db-wal
```

### Environment variable precedence

Variables are resolved in this order (highest precedence first):

1. Process-specific `environment` map in process-compose.yaml
2. Variables from loaded `.env.development` file
3. Shell environment variables

This allows per-process overrides while sharing common configuration.

### Production secrets

Production deployments should never use `.env` files.
Use Nix-managed secrets with sops-nix or similar:

```nix
# flake.nix (production example, not implemented)
{
  services.ironstar = {
    enable = true;
    secrets = {
      DATABASE_URL = config.sops.secrets.database-url.path;
    };
  };
}
```

See `~/.claude/commands/preferences/secrets.md` for detailed secrets management patterns.

---

## Related documentation

- Process orchestration details: `process-compose.yaml` (repository root)
- Asset embedding decision: `docs/notes/architecture/architecture-decisions.md`
- Signal type contracts: `docs/notes/architecture/signal-contracts.md`
- Frontend build pipeline: `docs/notes/architecture/frontend-build-pipeline.md`
