# Ironstar architecture decisions

This document records technology selection decisions for the ironstar stack.
For foundational principles guiding these decisions, see `design-principles.md`.

## Frontend tooling decisions

**What we use and why:**

| Tool | Role | Why |
|------|------|-----|
| Open Props + Open Props UI | CSS tokens & components | Pure CSS, zero runtime, modern CSS features |
| Rolldown | JS/CSS bundler | Rust-native (over Go-based esbuild), Vite 8 default |
| Vanilla Web Components | Third-party lib wrappers | Thin encapsulation, no reactivity system |
| Lucide | Icons | Build-time SVG inlining, zero runtime |
| TypeScript | Type safety | For minimal JS (web components only) |

**What we avoid and why:**

| Tool | Why Not |
|------|---------|
| Lit | Redundant reactivity: Lit's reactive properties duplicate Datastar signals |
| React / Vue / Svelte | SPA philosophy contradicts hypermedia-driven architecture |
| Leptos / Dioxus | Rust WASM frameworks would duplicate Datastar's role entirely |
| esbuild | Go-based; prefer Rust-native Rolldown for toolchain consistency |
| Shadcn/ui | Requires React runtime; Open Props UI achieves similar aesthetics with pure CSS |
| Tailwind CSS | JIT compiler requires build-time template scanning; Open Props uses runtime CSS variables |

**Web component pattern for Datastar integration:**

When third-party JavaScript libraries require client-side DOM manipulation (charts, drag-and-drop, rich editors), wrap them in vanilla web components that emit custom events:

```typescript
// Thin wrapper: no reactivity system, just encapsulation
class SortableList extends HTMLElement {
  connectedCallback() {
    Sortable.create(this, {
      onEnd: (evt) => {
        // Dispatch event for Datastar to handle
        this.dispatchEvent(new CustomEvent('reorder', {
          detail: { oldIndex: evt.oldIndex, newIndex: evt.newIndex },
          bubbles: true
        }));
      }
    });
  }
}
customElements.define('sortable-list', SortableList);
```

```html
<!-- Datastar handles all state and server communication -->
<sortable-list data-on:reorder="@post('/api/reorder', {body: evt.detail})">
  <!-- Items rendered by server -->
</sortable-list>
```

The web component is a thin adapter.
All state management flows through Datastar signals and server SSE responses.

---

## Component decisions

### 1. hypertext — HTML as pure functions

**Algebraic justification:**

HTML generation is a *monoid* under concatenation.
Hypertext makes this explicit:

```rust
// Renderable is the core trait for lazy HTML generation.
// Actual signature: fn render_to(&self, buffer: &mut Buffer<C>) where C: Context
// Simplified here for exposition; the Buffer wrapper prevents XSS.
trait Renderable {
    fn render_to(&self, buffer: &mut Buffer);
}

// Pure function: no allocation, no side effects
fn header() -> impl Renderable { maud! { header { "..." } } }
fn content(items: &[Item]) -> impl Renderable { maud! { main { ... } } }
fn footer() -> impl Renderable { maud! { footer { "..." } } }

// Monoidal composition (associative, identity exists)
fn page(items: &[Item]) -> impl Renderable {
    maud! {
        (header())      // No effect
        (content(items)) // No effect
        (footer())      // No effect
    }
}
// Effect (allocation) deferred to boundary: page.render()
```

**Why not maud directly:**

- Maud's `Markup` type is *eager*—it allocates on construction
- Hypertext's `impl Renderable` is a *thunk*—a description of computation
- This follows the *free monad* pattern: build a description, interpret at the boundary

**Type-safety:**

- Compile-time HTML validation
- No runtime template parsing errors
- Datastar attributes (`data-signals`, `data-on:*`) are stringly-typed at the HTML level but structurally validated by the macro

**Hypertext + Datastar integration:**

Datastar attributes use quoted strings in maud syntax since they are custom attributes:

```rust
use hypertext::prelude::*;

// Signal binding and event handlers
fn counter_component(count: i32) -> impl Renderable {
    maud! {
        div
            #counter
            "data-signals"=(format!(r#"{{count: {}}}"#, count))
        {
            p { "Count: " (count) }
            button
                "data-on:click"="@post('/api/increment')"
                class="button filled"
            {
                "Increment"
            }
        }
    }
}

// Two-way binding (data-bind for inputs, not data-bind:value)
fn input_field() -> impl Renderable {
    maud! {
        input
            type="text"
            "data-bind"="$todoText"
            placeholder="What needs to be done?"
            {}
    }
}

// Conditional rendering
fn loading_indicator() -> impl Renderable {
    maud! {
        div "data-show"="$loading" class="spinner" { "Loading..." }
    }
}
```

**Converting Renderable to PatchElements:**

Ironstar defines a helper trait to bridge hypertext's `Renderable` with datastar-rust's `PatchElements`.
This trait is not part of either library; it lives in your application code.

