# Web Framework Evaluation: axum vs Rocket vs actix-web

This document evaluates three Rust web frameworks for ironstar's hypermedia-driven, event-sourced architecture.
Evaluation date: 2025-01-05.

## Executive summary

**Recommendation: axum 0.8+** with high confidence (90%).

axum is the optimal choice for ironstar because it:
1. Provides native SSE support that integrates seamlessly with datastar-rust
2. Offers the best alignment with functional programming principles via tower middleware composition
3. Has the most ergonomic extractor pattern for the Reader monad dependency injection style
4. Maintains the lowest memory footprint under load
5. Is already supported by datastar-rust with the `ReadSignals<T>` extractor

Rocket is a viable alternative (75% confidence) with excellent SSE support and cleaner syntax, but its macro-heavy approach and smaller ecosystem make it less suitable for ironstar's explicit-effects architecture.

actix-web is not recommended (40% confidence) due to lack of built-in SSE support, which would require significant custom implementation for datastar integration.

## Comparison matrix

| Dimension | axum | Rocket | actix-web | Winner |
|-----------|------|--------|-----------|--------|
| **SSE streaming** | Native `Sse<S>` type | Native `EventStream!` macro | Manual (BodyStream) | axum/Rocket tie |
| **datastar-rust integration** | Full (ReadSignals + Event) | Partial (Event only) | None | axum |
| **Extractor ergonomics** | Excellent (tower-based) | Good (Outcome type) | Good (tuple-based) | axum |
| **Middleware composition** | Tower layers (algebraic) | Fairings (lifecycle hooks) | Transform/Service | axum |
| **Functional alignment** | Excellent (Reader monad) | Good (explicit guards) | Moderate (type erasure) | axum |
| **Type safety** | Explicit traits | Macro-generated | Macro + traits | axum |
| **Performance** | Very fast, lowest memory | Very fast | Fastest throughput | actix-web |
| **Ecosystem** | Large (tower ecosystem) | Moderate | Large | axum |
| **Documentation** | Good | Excellent | Excellent | Rocket |
| **Release stability** | Stable (tokio team) | Stable (v0.5) | Stable | Tie |
| **Learning curve** | Moderate | Moderate | Steeper | Tie |

## Detailed analysis

### 1. SSE streaming capability

**axum:**
- Native `axum::response::sse::Sse<S>` wrapper with `keep_alive` support
- Stream-based via `S: TryStream<Ok = Event>`
- Automatic `Content-Type: text/event-stream` and `Cache-Control: no-cache` headers
- `KeepAliveStream` wrapper handles periodic heartbeats without blocking
- Event ID support enables `Last-Event-ID` header for client reconnection

```rust
async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive")
    )
}
```

**Rocket:**
- Native `EventStream![]` macro with built-in heartbeat (30s default)
- Generator syntax via `EventStream! { yield event; }`
- Automatic protocol normalization (handles carriage returns)
- Event type includes `id`, `event`, `retry`, and `data` fields

```rust
#[get("/feed")]
fn feed() -> EventStream![] {
    EventStream! {
        yield Event::data("Hello");
    }
}
```

**actix-web:**
- No dedicated SSE type; requires manual `BodyStream` implementation
- Must manually format `data: {...}\n\n` frames
- Must set SSE headers manually
- Requires building custom SSE infrastructure

**Verdict:** axum and Rocket both provide first-class SSE support. axum's approach is more explicit (traits over macros), while Rocket's generator syntax is more concise. actix-web requires significant custom work.

### 2. Extractor/guard ergonomics

**axum:**
- `FromRequest<S>` and `FromRequestParts<S>` traits
- Extractors compose via tuples with automatic ordering (body-consuming last)
- `State<T>` + `FromRef` pattern maps cleanly to Reader monad
- Error handling via `Rejection: IntoResponse` allows rich error types
- `Option<T>` wrapper for graceful extraction failure

```rust
async fn handler(
    State(db): State<Pool>,           // FromRequestParts - any position
    Path(id): Path<String>,           // FromRequestParts - any position
    Json(body): Json<Command>,        // FromRequest - must be last
) -> Result<Json<Response>, AppError> {
    // Handler receives extracted dependencies
}
```

