# Infrastructure architecture decisions

This document records infrastructure technology selection decisions for the ironstar stack, covering event bus, session management, caching, and asset handling.
For frontend, backend core, and CQRS implementation decisions, see the related documentation section below.

## tokio::sync::broadcast — event bus as observable

**Algebraic justification:**

The broadcast channel implements the *Observer pattern* as a pure data flow:

```rust
use tokio::sync::{broadcast, mpsc};
use tokio::sync::broadcast::error::SendError;

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

**In-process event notification:**

NATS KV Watch could accomplish the same goal with an external server.
tokio broadcast provides an embedded alternative:

- In-process, deterministic, and composable
- No network effects in the notification path
- No additional server to deploy

**Why tokio::sync::broadcast over NATS:**

Northstar (the Datastar Go template) uses embedded NATS, but analysis revealed it doesn't do true event sourcing — it uses NATS KV as a document store with last-write-wins semantics.
For ironstar's single-node deployment target, tokio::sync::broadcast provides sufficient pub/sub without external server dependency.

**Single-node scaling limits:**

tokio::broadcast is the current choice and is sufficient for single-node deployments up to:
- ~256 concurrent SSE clients (subscribers)
- ~1000 events/second throughput

These limits are imposed by in-memory channel capacity and lock contention.
For deployments exceeding these limits or requiring multi-node distribution, migrate to Zenoh.
See `../infrastructure/zenoh-event-bus.md` for detailed migration path and Zenoh key expression patterns.

---

## SQLite sessions — colocated with event store

**Architectural simplification:**

Sessions are stored in a SQLite table alongside the event store, eliminating the need for a separate embedded database (redb).
This simplifies the stack: one database handles both events (append-only log) and sessions (ephemeral state).

**Why SQLite for sessions (instead of redb):**

| Consideration | SQLite | redb |
|---------------|--------|------|
| Async API | Yes (sqlx) | No (sync only) |
| Single database | Shares connection pool with events | Separate .redb file |
| Dependency count | Already in stack | Additional dependency |
| Operational model | One file to backup/manage | Two files |
| Performance | Sufficient for session workload | Faster raw KV, but overhead of sync wrappers |

The session workload (hundreds of reads/writes per second at most) is well within SQLite's capabilities.
The async API from sqlx integrates cleanly with axum handlers without spawn_blocking wrappers.

**Session table schema:**

```sql
CREATE TABLE sessions (
    session_id TEXT PRIMARY KEY,
    user_id TEXT,
    data JSON NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_accessed TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_sessions_expires ON sessions(expires_at);
```

**Session management pattern:**

The cookie-session pattern remains the same: HTTP-only cookies contain session IDs, SQLite stores session data server-side.
The cookie is a *reference* (opaque identifier), and SQLite provides the *dereferencing function* (ID -> SessionData).

```rust
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionData {
    pub session_id: String,
    pub user_id: Option<String>,
    pub data: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct SessionStore {
    pool: SqlitePool,
    ttl_days: i64,
}

impl SessionStore {
    pub fn new(pool: SqlitePool, ttl_days: i64) -> Self {
        Self { pool, ttl_days }
    }

    pub async fn get(&self, session_id: &str) -> Result<Option<SessionData>, sqlx::Error> {
        sqlx::query_as::<_, SessionData>(
            "SELECT * FROM sessions WHERE session_id = ? AND expires_at > datetime('now')"
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, session_id: &str) -> Result<SessionData, sqlx::Error> {
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::days(self.ttl_days);

        sqlx::query(
            r#"
            INSERT INTO sessions (session_id, data, created_at, last_accessed, expires_at)
            VALUES (?, '{}', ?, ?, ?)
            "#
        )
        .bind(session_id)
        .bind(now)
        .bind(now)
        .bind(expires)
        .execute(&self.pool)
        .await?;

        Ok(SessionData {
            session_id: session_id.to_string(),
            user_id: None,
            data: serde_json::json!({}),
            created_at: now,
            last_accessed: now,
            expires_at: expires,
        })
    }

    pub async fn update(&self, session: &SessionData) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE sessions
            SET user_id = ?, data = ?, last_accessed = datetime('now'),
                expires_at = datetime('now', '+' || ? || ' days')
            WHERE session_id = ?
            "#
        )
        .bind(&session.user_id)
        .bind(&session.data)
        .bind(self.ttl_days)
        .bind(&session.session_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, session_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sessions WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Cleanup expired sessions (run periodically)
    pub async fn cleanup_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= datetime('now')")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

// Axum extractor for sessions
pub struct Session {
    pub data: SessionData,
}

#[async_trait::async_trait]
impl FromRequestParts<AppState> for Session {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cookie extraction failed"))?;

        let session_id = jar.get("session_id")
            .map(|c| c.value().to_string())
            .unwrap_or_else(generate_session_id);

        let data = match state.session_store.get(&session_id).await {
            Ok(Some(session)) => session,
            Ok(None) => state.session_store.create(&session_id).await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Session create failed"))?,
            Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Session read failed")),
        };

        Ok(Session { data })
    }
}

fn generate_session_id() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}
```

**Security considerations:**

| Cookie Attribute | Value | Rationale |
|------------------|-------|-----------|
| `HttpOnly` | `true` | Prevents JavaScript access, mitigates XSS |
| `Secure` | `true` (prod) | HTTPS-only transmission |
| `SameSite` | `Lax` | CSRF protection, allows top-level navigation |
| `Path` | `/` | Scope to entire application |
| `Max-Age` | `30 days` | Session lifetime (application-specific) |

**Background cleanup task:**

```rust
pub async fn spawn_session_cleanup(store: Arc<SessionStore>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            match store.cleanup_expired().await {
                Ok(count) => tracing::info!("Cleaned up {} expired sessions", count),
                Err(e) => tracing::error!("Session cleanup failed: {}", e),
            }
        }
    });
}
```

**Comparison to Northstar's pattern:**

Northstar uses gorilla/sessions with cookie-based storage and NATS KV for persistence.
Ironstar separates concerns: cookies contain only the session ID (opaque reference), and SQLite stores the session payload server-side.

| Northstar (Go) | Ironstar (Rust) |
|----------------|-----------------|
| gorilla/sessions cookie store | axum-extra CookieJar + SQLite |
| Encrypted session data in cookie | Session ID in cookie, data in SQLite |
| NATS KV for server-side state | SQLite sessions table |
| `sess.Values["id"]` map access | Typed `SessionData` struct |

The Rust pattern provides stronger type safety (no `interface{}` or type assertions) and explicit separation between client-side identifier and server-side state.

---

## moka — analytics cache with TTL

**Problem statement:**

DuckDB analytics queries are expensive (seconds, not milliseconds) but dashboards are accessed frequently.
Caching avoids redundant queries while maintaining consistency with the event-sourced architecture.

**Why moka:**

| Consideration | moka | redb (alternative) |
|---------------|------|---------------------|
| Async API | Native (`moka::future::Cache`) | Sync only (needs spawn_blocking) |
| TTL support | Built-in | Manual implementation |
| Persistence | No (cache is rebuildable from DuckDB) | Yes (unnecessary for cache) |
| Eviction | LFU-based, near-optimal hit ratio | Manual |
| Concurrency | Lock-free | Single-writer MVCC |

moka is optimized for the analytics cache use case: read-heavy, TTL-based expiration, no persistence needed.

**Serialization with rkyv:**

Analytics results are serialized with rkyv for zero-copy deserialization:

| Format | Deserialize latency | Rationale |
|--------|---------------------|-----------|
| bincode | ~300ns | Good general-purpose |
| rkyv | ~21ns | 10-15x faster, optimal for read-heavy cache |

Zero-copy deserialization aligns with the cache's read-heavy access pattern.
Each query result is written once and read many times.

**Cache implementation:**

```rust
use moka::future::Cache;
use rkyv::{Archive, Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Cached analytics result
#[derive(Archive, Deserialize, Serialize, Clone)]
#[archive(check_bytes)]
pub struct CachedAnalytics {
    pub query_id: String,
    pub computed_at: i64,
    pub data: Vec<u8>,
}

pub struct AnalyticsCache {
    cache: Cache<String, Vec<u8>>,
    analytics: Arc<DuckDBService>,
}

impl AnalyticsCache {
    pub fn new(analytics: Arc<DuckDBService>) -> Self {
        let cache = Cache::builder()
            .max_capacity(1_000)
            .time_to_live(Duration::from_secs(300)) // 5 min default TTL
            .time_to_idle(Duration::from_secs(60))  // Evict if unused for 1 min
            .build();

        Self { cache, analytics }
    }

    /// Get cached result or compute and cache
    pub async fn get_or_compute<T>(
        &self,
        key: &str,
        compute: impl FnOnce() -> Result<T, Error>,
    ) -> Result<T, Error>
    where
        T: Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>,
        T::Archived: rkyv::Deserialize<T, rkyv::Infallible>,
    {
        // Check cache first
        if let Some(bytes) = self.cache.get(key).await {
            let archived = rkyv::check_archived_root::<T>(&bytes)
                .map_err(|_| Error::DeserializeFailed)?;
            return Ok(archived.deserialize(&mut rkyv::Infallible).unwrap());
        }

        // Compute on blocking thread pool (DuckDB is sync)
        let result = tokio::task::spawn_blocking(compute).await??;

        // Serialize and cache
        let bytes = rkyv::to_bytes::<_, 256>(&result)
            .map_err(|_| Error::SerializeFailed)?
            .to_vec();
        self.cache.insert(key.to_string(), bytes).await;

        Ok(result)
    }

    /// Invalidate cache entries matching predicate
    pub async fn invalidate_where<F>(&self, predicate: F)
    where
        F: Fn(&str) -> bool,
    {
        self.cache.invalidate_entries_if(move |key, _| predicate(key)).await;
    }
}
```

**Cache invalidation via event subscription:**

Cache entries are invalidated when relevant events arrive via the broadcast channel:

```rust
use tokio::sync::broadcast;

pub async fn run_cache_invalidator(
    cache: AnalyticsCache,
    mut event_rx: broadcast::Receiver<StoredEvent>,
) {
    while let Ok(event) = event_rx.recv().await {
        // Invalidate queries that depend on this aggregate type
        let aggregate_type = event.aggregate_type.clone();
        cache.invalidate_where(|key| {
            // Query keys encode their dependencies, e.g., "daily_counts:Todo"
            key.contains(&aggregate_type)
        }).await;
    }
}
```

**Integration with AppState:**

```rust
#[derive(Clone)]
pub struct AppState {
    pub event_store: Arc<SqliteEventStore>,
    pub session_store: Arc<SessionStore>,  // SQLite-based
    pub analytics: Arc<DuckDBService>,
    pub analytics_cache: AnalyticsCache,   // moka-based
    pub event_bus: broadcast::Sender<StoredEvent>,
    pub projections: Arc<Projections>,
}
```

**Detailed design:** See `../infrastructure/analytics-cache-architecture.md` for full evaluation of alternatives and cache invalidation patterns.

---

## Distributed event bus migration

For Zenoh migration and distributed deployment beyond single-node tokio::broadcast limits, see `../infrastructure/distributed-event-bus-migration.md`.

That document covers:
- When to migrate (scaling triggers)
- DualEventBus coexistence pattern
- Zenoh key expression patterns
- Migration phases (broadcast → dual → Zenoh)
- Rollback procedure
- Performance characteristics

---

## rust-embed — static assets as compile-time constants

**Algebraic justification:**

Static assets (CSS, JS, images) in a web application form a *constant set* at deployment time.
The embedding decision determines when this constant is bound: build time (embedded) or runtime (filesystem).
rust-embed implements the *Yoneda lemma* for assets: instead of representing "the asset at path X", it embeds the asset's content directly, eliminating the indirection.

```rust
// Embedding is a functor from Path -> Content
// rust-embed lifts this to compile time
#[derive(RustEmbed)]
#[folder = "static/dist"]
pub struct Assets;

// Access is now a pure lookup, not an IO operation
let content: Option<Cow<'static, [u8]>> = Assets::get("bundle.js");
```

**Dual-mode pattern:**

Ironstar requires different asset serving behavior in development and production:

| Mode | Behavior | Headers |
|------|----------|---------|
| Development | Serve from filesystem | `Cache-Control: no-store` |
| Production | Serve from embedded binary | `Cache-Control: max-age=31536000, immutable` |

rust-embed's behavior changes automatically based on build profile.
In debug builds (`cargo build`), it reads from the filesystem; in release builds (`cargo build --release`), it embeds files at compile time.

**Conditional compilation pattern:**

```rust
#[cfg(debug_assertions)]
pub fn static_routes() -> Router {
    // Dev: ServeDir for hot reload
    Router::new()
        .nest_service("/static", ServeDir::new("static/dist"))
        .layer(SetResponseHeaderLayer::if_not_present(
            CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
}

#[cfg(not(debug_assertions))]
pub fn static_routes() -> Router {
    // Prod: Embedded assets with immutable caching
    #[derive(RustEmbed)]
    #[folder = "static/dist"]
    struct Assets;

    async fn serve_asset(Path(path): Path<String>) -> Result<impl IntoResponse, StatusCode> {
        Assets::get(&path)
            .map(|asset| {
                let mime = mime_guess::from_path(&path).first_or_octet_stream();
                ([(CONTENT_TYPE, mime.as_ref())], asset.data)
            })
            .ok_or(StatusCode::NOT_FOUND)
    }

    Router::new()
        .route("/static/*path", get(serve_asset))
        .layer(SetResponseHeaderLayer::overriding(
            CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        ))
}
```

**Content hashing strategy:**

Unlike Go's `hashfs` crate which computes content hashes at runtime, ironstar delegates hashing to the build tool.
Rolldown generates content-hashed filenames (`bundle.[hash].js`) and a manifest mapping logical names to hashed filenames:

```json
{
  "index.ts": { "file": "bundle.a1b2c3d4.js", "css": ["bundle.x9y8z7w6.css"] }
}
```

At runtime, templates resolve hashed URLs via manifest lookup:

```rust
pub struct AssetManifest {
    entries: HashMap<String, ManifestEntry>,
}

impl AssetManifest {
    pub fn load() -> Self {
        #[cfg(debug_assertions)]
        let content = std::fs::read_to_string("static/dist/manifest.json")
            .unwrap_or_else(|_| "{}".to_string());

        #[cfg(not(debug_assertions))]
        let content = String::from_utf8_lossy(
            &Assets::get("manifest.json").expect("manifest.json missing").data
        ).into_owned();

        serde_json::from_str(&content).unwrap_or_default()
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.entries.get(name).map(|e| e.file.as_str())
    }
}

// In templates
fn asset_url(manifest: &AssetManifest, name: &str) -> String {
    format!("/static/{}", manifest.get(name).unwrap_or(name))
}
```

**Why rust-embed over alternatives:**

| Crate | Pros | Cons |
|-------|------|------|
| **rust-embed** | Auto dev/prod switching, derive macro, framework integration | No built-in hashing |
| include_dir | Fine-grained control | No dev mode, high compile overhead (730MB RAM for 64MB files) |
| static-files | Simple API | Outdated, no conditional compilation |

rust-embed's automatic mode switching aligns with Rust's `debug_assertions` convention, eliminating feature flag complexity.

**Effect boundary:**

Embedding is a *compile-time effect*—the filesystem read occurs during `cargo build`, not at runtime.
In production, asset access is pure lookup with no IO.
In development, IO occurs but is transparent to application code.

---

## process-compose — orchestration as declarative spec

**Algebraic justification:**

Process-compose configurations are *declarative specifications* of system topology:

```yaml
# This is a product type: Process = { command, depends_on, environment, ... }
processes:
  ironstar:
    command: ./result/bin/ironstar
    depends_on:
      styles: { condition: process_completed_successfully }
    environment:
      DATABASE_URL: "sqlite:./data/ironstar.db"

  styles:
    command: rolldown build
```

**Why not docker-compose:**

- Nix provides reproducible builds
- process-compose is lighter, no container overhead
- Better for development iteration

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

## Related documentation

- Design principles: `../core/design-principles.md`
- Frontend stack decisions: `frontend-stack-decisions.md`
- Backend core decisions: `backend-core-decisions.md`
- CQRS implementation: `cqrs-implementation-decisions.md`
- Build tooling decisions: `build-tooling-decisions.md`
- Event sourcing core concepts: `../cqrs/event-sourcing-core.md`
- SSE connection lifecycle: `../cqrs/sse-connection-lifecycle.md`
- Analytics cache design: `../infrastructure/analytics-cache-architecture.md`
- Session management: `../infrastructure/session-management.md`
- Development workflow: `../infrastructure/development-workflow.md`