```rust
use hypertext::Renderable;
use datastar::prelude::*;

// Helper trait for ergonomic conversion
pub trait RenderableToDatastar: Renderable {
    fn to_patch_elements(&self) -> PatchElements {
        PatchElements::new(self.render().into_inner())
    }

    fn append_to(&self, selector: &str) -> PatchElements {
        PatchElements::new(self.render().into_inner())
            .selector(selector)
            .mode(ElementPatchMode::Append)
    }

    fn replace_inner(&self, selector: &str) -> PatchElements {
        PatchElements::new(self.render().into_inner())
            .selector(selector)
            .mode(ElementPatchMode::Inner)
    }
}

impl<T: Renderable> RenderableToDatastar for T {}

// ElementPatchMode variants:
// - Outer (default): morph entire element
// - Inner: morph inner HTML only
// - Remove: delete element
// - Replace: full replace without morphing
// - Prepend, Append: insert inside at start/end
// - Before, After: insert outside element
```

**Placement:** Define this trait in `src/presentation/helpers.rs` and import where needed in your axum handlers.

**Ergonomic benefit:** This trait provides a clean conversion from hypertext templates to datastar SSE events without manual `render()` and `into_inner()` boilerplate:

```rust
// Without helper trait (verbose)
let html = todo_list(&todos);
PatchElements::new(html.render().into_inner())

// With helper trait (concise)
todo_list(&todos).to_patch_elements()
```

**Usage in handler:**

```rust
async fn get_todos(State(store): State<TodoStore>) -> impl IntoResponse {
    let todos = store.list().await;
    let html = todo_list(&todos);
    Sse::new(stream::once(async move {
        Ok::<_, Infallible>(html.to_patch_elements().into())
    }))
}
```

---

### 2. axum — effect boundaries via extractors

**Algebraic justification:**

Axum's extractor pattern is essentially a *Reader monad* reified as types:

```rust
use axum::{
    extract::{State, Path, Json},
    response::IntoResponse,
};
use std::convert::Infallible;

// Extractors are Reader<Request, Result<T, Rejection>>
async fn handler(
    State(db): State<Pool>,           // Reader effect: access environment
    Path(id): Path<String>,           // Parser effect: extract from path
    Json(cmd): Json<Command>,         // Parser effect: deserialize body
) -> Result<impl IntoResponse, AppError> {  // Error effect explicit
    // Pure business logic here
    // Note: AppError is application-defined, see event-sourcing-sse-pipeline.md appendix
}
```

**Effect isolation:**

- Extractors handle IO/parsing at the boundary
- Handler body can be pure computation
- `Result` and `impl IntoResponse` make effects explicit in return type

**SSE as a lazy stream:**

```rust
use axum::response::sse::{Event, Sse};
use futures::stream::{Stream, StreamExt};
use std::convert::Infallible;

// Sse<S> is essentially Free[Stream, Event] — description of effects
async fn events(State(store): State<EventStore>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = store.subscribe()
        .map(|e| Ok(Event::default().data(e.to_json())));
    Sse::new(stream)  // No events emitted until consumed
}
```

---

### 3. tokio::sync::broadcast — event bus as observable

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

---

### 4. SQLite + sqlx — event store with type-safe queries

**Algebraic justification:**

sqlx provides compile-time query validation—the query is a *type-level proof* that it's valid:

```rust
use sqlx::SqlitePool;

// query_as! validates SQL at compile time against actual schema
// This is a dependent type approximation: Query<SQL, Schema> -> Result<T, E>
let events = sqlx::query_as!(
    StoredEvent,
    r#"
    SELECT id, aggregate_type, aggregate_id, sequence, event_type, payload, created_at
    FROM events
    WHERE aggregate_id = ?
    ORDER BY sequence
    "#,
    aggregate_id
)
.fetch_all(&pool)
.await?;
```

**Event sourcing as append-only:**

```rust
use sqlx::{SqlitePool, Transaction, Sqlite};

// Events form a monoid (concatenation) with identity (empty stream)
// Append is the only mutation — no update, no delete
pub async fn append(&self, events: Vec<NewEvent>) -> Result<Vec<StoredEvent>> {
    let mut tx = self.pool.begin().await?;

    // Transaction as a bracketed effect: begin -> operations -> commit/rollback
    for event in events {
        sqlx::query!(...).execute(&mut *tx).await?;
    }

    tx.commit().await?; // Effect realized at boundary

    // Publish to observers (pure data flow)
    for event in &stored {
        let _ = self.bus.publish(event.clone());
    }

    Ok(stored)
    // Note: Result type is application-specific, see event-sourcing-sse-pipeline.md appendix
}
```

**Why SQLite for event storage:**

- Embedded: no external server dependency
- Durability model is *synchronous* by default (WAL + fsync)
- Single-writer semantics prevent split-brain by construction

---

### 5. redb — session state with ACID guarantees

**Algebraic justification:**

redb's transaction model is a *bracket* pattern:

```rust
use redb::{Database, WriteTransaction, TableDefinition};

// WriteTransaction is a linear type (must be committed or dropped)
// This enforces the bracket law: acquire -> use -> release
let txn = db.begin_write()?;  // Acquire
{
    let mut table = txn.open_table(SESSIONS)?;
    table.insert(key, value)?;  // Use (pure within transaction)
}
txn.commit()?;  // Release (effect realized)
```

**Durability as explicit choice:**