**Rocket:**
- `FromRequest<'r>` trait with `Outcome<S, E, F>` (Success/Error/Forward)
- Guards fire left-to-right with short-circuit on failure
- `Forward` outcome allows trying next route (unique feature)
- Request-local caching via `local_cache()` prevents duplicate work
- Macro-generated extraction via route attributes

```rust
#[get("/users/<id>")]
async fn handler(
    id: &str,
    state: &State<Pool>,
    user: AuthUser,
) -> Result<Json<User>, Custom<&str>> {
    // Guards extracted by position
}
```

**actix-web:**
- `FromRequest` trait with tuple-based extraction (max 12 types)
- All extractors run simultaneously (no ordering control)
- Error handling via `type Error: Into<Error>` with type erasure
- `Data<T>` for state with scope-based lookups

```rust
async fn handler(
    path: web::Path<Id>,
    query: web::Query<Params>,
    data: web::Data<Pool>,
) -> impl Responder {
    // Tuple extraction
}
```

**Verdict:** axum's extractor system best aligns with the Reader monad pattern. The `FromRef` trait enables clean dependency extraction from a single `AppState` type. Rocket's `Outcome` type is algebraically elegant but the macro-heavy syntax obscures the types.

### 3. Middleware and tower integration

**axum:**
- Built on tower `Layer` + `Service` traits
- Middleware composes algebraically (associative layering)
- Full tower-http ecosystem: compression, CORS, tracing, timeouts
- `from_fn` for simple middleware, full `Service` impl for complex cases
- Layers apply to routes, routers, or entire apps

```rust
let app = Router::new()
    .route("/", get(handler))
    .layer(ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30))));
```

**Rocket:**
- Fairings with 5 explicit lifecycle hooks:
  - `on_ignite`: App startup (can modify config)
  - `on_liftoff`: After bind (read-only)
  - `on_request`: Before routing (mutable)
  - `on_response`: After handler (mutable)
  - `on_shutdown`: Graceful shutdown
- `AdHoc` fairings for closures
- Singleton support (last-attached wins)

```rust
rocket::build()
    .attach(counter_fairing)
    .attach(AdHoc::on_request("logger", |req, _| Box::pin(async {
        info!("Request: {}", req.uri());
    })))
```

**actix-web:**
- `Transform` + `Service` pattern (similar to tower but not compatible)
- `from_fn` middleware helper
- LIFO execution order (reversed from registration)
- Per-scope middleware support

**Verdict:** axum's tower integration enables reuse of the entire tower ecosystem. Rocket's fairings are more structured but less composable. actix-web's custom Transform/Service pattern precludes tower middleware reuse.

### 4. Async runtime and concurrency

**axum:**
- Direct tokio integration (maintained by tokio team)
- No runtime abstraction layer
- Full access to tokio primitives (channels, tasks, time)
- Connection pooling via tokio-native crates (sqlx, deadpool)
- Graceful shutdown via `axum::serve::Serve::with_graceful_shutdown`

**Rocket:**
- tokio-based via `rocket::tokio` re-export
- Async support via `#[rocket::async_trait]` macro
- `#[rocket::main]` macro for runtime setup
- Graceful shutdown built into `Rocket::launch()`

**actix-web:**
- `actix-rt` runtime (thin tokio wrapper)
- Single-threaded workers by default (better isolation)
- Can run under `#[tokio::main]` but loses some features
- Multi-worker architecture with factory closure per worker

**Verdict:** axum's direct tokio integration provides the cleanest async story. All three work well with tokio-based connection pools (sqlx, async-duckdb).

### 5. Type safety and compile-time guarantees

**axum:**
- Handler signatures are explicit async functions
- Extractors have clear trait bounds
- State extraction via `FromRef` is type-checked at compile time
- No hidden code generation (macros only for routing DSL)
- `IntoResponse` trait enables any return type

**Rocket:**
- Route attributes generate significant code
- Guards are type-safe but types hidden by macros
- `Sentinel` pattern provides compile-time checks for managed state
- Responder trait allows custom response types
- Some type information lost in macro expansion

