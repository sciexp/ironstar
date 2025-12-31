# Session management

> **Semantic foundation**: Sessions implement indexed profunctor filtering.
> Session-scoped Zenoh subscriptions partition event delivery without modifying the global event log's free monoid structure.
> See [semantic-model.md § Sessions as indexed profunctor](../core/semantic-model.md#sessions-as-indexed-profunctor).

Session management for Ironstar, enabling per-user SSE subscriptions and state isolation in a Datastar-driven hypermedia application.

## Design principles

Sessions serve three critical purposes in Ironstar's architecture:

1. **Per-user event filtering**: Zenoh subscriptions use session IDs in key expressions to route events only to the correct SSE connections, avoiding broadcast-then-filter inefficiency.
2. **State isolation**: Each session maintains independent application state (e.g., TodoMVC data), enabling multiple concurrent users without interference.
3. **Reconnection resilience**: Session IDs in cookies allow SSE reconnections to resume the same logical session, preserving user context across network interruptions.

Unlike traditional session stores that hold authentication state, Ironstar sessions are lightweight identifiers for pub/sub scoping.
They exist in a single SQLite table alongside the event store, simplifying deployment without external session services.

## Session identification

### Session ID generation

Session IDs must be cryptographically secure to prevent enumeration attacks.
Use 192 bits (24 bytes) of entropy from a CSPRNG, encoded as URL-safe base64 without padding.

```rust
use rand::Rng;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

pub fn generate_session_id() -> String {
    let mut bytes = [0u8; 24]; // 192 bits of entropy
    rand::thread_rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}
```

**Rationale for 192 bits**: At 1 million sessions created per second, the probability of a collision remains negligible (< 2^-64) over the lifetime of the universe.
This eliminates the need for uniqueness checks in the database.

### Cookie configuration

Session cookies must use security flags to prevent common web attacks.

```rust
use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;

pub fn create_session_cookie(session_id: &str) -> Cookie<'static> {
    Cookie::build(("ironstar_session", session_id.to_owned()))
        .path("/")
        .http_only(true)        // Prevent XSS access to session ID
        .secure(true)           // Require HTTPS in production
        .same_site(SameSite::Lax) // CSRF protection with SSE compatibility
        .max_age(Duration::days(30))
        .build()
}

pub fn delete_session_cookie() -> Cookie<'static> {
    Cookie::build(("ironstar_session", ""))
        .path("/")
        .max_age(Duration::ZERO)
        .build()
}
```

**Security flags explained**:

- `http_only`: Prevents JavaScript access via `document.cookie`, mitigating XSS attacks.
- `secure`: Ensures transmission only over HTTPS (disable in dev mode).
- `same_site = Lax`: Allows cookies on top-level navigation (e.g., clicking links) but blocks third-party POST requests. SSE connections initiated by client-side JS are same-site, so `Lax` suffices.

**Why not `SameSite::Strict`?** Strict mode would block cookies when users navigate to the site from external links, breaking the initial SSE connection.

## Session storage

### SQLite schema

Sessions live in a dedicated table alongside the event store, avoiding external dependencies.

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT,                    -- Optional: link to authenticated user (NULL for anonymous)
    created_at TEXT DEFAULT (datetime('now')),
    last_seen_at TEXT DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    data TEXT DEFAULT '{}'           -- Flexible session-scoped state storage (JSON serialized)
) STRICT;

CREATE INDEX idx_sessions_expires ON sessions(expires_at);
CREATE INDEX idx_sessions_user ON sessions(user_id) WHERE user_id IS NOT NULL;
```

**Schema notes**:

- `id`: Session ID (primary key), generated via `generate_session_id()`.
- `user_id`: Optional foreign key to a users table. NULL for anonymous sessions.
- `created_at`: TEXT column storing ISO 8601 timestamp via `datetime('now')` (STRICT mode requirement).
- `last_seen_at`: Updated on every request for activity tracking and idle timeout.
- `expires_at`: Absolute expiration timestamp (e.g., `created_at + 30 days`).
- `data`: TEXT column storing JSON-serialized session state (STRICT mode does not support JSON type).

**Why TEXT for timestamps?** SQLite STRICT mode requires explicit type affinity. TEXT with `datetime('now')` provides ISO 8601 timestamps compatible with chrono::DateTime parsing.

**Why TEXT for JSON?** STRICT mode does not support the JSON type. Store JSON as TEXT and deserialize with `serde_json::from_str()` in Rust.

### Migration file naming convention

Database schema changes are managed via numbered migration files following the pattern `NNN_description.sql` where NNN is a zero-padded sequential number and description uses lowercase snake_case.

**Migration directory structure:**

```
migrations/
├── 001_create_events.sql
├── 002_create_sessions.sql
├── 003_create_users.sql
├── 004_create_user_identities.sql
└── ...
```

**Naming rules:**

- Three-digit zero-padded sequence number (001, 002, ..., 999)
- Underscore separator between number and description
- Description in lowercase with underscores (snake_case)
- `.sql` extension

**Sequential numbering ensures:**

- Deterministic application order (migrations run in lexicographic order)
- Easy identification of the latest schema version
- Clear dependency relationships (migration N can assume migrations 1..N-1 have run)

**sqlx migration management:**

Ironstar uses sqlx's built-in migration support for applying and tracking migrations.

```rust
use sqlx::migrate::MigrateDatabase;
use sqlx::SqlitePool;

// Apply all pending migrations on startup
let pool = SqlitePool::connect(&database_url).await?;
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

The `sqlx::migrate!` macro embeds migration files at compile time and tracks applied migrations in a `_sqlx_migrations` table.
This ensures idempotent schema initialization across deployments.

**Migration best practices:**

- Each migration file contains exactly one logical schema change
- Use `IF NOT EXISTS` for tables and indexes (supports local development re-runs)
- Never modify existing migration files after deployment (create new migrations for schema changes)
- Test migrations against empty database and populated database to verify both initial setup and incremental updates

## Related documentation

- **Rust implementation patterns**: `session-implementation.md` — Session service CRUD, axum extractors, SSE handler integration
- **Security and operations**: `session-security.md` — Zenoh key expressions, rate limiting, cleanup tasks, security mitigations
- **SSE connection lifecycle**: `../cqrs/sse-connection-lifecycle.md` — Client subscription, reconnection resilience, Last-Event-ID
- **Zenoh event bus**: `zenoh-event-bus.md` — Key expression patterns, embedded configuration

## References

- [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- Northstar session pattern: `/Users/crs58/projects/lakescope-workspace/datastar-go-nats-template-northstar/features/index/services/todo_service.go` (lines 165-182)
- Northstar SSE subscription: `/Users/crs58/projects/lakescope-workspace/datastar-go-nats-template-northstar/features/index/handlers.go` (lines 32-71)
- axum cookie handling: `axum-extra` crate documentation