```rust
// 1PC+C: single fsync, checksums detect partial writes
// 2PC: two fsyncs, stronger guarantee
// The choice is explicit in the API, not hidden
db.set_two_phase_commit(true);
```

**Why redb for session state:**

- Embedded: no external server dependency
- Session state is ephemeral but should survive restarts
- TTL logic is application-level (functional, testable)

**Session management pattern:**

Session management implements the cookie-session pattern: HTTP-only cookies contain session IDs, while redb stores session data server-side.
This separation follows the algebraic principle of referential transparency—the cookie is a *reference* (opaque identifier), and redb provides the *dereferencing function* (ID → SessionData).

```rust
use redb::{Database, TableDefinition, ReadableTable};
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Table definition as a type-level constant
const SESSIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("sessions");

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: Option<String>,
    pub created_at: u64,
    pub last_accessed: u64,
    // Application-specific fields
}

// Session store wraps redb database
pub struct SessionStore {
    db: Arc<Database>,
}

impl SessionStore {
    pub fn new(path: &str) -> Result<Self, redb::Error> {
        let db = Database::create(path)?;

        // Initialize table if needed
        let txn = db.begin_write()?;
        {
            let _ = txn.open_table(SESSIONS)?;
        }
        txn.commit()?;

        Ok(Self { db: Arc::new(db) })
    }

    // Pure read: ID -> Option<SessionData>
    pub fn get(&self, session_id: &str) -> Result<Option<SessionData>, redb::Error> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(SESSIONS)?;

        match table.get(session_id)? {
            Some(bytes) => {
                let data: SessionData = bincode::deserialize(bytes.value())
                    .map_err(|_| redb::Error::Corrupted)?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    // Effect: create or update session
    pub fn put(&self, session_id: &str, data: &SessionData) -> Result<(), redb::Error> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(SESSIONS)?;
            let bytes = bincode::serialize(data)
                .map_err(|_| redb::Error::Corrupted)?;
            table.insert(session_id, bytes.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    // Effect: delete session
    pub fn delete(&self, session_id: &str) -> Result<(), redb::Error> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(SESSIONS)?;
            table.remove(session_id)?;
        }
        txn.commit()?;
        Ok(())
    }
}

// Axum extractor for sessions
pub struct Session {
    pub id: String,
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

        // Get or create session ID
        let session_id = match jar.get("session_id") {
            Some(cookie) => cookie.value().to_string(),
            None => {
                // Generate new session ID
                let id = generate_session_id();
                id
            }
        };

        // Load or initialize session data
        let data = state.session_store
            .get(&session_id)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Session read failed"))?
            .unwrap_or_else(|| SessionData {
                user_id: None,
                created_at: unix_timestamp(),
                last_accessed: unix_timestamp(),
            });

        Ok(Session {
            id: session_id,
            data,
        })
    }
}

// Helper to update session and set cookie
pub async fn save_session(
    jar: CookieJar,
    store: &SessionStore,
    session: &Session,
) -> Result<CookieJar, redb::Error> {
    store.put(&session.id, &session.data)?;

    // Create HTTP-only, secure, SameSite cookie
    let cookie = Cookie::build(("session_id", session.id.clone()))
        .http_only(true)
        .secure(true)  // HTTPS only in production
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .path("/")
        .max_age(time::Duration::days(30))
        .build();

    Ok(jar.add(cookie))
}

// Usage in handlers
async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    mut session: Session,
) -> Result<impl IntoResponse, StatusCode> {
    // Update session after authentication
    session.data.user_id = Some("user123".to_string());
    session.data.last_accessed = unix_timestamp();

    // Save and return with cookie
    let jar = save_session(jar, &state.session_store, &session).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((jar, StatusCode::OK))
}

fn generate_session_id() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    base64::encode_config(&bytes, base64::URL_SAFE_NO_PAD)
}

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
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

**TTL and cleanup:**

Session expiration is application-level logic, not database-enforced:

```rust
impl SessionStore {
    // Functional TTL check: pure predicate
    fn is_expired(data: &SessionData, ttl_seconds: u64) -> bool {
        let now = unix_timestamp();
        now - data.last_accessed > ttl_seconds
    }

    // Effect: periodic cleanup task
    pub fn cleanup_expired(&self, ttl_seconds: u64) -> Result<usize, redb::Error> {
        let txn = self.db.begin_write()?;
        let mut removed = 0;

        {
            let mut table = txn.open_table(SESSIONS)?;
            let to_remove: Vec<String> = table.iter()?
                .filter_map(|entry| {
                    let (key, value) = entry.ok()?;
                    let data: SessionData = bincode::deserialize(value.value()).ok()?;
                    if Self::is_expired(&data, ttl_seconds) {
                        Some(key.value().to_string())
                    } else {
                        None
                    }
                })
                .collect();

            for key in to_remove {
                table.remove(key.as_str())?;
                removed += 1;
            }
        }

        txn.commit()?;
        Ok(removed)
    }
}