**actix-web:**
- Handler trait bounds explicit but verbose
- Tuple extraction limits (max 12-16 parameters)
- `ResponseError` trait with type erasure (`Box<dyn ResponseError>`)
- Macro routing similar to Rocket

**Verdict:** axum provides the most transparent type safety with minimal macro magic. The explicit trait bounds make error messages clearer and types more visible.

### 6. Ecosystem and maintenance

**axum:**
- Maintained by tokio team (long-term stability)
- Tower middleware ecosystem (~50+ crates)
- Active development (0.8 released recently)
- Strong integration with tower-http
- Growing adoption in production

**Rocket:**
- Single maintainer (Sergio Benitez)
- Smaller but focused ecosystem
- v0.5 released November 2023 (4 years development)
- Excellent documentation
- Stable release cadence

**actix-web:**
- Mature project with corporate backing
- Large ecosystem
- Well-documented
- Active community

**Verdict:** All three have stable ecosystems. axum's tower ecosystem is the most composable. Rocket's documentation is excellent but ecosystem is smaller.

### 7. Performance characteristics

Based on [2024-2025 benchmark data](https://markaicode.com/rust-web-frameworks-performance-benchmark-2025/):

| Metric | actix-web | axum | Rocket |
|--------|-----------|------|--------|
| Requests/sec | Highest | ~95% of actix | ~85% of actix |
| Memory (idle) | 12-15MB | 10-13MB | 15-20MB |
| Memory (load) | Higher | Lowest | Moderate |
| Latency P99 | Lowest | ~Equal | Slightly higher |

**Important context:** All three frameworks are extremely fast. Real-world bottlenecks are almost always database queries, network I/O, or business logic — not the web framework.

**Verdict:** actix-web has highest raw throughput, axum has lowest memory footprint. For ironstar's SSE-heavy workload with ~10K concurrent connections, axum's memory efficiency is advantageous.

### 8. Developer experience

**axum:**
- Clear, explicit error messages
- Hot reload via cargo-watch
- Testing via `axum::test::TestClient` or tower's testing utilities
- No nightly Rust required (stable since 0.6)

**Rocket:**
- Excellent error messages with suggestions
- Built-in testing framework
- Hot reload via cargo-watch
- No nightly required since 0.5

**actix-web:**
- Good error messages
- Testing via `actix_web::test`
- Multi-worker model can complicate debugging

**Verdict:** All three have good developer experience. Rocket's error messages are particularly helpful.

### 9. Specific integration compatibility

**datastar-rust integration:**
- **axum:** Full support with `ReadSignals<T>` extractor and `Event` conversion
- **Rocket:** Event conversion only; signals via native `Json<T>` extraction
- **actix-web:** Not supported; would require new adapter (~100 lines)

**hypertext lazy rendering:**
- All three support lazy evaluation via their response traits
- axum: `impl IntoResponse` allows any `Renderable`
- Rocket: `impl Responder` with `respond_to` method
- actix-web: `impl Responder` with similar pattern

**sqlx transaction management:**
- All three support sqlx transactions via `State<Pool>` or equivalent
- axum's extractors allow extracting `Transaction` directly if needed

**rust-embed asset serving:**
- axum: `tower_http::services::ServeDir` (dev) + custom handler (prod)
- Rocket: `FileServer` helper + custom responder
- actix-web: `actix_files::Files` + custom handler

**WebSocket support (future):**
- axum: `axum::extract::ws::WebSocketUpgrade`
- Rocket: `rocket_ws` crate
- actix-web: `actix-web-actors` crate

**Verdict:** datastar-rust's full axum support is a significant advantage. Adding Rocket support would be straightforward but adds maintenance burden.

## Functional programming alignment analysis

ironstar's architecture emphasizes:
- Effects explicit in type signatures, isolated at boundaries
- Algebraic data types (sum types for states, product types for data)
- Reader monad pattern for dependency injection
- Pure functions in domain/application layers

**axum alignment:**

1. *Reader monad pattern*: `State<T>` extraction with `FromRef` directly implements the Reader pattern. Extracting `State<Pool>` is equivalent to `ask` in a Reader monad.

2. *Effect boundaries*: Handler signatures are explicit functions from `(Extractors...) → impl IntoResponse`. All I/O happens at the handler boundary.

3. *Algebraic composition*: Tower `Layer` composition is associative: `layer1.layer(layer2.layer(service))` equals `layer1.layer(layer2).layer(service)`.

4. *Type-driven design*: `FromRequest` trait bounds enforce compile-time guarantees about extraction.

**Rocket alignment:**

1. *Reader pattern*: `&State<T>` guard provides similar semantics but via macros.

2. *Effect boundaries*: Less explicit due to macro-generated code.

3. *Algebraic types*: `Outcome<S, E, F>` is a proper three-way sum type, more expressive than `Result`.

4. *Type-driven*: Sentinel pattern provides compile-time state checks.

**actix-web alignment:**

1. *Reader pattern*: `Data<T>` provides state but with type erasure concerns.

2. *Effect boundaries*: Good but less explicit than axum.

3. *Composition*: Tuple extraction limits algebraic composition.

4. *Type-driven*: `ResponseError` trait uses `Box<dyn>` type erasure.

**Verdict:** axum best embodies functional programming principles with explicit types, algebraic middleware composition, and clean Reader monad semantics.

## Risk assessment

### axum risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking changes in 0.x versions | Medium | Medium | Pin versions, test upgrades |
| Tower ecosystem complexity | Low | Low | Use tower-http helpers |
| Learning curve for tower | Medium | Low | Team training, docs |

### Rocket risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Single maintainer bus factor | Medium | High | Monitor project health |
| Macro debugging difficulty | Medium | Medium | Prefer explicit patterns |
| Smaller ecosystem | Medium | Low | Build custom solutions |
| datastar-rust partial support | Low | Medium | Contribute ReadSignals for Rocket |

### actix-web risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| No SSE support | High | Critical | Build custom SSE layer |
| Runtime incompatibility | Low | Medium | Test tokio integration |
| Type erasure debugging | Medium | Medium | Use custom error types |

## Migration considerations

If Rocket were selected, the following ironstar documentation changes would be required:

1. **CLAUDE.md:** Replace all axum references with Rocket equivalents
2. **architecture-decisions.md:** Update framework choice rationale
3. **backend-core-decisions.md:** Rewrite extractor patterns
4. **session-implementation.md:** Convert middleware to fairings
5. **sse-connection-lifecycle.md:** Update to EventStream! patterns
6. **datastar-request-extractor.md:** Rewrite for Rocket guards
7. **error-handling-decisions.md:** Update for Rocket catchers

Estimated migration effort: 3-5 days of documentation updates.

## Decision rationale

**Final recommendation: axum 0.8+**

The decision is based on three weighted criteria:

1. **datastar-rust integration (40%):** axum has full support including the `ReadSignals<T>` extractor. This eliminates custom integration work and ensures SDK compliance.

2. **Functional programming alignment (35%):** axum's tower-based architecture provides the cleanest mapping to Reader monad patterns and algebraic composition that ironstar's architecture requires.

3. **Ecosystem and maintainability (25%):** axum's tower ecosystem, tokio team maintenance, and active development provide long-term stability.

**Tradeoffs acknowledged:**

- Rocket's `EventStream!` syntax is more concise than axum's `Sse::new(stream)`
- Rocket's fairings provide clearer lifecycle hooks than tower middleware
- actix-web has marginally higher raw throughput (likely irrelevant for ironstar)

**Confidence level:** 90%

The 10% uncertainty accounts for:
- Potential improvements to Rocket's datastar-rust support
- Unknown edge cases in SSE reconnection handling
- Future framework evolution

## Sources

- [Rust Web Frameworks Comparison 2025](https://markaicode.com/rust-web-frameworks-performance-benchmark-2025/)
- [axum SSE documentation](https://docs.rs/axum/latest/axum/response/sse/index.html)
- [Rocket v0.5 release notes](https://rocket.rs/news/2023-11-17-version-0.5/)
- [datastar SDK specification](~/projects/lakescope-workspace/datastar/sdk/ADR.md)
- Local source analysis: axum, Rocket, actix-web, datastar-rust repositories
