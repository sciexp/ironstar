# Backend core architecture decisions

This document records backend core technology selection decisions for the ironstar stack, covering HTML templating, web framework, and database choices.
For frontend, infrastructure, and CQRS implementation decisions, see the related documentation section below.

## hypertext — HTML as pure functions

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

## axum — effect boundaries via extractors

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
    // Note: AppError is application-defined, see event-sourcing-core.md appendix
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

## SQLite + sqlx — event store with type-safe queries

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
    // Note: Result type is application-specific, see event-sourcing-core.md appendix
}
```

**Why SQLite for event storage:**

- Embedded: no external server dependency
- Durability model is *synchronous* by default (WAL + fsync)
- Single-writer semantics prevent split-brain by construction

**Event store schema informed by sqlite-es:**

The sqlite-es crate uses a compound primary key `(aggregate_type, aggregate_id, sequence)` with JSON payload.
Ironstar adapts this with a global sequence number (INTEGER PRIMARY KEY AUTOINCREMENT) for SSE Last-Event-ID support, enabling browser reconnection to resume from the correct position.

---

## DuckDB — analytics as pure queries

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

**Remote data sources via httpfs:**

Beyond local analytics over event history, DuckDB can query remote datasets directly via its httpfs extension.
This enables the axum backend to fetch external data for ECharts/Vega visualizations without requiring local data ingestion.

Supported protocols:
- `hf://` — HuggingFace-hosted datasets (parquet files on HuggingFace Hub)
- `s3://` — S3-compatible object storage (AWS S3, Cloudflare R2, MinIO)
- DuckLake catalogs — versioned data lake abstraction over object storage

```rust
use duckdb::Connection;

// Query HuggingFace-hosted parquet data directly
let conn = Connection::open_in_memory()?;
conn.execute("INSTALL httpfs; LOAD httpfs;", [])?;

let results = conn.execute(
    "SELECT * FROM 'hf://datasets/org/dataset/data.parquet' LIMIT 100",
    []
)?;

// S3-compatible storage (e.g., Cloudflare R2)
conn.execute("SET s3_endpoint='account.r2.cloudflarestorage.com';", [])?;
let results = conn.execute(
    "SELECT * FROM read_parquet('s3://bucket/analytics/*.parquet')",
    []
)?;
```

This pattern separates concerns: visualization components (ECharts, Vega-Lite) handle rendering, while DuckDB handles data access regardless of whether the source is local event projections or remote datasets.

**Local references:**
- `~/projects/rust-workspace/rust-duckdb-huggingface-ducklake-query` — reference implementation demonstrating hf:// queries
- `~/projects/omicslake-workspace/marhar-frozen` — DuckLake fixture data creation
- `~/projects/omicslake-workspace/marhar-duckdb-tools` — DuckDB tooling for data lake operations

---

## Related documentation

- Design principles: `design-principles.md`
- Frontend stack decisions: `frontend-stack-decisions.md`
- Infrastructure decisions: `infrastructure-decisions.md`
- CQRS implementation: `cqrs-implementation-decisions.md`
- Build tooling decisions: `build-tooling-decisions.md`
- Event sourcing core concepts: `event-sourcing-core.md`
- Command write patterns: `command-write-patterns.md`
- Analytics cache design: `analytics-cache-architecture.md`
- Module organization: `architecture-decisions.md` (see Module organization section)