// Spawn background cleanup task
pub async fn spawn_session_cleanup(store: Arc<SessionStore>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour
        loop {
            interval.tick().await;
            let ttl = 30 * 24 * 60 * 60; // 30 days in seconds
            match store.cleanup_expired(ttl) {
                Ok(count) => tracing::info!("Cleaned up {} expired sessions", count),
                Err(e) => tracing::error!("Session cleanup failed: {}", e),
            }
        }
    });
}
```

**Comparison to Northstar's pattern:**

Northstar uses gorilla/sessions with cookie-based storage (encrypted session data in cookie) and NATS KV for persistence.
Ironstar separates concerns: cookies contain only the session ID (opaque reference), and redb stores the session payload server-side.

| Northstar (Go) | Ironstar (Rust) |
|----------------|-----------------|
| gorilla/sessions cookie store | axum-extra CookieJar + redb |
| Encrypted session data in cookie | Session ID in cookie, data in redb |
| NATS KV for server-side state | redb for server-side state |
| `sess.Values["id"]` map access | Typed `SessionData` struct |

The Rust pattern provides stronger type safety (no `interface{}` or type assertions) and explicit separation between client-side identifier and server-side state.

**Relevant source paths:**

- redb implementation: `~/projects/rust-workspace/redb/`
- Northstar session pattern: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/features/index/services/todo_service.go` (lines 53-76, 165-182)

---

### 6. DuckDB — analytics as pure queries

**Algebraic justification:**

DuckDB queries are *referentially transparent*—same input, same output:

```rust
use duckdb::Connection;

// Analytical query is a pure function: Projection -> Result<DataFrame>
let results = conn.execute(
    "SELECT aggregate_type, COUNT(*) as event_count
     FROM events
     GROUP BY aggregate_type",
    []
)?;
```

**Separation of concerns:**

- SQLite: OLTP (transactional event store)
- DuckDB: OLAP (analytical projections)
- This is the *CQRS* pattern: commands and queries have different algebra

**Threading constraints:**

DuckDB-rs is a synchronous, blocking library with specific thread safety characteristics:
- `Connection` is `Send` but NOT `Sync` — can be moved between threads but not shared
- `Statement` is `!Send` and `!Sync` — must stay on the thread where created

For async axum handlers, wrap DuckDB operations:

```rust
// Quick queries: use block_in_place
let result = tokio::task::block_in_place(|| {
    conn.prepare("SELECT ...")?.query_map([], |row| Ok(row.get(0)?))
})?;

// Long-running queries: use spawn_blocking
let result = tokio::task::spawn_blocking(move || {
    // DuckDB operations here
}).await??;
```

---

### 7. Zenoh — future distribution via unified abstraction

**Algebraic justification:**

Zenoh's key-expression model is a *free monoid* over path segments:

```rust
// Key expressions form a monoid under path concatenation
// Pattern matching is a semilattice (wildcards, unions)
let key = format!("events/{}/{}/{}", aggregate_type, aggregate_id, sequence);

// Put is an effectful operation (IO monad)
session.put(&key, payload).await?;

// Subscribe returns a stream (comonadic, produces values)
let subscriber = session.subscribe("events/**").await?;
```

**Why Zenoh over Apache Iggy:**

- Zenoh has both streaming and storage in one abstraction
- Storage backends (RocksDB, S3) provide durability
- Subscriptions provide the "watch" semantics
- More production-ready (Eclipse Foundation, April 2025 Gozuryū release)

**Migration path:**

```rust
use async_trait::async_trait;
use futures::stream::Stream;

// Trait abstraction allows swapping implementations
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, events: Vec<Event>) -> Result<Vec<StoredEvent>>;
    async fn load(&self, aggregate_id: &str) -> Result<Vec<StoredEvent>>;
    fn subscribe(&self) -> impl Stream<Item = StoredEvent>;
}

// Phase 1: SQLite implementation
pub struct SqliteEventStore { pool: SqlitePool, bus: EventBus }

// Phase 2: Zenoh implementation (same trait)
pub struct ZenohEventStore { session: zenoh::Session }
```

---

### 8. datastar-rust — frontend as signal algebra

**Algebraic justification:**

Datastar's signals are a *reactive graph*—essentially FRP (Functional Reactive Programming):

```rust
// Signals form an applicative functor
// - pure: lift value into signal
// - ap: combine signals
// - map: transform signal values

// PatchSignals is a morphism: JSON -> Signal State
PatchSignals::new(r#"{"count": 0, "loading": false}"#)

// PatchElements is a morphism: HTML -> DOM
PatchElements::new(render_component(state))
```

**SSE as a stream of patches:**

```rust
use axum::response::sse::{Event, Sse};
use datastar::prelude::*;  // PatchSignals
use futures::stream::{self, Stream};
use std::{convert::Infallible, time::Duration};

// The SSE stream is Free[Patch, DOM] — a program describing UI updates
async fn counter_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::unfold(0, |count| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let patch = PatchSignals::new(format!(r#"{{"count": {}}}"#, count + 1));
        Some((Ok(patch.write_as_axum_sse_event()), count + 1))
    });
    Sse::new(stream)
}
```

**ReadSignals extractor — type-safe signal parsing:**

The `ReadSignals<T>` extractor provides ergonomic, type-safe parsing of incoming Datastar signals.
It handles both GET (query param) and POST (JSON body) transparently:

```rust
use datastar::axum::ReadSignals;
use serde::Deserialize;

// Signal contract as algebraic product type
#[derive(Deserialize)]
struct TodoSignals {
    input: Option<String>,
    filter: TodoFilter,
    editing_id: Option<uuid::Uuid>,
}

#[derive(Deserialize)]
enum TodoFilter {
    All,
    Active,
    Completed,
}

// ReadSignals extracts and deserializes in one step
// Morphism: Request -> Result<T, Rejection>
async fn handle_add_todo(
    State(store): State<EventStore>,
    ReadSignals(signals): ReadSignals<TodoSignals>,
) -> impl IntoResponse {
    // signals.input, signals.filter available as typed values
    if let Some(text) = signals.input {
        store.append(TodoEvent::Added { text }).await?;
    }
    StatusCode::ACCEPTED
}
```

This pattern is preferred over manual JSON parsing.
The alternative (wrapping with `Query<Wrapper>` and `serde_json::from_str`) is verbose and error-prone.

**ts-rs integration for type-safe signal contracts:**

Signal types defined in Rust can generate corresponding TypeScript definitions via ts-rs, ensuring frontend and backend contracts stay synchronized:

```rust
use ts_rs::TS;
use serde::{Serialize, Deserialize};

// Derive TS alongside Serialize/Deserialize
#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "web-components/types/")]
pub struct TodoSignals {
    #[ts(optional)]
    pub input: Option<String>,
    pub filter: TodoFilter,
    #[ts(optional)]
    pub editing_id: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize, TS)]
pub enum TodoFilter {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "completed")]
    Completed,
}
```

Running `cargo test --lib` generates TypeScript in `web-components/types/`:

```typescript
// TodoSignals.ts (auto-generated)
export type TodoSignals = {
  input?: string;
  filter: TodoFilter;
  editing_id?: string;
};

export type TodoFilter = "all" | "active" | "completed";
```

This ensures the JSON structure in `data-signals` attributes matches what `ReadSignals<T>` expects.
See `docs/notes/architecture/signal-contracts.md` for detailed integration patterns.

---

### 9. open-props — design tokens as constants

**Algebraic justification:**

Open Props provides design tokens as CSS custom properties, representing a fundamentally different abstraction than utility-class systems.
Rather than a combinator language that generates styles at build time, Open Props is a *vocabulary* of constants that exist at runtime as CSS variables:

```rust
// Open Props tokens are constant lookups: TokenName -> Value
// This is dictionary access, not morphism application
maud! {
    div style="
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: var(--size-4);
        background: var(--gray-2);
        border-radius: var(--radius-2);
    " {
        // --size-4: constant value from sizing scale
        // --gray-2: constant value from color palette
        // var() performs lookup, not generation
    }
}
```

**Algebraic model: constants vs. morphisms**

Where Tailwind implements `ClassName -> CSSProperty` (a morphism from class to generated style), Open Props implements `TokenName -> Value` (a constant lookup).
The key difference is that Open Props tokens are *pre-defined values* referenced directly in CSS, while Tailwind classes are *instructions* for generating CSS at build time.

This makes Open Props closer to a **map data structure** than a combinator algebra.
The composition happens in your own CSS rules, not in generated utilities.

**Token categories:**

Open Props provides "sub-atomic styles" organized into semantic categories:

- **Colors**: 19 color palettes (gray, red, pink, purple, violet, indigo, blue, cyan, teal, green, lime, yellow, orange, choco, brown, sand, camo, jungle) with OKLCH variants for modern color spaces
- **Sizing**: `--size-*` (rem-based), `--size-fluid-*` (clamp-based responsive), `--size-content-*`, `--size-header-*`, `--size-*-px` (pixel-based)
- **Typography**: `--font-sans`, `--font-serif`, `--font-mono`, `--font-weight-*`, `--font-size-*`, `--font-lineheight-*`
- **Spacing**: `--space-*` for margins/padding
- **Animations**: 15+ keyframe animations (`@keyframes` defined, ready to use)
- **Easings**: 60+ easing functions including spring, elastic, bounce variations
- **Shadows**: `--shadow-*` (elevation-based), `--inner-shadow-*`
- **Borders**: `--border-size-*`, `--radius-*` (border radius), `--radius-*-px`
- **Gradients**: predefined gradient values
- **Z-index**: `--layer-*` (semantic layering)
- **Aspect ratios**: `--ratio-*` (square, landscape, portrait, widescreen, ultrawide, golden)

**Framework-agnostic abstraction:**

CSS custom properties are browser-native.
This means:

- No build-time class scanning or JIT compilation required
- Works with any templating system (hypertext, JSX, plain HTML)
- Tokens are defined once, available everywhere
- No purging step needed (tokens are referenced, not generated)

**Why chosen:**

Open Props aligns better with Datastar's hypermedia philosophy.
When the server generates complete HTML fragments, it needs to reference *known class names* or *CSS properties*.
Open Props provides stable token names that can be referenced in inline styles or custom CSS classes without requiring a build-time scanning step to detect which utilities are used.

The backend-driven model means: server knows exact styling -> emits HTML with tokens -> browser applies via native CSS variables.
There's no need for a JIT compiler to watch template files and generate utility classes.

**Local repository:**

- `open-props` at `~/projects/lakescope-workspace/open-props` - CSS design tokens library

**Integration pattern:**

```rust
// Import Open Props tokens in your CSS
// @import "open-props/style";
// Or selective imports:
// @import "open-props/colors";
// @import "open-props/sizes";

// Then reference in templates via inline styles or custom classes
maud! {
    button style="
        padding: var(--size-2) var(--size-4);
        background: var(--blue-6);
        color: var(--gray-0);
        border-radius: var(--radius-2);
        font-weight: var(--font-weight-6);
    " {
        "Submit"
    }
}
```

**Effect boundary:**

Open Props CSS is imported at load time as standard CSS.
No build-time generation step required beyond standard CSS concatenation/minification.
The browser's CSS engine resolves `var()` lookups during style computation.

---

### 10. open-props-ui — semantic components via modern CSS

**Algebraic justification:**

Open Props UI extends Open Props tokens into a component layer using pure CSS and modern browser features.
Rather than generating utility classes or requiring a JavaScript framework, it provides semantic component classes that compose via a three-layer architecture:

```rust
// Three-layer composition:
// 1. Open Props tokens (constants)
// 2. Theme variables (application-specific derivations)
// 3. Component classes (semantic composition)

// Layer 1: Token constants
// --blue-6, --size-2, --radius-2 (from Open Props)

// Layer 2: Theme variables (your app-level CSS)
// --brand-primary: var(--blue-6);
// --spacing-md: var(--size-2);
// --corner-md: var(--radius-2);

// Layer 3: Component classes
maud! {
    button class="button filled" {
        // .button uses theme variables
        // .filled uses --brand-primary
        "Submit"
    }

    div class="card" {
        // .card uses theme variables for padding, radius, shadow
        h2 { "Card Title" }
        p { "Card content goes here." }
    }
}
```

**Modern CSS features:**

Open Props UI leverages CSS capabilities from 2023+ browsers:

- **OKLCH colors**: Perceptually uniform color space via `oklch()` function
- **Container queries**: `@container` for component-scoped responsive behavior
- **CSS layers**: `@layer` for cascade management
- **color-mix()**: Dynamic color blending without preprocessor
- **light-dark()**: Native light/dark theme switching without JavaScript
- **Nesting**: Native CSS nesting without Sass/PostCSS
- **:has()**: Parent selector for relational styling

Browser requirements: Chrome 111+, Firefox 119+, Safari 17+ (mid-2023 forward).

**Component catalog:**

Open Props UI provides 31 component types:

- **Forms**: button, field (input/textarea/select), checkbox, radio, switch, slider
- **Layout**: card, dialog (modal), drawer (side panel), tabs, accordion
- **Navigation**: navbar, breadcrumb, menu, pagination
- **Feedback**: alert, toast, tooltip, badge, progress, skeleton
- **Data**: table, avatar, chip/tag
- **Media**: image, video wrapper
- **Utilities**: divider, spacer

**Copy-paste ownership model:**

Open Props UI differs from traditional component libraries.
Components are *copied into your project*, not imported from node_modules:

```bash
# Typical workflow
cp node_modules/open-props-ui/components/button.css src/components/
# Edit button.css to match your needs
```

This gives you **full ownership**:

- Modify component styles directly (no overrides needed)
- No breaking changes from library updates
- Complete transparency (you own the CSS)
- Tree-shake automatically (only include what you use)

**Why chosen over DaisyUI:**

| Consideration | DaisyUI | Open Props UI |
|--------------|---------|---------------|
| Build dependencies | Tailwind JIT compiler, PostCSS | None (pure CSS) |
| Class generation | Build-time utility scanning | No generation step |
| Modern CSS | Limited (Tailwind constraints) | Full (OKLch, container queries, :has()) |
| Ownership model | NPM dependency, version updates | Copy-paste, full control |
| Server-side HTML | Requires JIT to scan templates | Direct token/class reference |
| Theme system | CSS variables via Tailwind config | Native CSS custom properties |

For Ironstar's hypermedia-driven architecture, Open Props UI provides:

1. **Stable class names**: The server can emit `class="button filled"` without build-time coordination
2. **No JIT scanning**: Backend templates don't need to be watched for class extraction
3. **Modern CSS alignment**: Embraces browser-native features over build-time abstractions
4. **Copy-paste ownership**: Full control over component CSS without override complexity

**Why not Shadcn/ui:**

Shadcn/ui provides similar aesthetics but requires React:

```tsx
// Shadcn: React component with JS runtime
import { Button } from "@/components/ui/button"
<Button variant="primary" size="lg">Submit</Button>
```

```rust
// Open Props UI: Pure CSS class, no runtime
maud! { button class="button filled large" { "Submit" } }
```

When Datastar already provides reactivity, adding React for UI components introduces redundant complexity.

**Local repository:**

- `open-props-ui` at `~/projects/lakescope-workspace/open-props-ui` - Pure CSS component library

**Integration pattern:**

```rust
// 1. Import Open Props and component CSS in your stylesheet
// @import "open-props/style";
// @import "./components/button.css";  // Copied from open-props-ui
// @import "./components/card.css";

// 2. Define theme variables (optional customization layer)
// :root {
//   --brand-primary: var(--blue-6);
//   --brand-secondary: var(--purple-6);
// }

// 3. Use semantic classes in templates
maud! {
    button class="button filled large" {
        "Submit"
    }

    div class="card" {
        div class="card-header" {
            h2 { "Title" }
        }
        div class="card-body" {
            p { "Content goes here." }
        }
    }
}
```

**Effect boundary:**

Open Props UI CSS is loaded as standard stylesheets.
Theme switching via `light-dark()` is handled by the browser's native color scheme support (no JavaScript).
Component styles apply immediately via the browser's CSS engine with no runtime compilation.

---

### 11. process-compose — orchestration as declarative spec

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

### 12. Rolldown — build as a pure morphism

**Algebraic justification:**

A bundler implements a morphism in the category of module graphs: `Bundle: ModuleGraph -> OutputChunks`.
For this morphism to compose cleanly with other build steps, it must be *referentially transparent*: identical inputs yield identical outputs.

Rolldown's three-stage pipeline makes this explicit:

```
Scan: Sources -> ModuleGraph      (parsing as functor)
Link: ModuleGraph -> SymbolTable  (resolution as join semilattice)
Generate: (ModuleGraph, SymbolTable) -> Chunks  (codegen as fold)
```

Each stage transforms immutable data structures.
The `ModuleTable` and `SymbolRefDb` built during scanning are not mutated during linking; linking produces a new `SymbolTable`.
This mirrors the *free monad* pattern: build a description (module graph), interpret at the boundary (generate chunks).

```typescript
// rolldown.config.ts
import { defineConfig } from 'rolldown';
import postcss from 'rolldown-plugin-postcss';

export default defineConfig({
  input: 'web-components/index.ts',
  output: {
    dir: 'static/dist',
    format: 'esm',
    // Deterministic chunk naming: content hash as identity
    entryFileNames: '[name].[hash].js',
    chunkFileNames: '[name].[hash].js',
  },
  plugins: [
    // PostCSS runs within the bundle pipeline
    // Open Props imports and custom CSS processed as standard CSS
    postcss({
      config: './postcss.config.js',
      extract: 'bundle.css',
      minimize: true,
    }),
  ],
});
```

**Why Rust-native matters:**

| Property | esbuild (Go) | Rolldown (Rust) |
|----------|--------------|-----------------|
| Memory model | GC-managed heap | Ownership + borrowing |
| Parallelism | Goroutines + shared heap | Rayon + zero-copy |
| WASM performance | 22s (2.5k modules) | 613ms (2.5k modules) |
| Memory overhead | GC pauses | Deterministic allocation |

The algebraic significance: Rust's ownership system provides *linear types* at the language level.
A `ModuleTask` owns its module data; when processing completes, ownership transfers to `ModuleTable`.
No defensive copies, no GC pauses during the build effect.

**Why Rolldown over esbuild:**

- esbuild lacks Rollup-compatible plugin API (ecosystem fragmentation)
- esbuild's tree-shaking is less sophisticated (larger bundles)
- esbuild's WASM performance degrades significantly in browser environments
- Rolldown unifies dev bundling and production bundling (same tool, same behavior)
- Rust aligns with the stack's language choice (no Go in the dependency tree)

**Effect boundary:**

The build is a single effect executed at deploy time or watch-mode trigger.
All file I/O, network fetches, and process spawning occur within `rolldown build`.
The effect completes atomically: either all outputs are written or none are (via temp files + rename).

**PostCSS integration:**

PostCSS plugins run as transforms within the bundle pipeline.
For Open Props, this primarily handles CSS imports, autoprefixing, and minification.
No class extraction or template scanning is required—Open Props tokens are referenced directly in CSS via `var()`, making the pipeline a straightforward transform: `Import(CSS) -> Optimize(CSS) -> Output`.

---

### 13. Lucide — icons as pure data

**Algebraic justification:**

Icons are *constants*: they do not vary with application state.
This places them in a different category than reactive UI elements.
The optimal representation is as *compile-time constants*, not runtime-fetched resources.

Lucide provides icons as pure functions from props to SVG:

```typescript
// Each icon is a pure function: IconProps -> SVGElement
// No side effects, no state, no network requests
import { Camera, Settings, User } from 'lucide-static';

// Icon as string constant (build-time resolved)
const cameraIcon: string = Camera;  // Pure SVG markup

// Or in hypertext templates (Rust side):
// The SVG string is embedded at compile time
maud! {
    button class="btn" {
        (Raw::dangerously_create(include_str!("../static/icons/camera.svg")))
        "Upload"
    }
}
```

**Build-time inlining pattern:**

Rather than loading icons at runtime (HTTP requests, bundle bloat, FOUC), Lucide icons are inlined during the build:

```
Build step: Icon name -> SVG file -> Embedded constant
Runtime: Zero icon-related network requests, zero JS execution
```

This is the *Yoneda embedding* for icons: instead of asking "what icon should I render?" at runtime, we pre-compute the answer and embed it.
The icon's identity is its content, making caching trivial (content-addressed).

**Why build-time over runtime:**

| Approach | HTTP Requests | JS Runtime | Bundle Size | FOUC Risk |
|----------|--------------|------------|-------------|-----------|
| Runtime fetch | N per icon | Required | Small initial | High |
| Icon font | 1 (all icons) | CSS only | Large fixed | Medium |
| SVG sprite | 1 (all icons) | None | Large fixed | Low |
| Build-time inline | 0 | None | Minimal (used only) | None |

Build-time inlining achieves the optimal trade-off: zero runtime overhead, minimal bundle size (tree-shaking applies), and no flash of unstyled content.

**Why Lucide specifically:**

- Consistent 24x24 grid with 2px stroke (compositional uniformity)
- 1600+ icons covering common UI patterns
- MIT licensed, actively maintained fork of Feather Icons
- Multiple output formats: individual SVGs, static strings, framework components
- `lucide-static` package provides SVG strings for server-side embedding

**Integration with hypertext:**

```rust
// icons.rs - Generated at build time by a simple script
pub mod icons {
    pub const CAMERA: &str = include_str!("../../static/icons/camera.svg");
    pub const SETTINGS: &str = include_str!("../../static/icons/settings.svg");
    pub const USER: &str = include_str!("../../static/icons/user.svg");
}

// In templates
use crate::icons;

fn icon_button(icon: &str, label: &str) -> impl Renderable {
    maud! {
        button class="button icon-only" {
            (Raw::dangerously_create(icon))
            span { (label) }
        }
    }
}

// Usage: icon_button(icons::CAMERA, "Upload")
```

---

### 14. rust-embed — static assets as compile-time constants

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

## Architecture summary

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
│  │ SQLite/sqlx │ redb        │ DuckDB      │ Zenoh (future)      │ │
│  │ EventStore  │ SessionKV   │ Analytics   │ Distributed         │ │
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
| **SQLite/sqlx** | Event store | Append monoid | `.commit()` |
| **redb** | Session KV | Bracket (linear) | `.commit()` |
| **DuckDB** | Analytics | Pure query | `.execute()` |
| **Zenoh** | Distribution | Free monoid | `.put()` / `.subscribe()` |
| **datastar-rust** | Frontend | FRP signals | SSE emit |
| **open-props** | CSS Tokens | Constants (map/dictionary) | CSS `var()` resolution |
| **open-props-ui** | CSS Components | Three-layer composition | Style application |
| **process-compose** | Orchestration | Product spec | Process start |
| **Rolldown** | JS/CSS bundler | Pure morphism (deterministic) | `rolldown build` |
| **Lucide** | Icons | Constants (Yoneda embedding) | Build time |
| **rust-embed** | Asset embedding | Compile-time constants | `cargo build --release` |

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
│   ├── session_store.rs         # redb session KV
│   ├── analytics.rs             # DuckDB queries
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

When the codebase grows, extract layers into separate crates:

```
ironstar/
├── Cargo.toml                    # [workspace] with members
├── crates/
│   ├── ironstar-domain/          # Pure types, no dependencies
│   │   └── Cargo.toml
│   ├── ironstar-app/             # Business logic
│   │   └── Cargo.toml            # depends on: ironstar-domain
│   ├── ironstar-infra/           # Persistence, external services
│   │   └── Cargo.toml            # depends on: ironstar-domain, ironstar-app
│   └── ironstar-web/             # HTTP layer
│       └── Cargo.toml            # depends on: all above
└── ironstar/                     # Binary crate, wires everything
    └── Cargo.toml                # depends on: ironstar-web
```

Each crate has a dedicated `crate.nix` file for Nix integration (per `rustlings-workspace` pattern).

### AppState composition

Dependency injection uses axum's `State` extractor with a shared state struct:

```rust
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub event_store: Arc<SqliteEventStore>,
    pub session_store: Arc<RedbSessionStore>,
    pub analytics: Arc<DuckDbAnalytics>,
    pub event_bus: broadcast::Sender<StoredEvent>,
    pub projections: Arc<Projections>,
    #[cfg(debug_assertions)]
    pub reload_tx: broadcast::Sender<()>,
}

impl AppState {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let event_store = SqliteEventStore::new(&config.database_url).await?;
        let session_store = RedbSessionStore::new(&config.session_path)?;
        let analytics = DuckDbAnalytics::new(&config.analytics_path)?;
        let (event_bus, _) = broadcast::channel(256);

        // Initialize projections by replaying events
        let projections = Projections::init(&event_store, event_bus.clone()).await?;

        Ok(Self {
            event_store: Arc::new(event_store),
            session_store: Arc::new(session_store),
            analytics: Arc::new(analytics),
            event_bus,
            projections: Arc::new(projections),
            #[cfg(debug_assertions)]
            reload_tx: broadcast::channel(16).0,
        })
    }
    // Note: Error type is application-specific, see event-sourcing-sse-pipeline.md appendix
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

- Design principles: `design-principles.md`
- Development workflow: `development-workflow.md`
- Event sourcing patterns: `event-sourcing-sse-pipeline.md`
- Third-party integration: `integration-patterns.md`
- Signal contracts: `signal-contracts.md`
- Build pipeline: `frontend-build-pipeline.md`
