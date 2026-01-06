# Northstar SSE patterns for Idris2 formalization

**Status**: Extracted from northstar Go template for Task .14
**Purpose**: Identify effect boundaries and type signatures for Idris2 formal specification
**Source**: `~/projects/lakescope-workspace/datastar-go-nats-template-northstar/`

## Type signatures (language-agnostic)

### Core SSE streaming abstraction

```
type SessionID = String
type AggregateID = String
type EventID = Int64  -- Monotonic global sequence

-- Pure state transformer
type Decide : Command → State → List Event
type Evolve : State → Event → State

-- Effect boundary: I/O at edges
type AppendEvent : Event → IO EventID
type PublishEvent : Event → IO ()
type WatchKey : KeyPattern → IO (Stream Event)

-- SSE projection function (total, deterministic)
type EventToPatch : Event → SSEPatch
type PatchToBytes : SSEPatch → ByteString

-- Connection lifecycle
type SSEConnection =
  { lastEventID : Maybe EventID
  , sessionID : SessionID
  , subscriber : Stream Event
  , clientAddr : IPAddr
  }

-- Server-side event loop (effect at boundary)
type SSEHandler : SSEConnection → IO (Stream SSEPatch)
```

### Northstar concrete patterns extracted

**Command handling** (index/handlers.go:175-204):
```
ReadSignals[T] : HTTPRequest → IO (Result T Error)
GetSessionMVC : (HTTPRequest, HTTPResponse) → IO (SessionID, MVC)
SaveMVC : (Context, SessionID, MVC) → IO ()
```

**SSE streaming** (index/handlers.go:32-71):
```
-- Pure logic: commands mutate state, emit events
ToggleTodo : (MVC, Index) → MVC
EditTodo : (MVC, Index, Text) → MVC
DeleteTodo : (MVC, Index) → MVC

-- Effect boundary: NATS KV watch → SSE stream
WatchUpdates : (Context, SessionID) → IO KeyWatcher
KeyWatcher.Updates() : () → Channel (Maybe Entry)

-- SSE event emission (effect)
PatchElementTempl : Template → IO ()
ConsoleError : Error → IO ()
```

**NATS JetStream KV** (services/todo_service.go:35-51, 86-88):
```
-- Storage effect
CreateOrUpdateKeyValue : KeyValueConfig → IO KeyValue
KeyValue.Get : (Context, Key) → IO (Result Entry Error)
KeyValue.Put : (Context, Key, Bytes) → IO ()
KeyValue.Watch : (Context, Key) → IO KeyWatcher

-- Watch semantics: emits current value + future updates
-- Critically: watch established BEFORE historical replay
```

**Counter atomic updates** (counter/handlers.go:88-103):
```
-- Pure increment (referentially transparent for given state)
globalCounter.Add(1) : () → Uint32

-- Effect boundary: session state
getUserValue : HTTPRequest → IO (Uint32, Session, Error)
Session.Save : (HTTPRequest, HTTPResponse) → IO ()
```

**Monitor SSE with tickers** (monitor/handlers.go:32-81):
```
-- Periodic sampling (effect)
time.NewTicker : Duration → Ticker
Ticker.C : Channel Time

-- select! multiplexing
select {
  case <-ctx.Done(): return           -- Client disconnect
  case <-memT.C: emit memory stats    -- Periodic update
  case <-cpuT.C: emit CPU stats       -- Periodic update
}
```

## Invariants → Dependent type candidates

### Monotonicity invariants

1. **EventID ordering**: `∀ e1 e2. timestamp(e1) < timestamp(e2) ⇒ eventID(e1) < eventID(e2)`
   - Type: `MonotonicSequence : List Event → Type`
   - Enforces append-only log with strict ordering
   - Maps to SQLite `AUTOINCREMENT` or PostgreSQL `BIGSERIAL`

2. **Last-Event-ID resumption**: `replay(lastEventID) = filter (λe. eventID(e) > lastEventID) allEvents`
   - Type: `ReplayFrom : EventID → List Event → List Event`
   - Property: `length (replay id) ≤ length allEvents`
   - Idempotent replay: `replay id . replay id = replay id`

3. **Signal update idempotence**: Multiple `PatchSignals` with same data produce identical DOM state
   - Type: `SignalPatch : Signals → Signals → Signals` (merge function)
   - Property: `patch s s = s` (reflexive)
   - Property: `patch s1 (patch s1 s2) = patch s1 s2` (idempotent for same base)

### NATS KV watch semantics

4. **Watch ordering**: `watch(key)` emits current value, then future updates in order
   - Type: `Watch : Key → IO (Current, Stream Update)`
   - Invariant: `current` reflects state at watch establishment time
   - Subsequent updates arrive in NATS sequence order

5. **Session isolation**: `watch(sessionA)` never receives `update(sessionB)`
   - Type: `Isolated : SessionID → KeyPattern → Type`
   - Enforced by key prefix: `events/session/{id}/**`
   - Zenoh/NATS server-side filtering preserves isolation

### Effect sequencing

6. **Append-then-publish**: Event must be persisted before notification
   - Type: `AppendThenPublish : Event → IO ()`
   - Implementation: `appendEvent e >>= \eid → publishEvent eid e`
   - Prevents lost events if publish fails before persistence

7. **Subscribe-before-replay**: Subscription established before historical replay prevents race
   - Type: `SubscribeThenReplay : KeyPattern → Maybe EventID → IO (Stream Event)`
   - Critical in index/handlers.go:43-48 (watch before GetSessionMVC)
   - Prevents missing events that arrive during replay window

## Effect boundaries (precise mapping)

### Server boundary (Go handler → ironstar axum)

**Northstar Go**:
```go
func (h *Handlers) TodosSSE(w http.ResponseWriter, r *http.Request) {
    sessionID, mvc, err := h.todoService.GetSessionMVC(w, r)  // [EFFECT 1]
    sse := datastar.NewSSE(w, r)                              // [EFFECT 2]
    watcher, err := h.todoService.WatchUpdates(ctx, sessionID) // [EFFECT 3]

    for {
        select {
        case <-ctx.Done():     // [EFFECT 4: Client disconnect]
            return
        case entry := <-watcher.Updates():  // [EFFECT 5: NATS KV update]
            json.Unmarshal(entry.Value(), mvc)  // [PURE]
            c := components.TodosMVCView(mvc)   // [PURE: hypertext render]
            sse.PatchElementTempl(c)            // [EFFECT 6: SSE write]
        }
    }
}
```

**Ironstar Rust** (axum + zenoh):
```rust
async fn todos_sse(
    State(app_state): State<AppState>,        // [EFFECT 1: Extract state]
    session: SessionExtractor,                // [EFFECT 2: Read session cookie]
    headers: HeaderMap,                       // [EFFECT 3: Last-Event-ID]
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let last_event_id = parse_last_event_id(&headers);  // [PURE]

    // [EFFECT 4: Subscribe to Zenoh key expression]
    let subscriber = app_state.zenoh_session
        .declare_subscriber(format!("events/session/{}", session.id))
        .await?;

    // [EFFECT 5: Replay historical events from SQLite]
    let historical = sqlx::query_as::<_, Event>("SELECT ...")
        .fetch_all(&app_state.db)
        .await?;

    // [PURE: Map events to SSE patches]
    let stream = futures::stream::select(
        futures::stream::iter(historical.into_iter().map(event_to_patch)),
        subscriber.map(|sample| event_to_patch(sample.payload))
    );

    Sse::new(stream)  // [EFFECT 6: Upgrade to SSE stream]
}

// [PURE: Total function, no I/O]
fn event_to_patch(event: Event) -> datastar::Event {
    let html = render_todo_view(&event);  // [PURE: hypertext lazy thunk]
    datastar::Event::patch_element(html)
}
```

**Effect boundary placement**:

| Layer | Go (northstar) | Rust (ironstar) | Effect Type |
|-------|----------------|-----------------|-------------|
| HTTP request | `w http.ResponseWriter, r *http.Request` | `HeaderMap, SessionExtractor` | Read |
| Session state | `GetSessionMVC(w, r)` | `SessionExtractor` (axum middleware) | Read |
| Subscribe | `WatchUpdates(ctx, sessionID)` → `KeyWatcher` | `zenoh_session.declare_subscriber(key)` | Subscribe |
| Historical replay | Implicit in NATS KV `Watch()` (emits current) | Explicit SQLite query | Read |
| Event loop | `select { case entry := <-watcher.Updates() }` | `futures::stream::select(historical, live)` | Multiplex |
| Render HTML | `components.TodosMVCView(mvc)` (Templ) | `render_todo_view(&event)` (hypertext) | Pure |
| SSE write | `sse.PatchElementTempl(c)` | `Sse::new(stream)` (axum) | Write |
| Cleanup | `defer watcher.Stop()` | Drop impl for `Subscriber` | Close |

### Client boundary (Browser EventSource → datastar.js)

**Browser SSE client** (native EventSource API):
```javascript
// [EFFECT: Establish connection]
const eventSource = new EventSource('/api/todos/sse', {
  withCredentials: true  // Send session cookie
});

// [EFFECT: Register listener]
eventSource.addEventListener('datastar-patch', (event) => {
  const patch = JSON.parse(event.data);  // [PURE: deserialize]
  applyPatch(patch);                     // [EFFECT: DOM update]
});

// [EFFECT: Automatic reconnection with Last-Event-ID]
// Browser sends Last-Event-ID: <id> header on reconnect
```

**Datastar signal updates** (reactive):
```javascript
// Server sends: data: {"user": 42, "global": 100}
// [EFFECT: Signal write triggers reactivity]
store.signals.merge(patch.signals);

// [EFFECT: DOM patch via Idiomorph]
Idiomorph.morph(targetElement, patch.html, {
  callbacks: { beforeNodeMorphed: (from, to) => !to.hasAttribute('data-ignore-morph') }
});
```

**Effect isolation**:
- **Pure**: JSON parsing, HTML string manipulation
- **Effect**: Network I/O (SSE connection), DOM updates (signal writes, morphing)
- **Boundary**: `applyPatch` is the single entry point for all DOM effects

### Pub/sub boundary (NATS JetStream KV → Zenoh)

**NATS JetStream KV** (northstar):
```go
// [EFFECT: Create or open KV bucket]
kv, err := js.CreateOrUpdateKeyValue(ctx, jetstream.KeyValueConfig{
    Bucket: "todos",
    TTL: time.Hour,
})

// [EFFECT: Write with revision check (optimistic locking)]
_, err := kv.Put(ctx, sessionID, jsonBytes)

// [EFFECT: Watch with historical replay]
watcher, err := kv.Watch(ctx, sessionID)
// Emits: current value (if exists), then future updates

// [EFFECT: Blocking read from channel]
for entry := range watcher.Updates() {
    // Process entry
}
```

**Zenoh** (ironstar):
```rust
// [EFFECT: Configure embedded mode (no networking)]
let mut config = zenoh::config::Config::default();
config.insert_json5("listen/endpoints", "[]")?;
config.insert_json5("connect/endpoints", "[]")?;
let session = zenoh::open(config).await?;

// [EFFECT: Publish event notification]
session.put(format!("events/session/{}", session_id), event_bytes).await?;

// [EFFECT: Subscribe with key expression filtering]
let subscriber = session
    .declare_subscriber(format!("events/session/{}/**", session_id))
    .await?;

// [EFFECT: Async stream]
while let Ok(sample) = subscriber.recv_async().await {
    let event = deserialize(sample.payload)?;  // [PURE]
    emit_sse_patch(event)?;                    // [EFFECT]
}
```

**Critical difference**:
- **NATS KV**: `Watch()` emits current value automatically (built-in historical replay)
- **Zenoh**: Pub/sub only; historical replay requires explicit SQLite query
- **ironstar pattern**: `futures::stream::select(historical_from_sqlite, live_from_zenoh)`

## Composition patterns

### Pattern 1: Session-scoped event routing

**Northstar** (NATS KV per session):
```go
// Key: sessionID (flat key, no hierarchy)
kv.Watch(ctx, sessionID)
```

**Ironstar** (Zenoh key expressions):
```rust
// Hierarchical key with wildcards
format!("events/session/{}/Todo/**", session_id)
format!("events/session/{}/User/**", session_id)

// Multi-pattern subscription with tokio::select!
let todo_sub = session.declare_subscriber(todo_key).await?;
let user_sub = session.declare_subscriber(user_key).await?;

loop {
    tokio::select! {
        Ok(sample) = todo_sub.recv_async() => handle_todo(sample),
        Ok(sample) = user_sub.recv_async() => handle_user(sample),
        _ = ctx.cancelled() => break,
    }
}
```

### Pattern 2: Command → State → Event → Notification

**Pure core** (identical in Go and Rust):
```
1. Validate command          [PURE: Command → Result ValidationError]
2. Load current state        [EFFECT: StateRepository.get(id)]
3. Decide on events          [PURE: Decider.decide(cmd, state) → List Event]
4. Evolve state              [PURE: foldl Decider.evolve state events]
5. Persist events            [EFFECT: EventStore.append(events)]
6. Publish notifications     [EFFECT: EventBus.publish(events)]
7. Return success            [PURE: Ok(())]
```

**Northstar implementation** (index/handlers.go:133-154):
```go
func (h *Handlers) ToggleTodo(w http.ResponseWriter, r *http.Request) {
    sessionID, mvc, _ := h.todoService.GetSessionMVC(w, r)  // [2: EFFECT]
    i, _ := h.parseIndex(w, r)                              // [1: PURE]

    h.todoService.ToggleTodo(mvc, i)                        // [3+4: PURE]
    h.todoService.SaveMVC(r.Context(), sessionID, mvc)      // [5+6: EFFECT]
}

// Pure business logic
func (s *TodoService) ToggleTodo(mvc *TodoMVC, index int) {
    if index < len(mvc.Todos) {
        mvc.Todos[index].Completed = !mvc.Todos[index].Completed
    }
}

// Effect: persist + publish
func (s *TodoService) SaveMVC(ctx context.Context, sessionID string, mvc *TodoMVC) error {
    b, _ := json.Marshal(mvc)
    _, err := s.kv.Put(ctx, sessionID, b)  // Triggers Watch() listeners
    return err
}
```

**Ironstar equivalent** (fmodel-rust Decider pattern):
```rust
// [3: PURE - Decider.decide]
fn decide(cmd: TodoCommand, state: &TodoState) -> Vec<TodoEvent> {
    match cmd {
        TodoCommand::Toggle { id } => {
            if state.todos.contains_key(&id) {
                vec![TodoEvent::Toggled { id }]
            } else {
                vec![]
            }
        }
    }
}

// [4: PURE - Decider.evolve]
fn evolve(state: TodoState, event: &TodoEvent) -> TodoState {
    match event {
        TodoEvent::Toggled { id } => {
            let mut new_state = state.clone();
            if let Some(todo) = new_state.todos.get_mut(id) {
                todo.completed = !todo.completed;
            }
            new_state
        }
    }
}

// [5+6: EFFECT - EventSourcedAggregate]
async fn handle_command(cmd: TodoCommand) -> Result<(), Error> {
    let state = event_repository.load(cmd.aggregate_id()).await?;  // [2: EFFECT]
    let events = decide(cmd, &state);                              // [3: PURE]
    let new_state = events.iter().fold(state, |s, e| evolve(s, e)); // [4: PURE]

    event_repository.append(events).await?;                        // [5: EFFECT]
    for event in &events {
        zenoh_session.put(event_key(event), event.to_bytes()).await?; // [6: EFFECT]
    }
    Ok(())
}
```

### Pattern 3: SSE reconnection with historical replay

**Northstar** (implicit via NATS KV Watch semantics):
```go
// Watch() automatically emits current value, then future updates
watcher, _ := kv.Watch(ctx, sessionID)
for entry := range watcher.Updates() {
    // First entry is current value (if key exists)
    // Subsequent entries are future updates
}
```

**Ironstar** (explicit via Last-Event-ID):
```rust
// [EFFECT: Parse Last-Event-ID from request headers]
let last_event_id = headers
    .get("Last-Event-ID")
    .and_then(|v| v.to_str().ok())
    .and_then(|s| s.parse::<i64>().ok());

// [EFFECT: Subscribe BEFORE replay]
let subscriber = zenoh_session
    .declare_subscriber(format!("events/session/{}", session_id))
    .await?;

// [EFFECT: Replay historical events]
let historical = sqlx::query_as::<_, Event>(
    "SELECT * FROM events WHERE session_id = ? AND id > ? ORDER BY id"
)
.bind(&session_id)
.bind(last_event_id.unwrap_or(0))
.fetch_all(&db)
.await?;

// [PURE: Merge streams]
let stream = futures::stream::select(
    futures::stream::iter(historical),
    subscriber.into_stream()
);

Sse::new(stream.map(|event| Ok(event_to_patch(event))))
```

**Critical race condition prevention**:
- Subscribe to Zenoh **before** querying SQLite
- Events arriving during SQLite query are buffered in Zenoh subscriber channel
- No events lost between historical replay and live stream
- Northstar achieves same via NATS KV Watch semantics (built-in)

### Pattern 4: Multiplexing multiple event sources (monitor example)

**Northstar** (monitor/handlers.go:32-81):
```go
func (h *Handlers) MonitorEvents(w http.ResponseWriter, r *http.Request) {
    memT := time.NewTicker(time.Second)
    defer memT.Stop()

    cpuT := time.NewTicker(time.Second)
    defer cpuT.Stop()

    sse := datastar.NewSSE(w, r)
    for {
        select {
        case <-r.Context().Done():
            return
        case <-memT.C:
            vm, _ := mem.VirtualMemory()
            memStats := pages.SystemMonitorSignals{...}
            sse.MarshalAndPatchSignals(memStats)
        case <-cpuT.C:
            cpuTimes, _ := cpu.Times(false)
            cpuStats := pages.SystemMonitorSignals{...}
            sse.MarshalAndPatchSignals(cpuStats)
        }
    }
}
```

**Ironstar equivalent** (tokio::select! with interval streams):
```rust
use tokio::time::{interval, Duration};
use futures::stream::{StreamExt, select_all};

async fn monitor_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut mem_interval = interval(Duration::from_secs(1));
    let mut cpu_interval = interval(Duration::from_secs(1));

    let stream = async_stream::stream! {
        loop {
            tokio::select! {
                _ = mem_interval.tick() => {
                    let stats = get_memory_stats().await;  // [EFFECT]
                    let patch = datastar::Event::patch_signals(stats);  // [PURE]
                    yield Ok(patch);
                }
                _ = cpu_interval.tick() => {
                    let stats = get_cpu_stats().await;  // [EFFECT]
                    let patch = datastar::Event::patch_signals(stats);  // [PURE]
                    yield Ok(patch);
                }
            }
        }
    };

    Sse::new(stream)
}
```

## Cross-reference findings (ironstar docs vs northstar code)

### ✓ Validated patterns (ironstar docs match northstar)

1. **SSE connection lifecycle** (`sse-connection-lifecycle.md`):
   - ✓ Last-Event-ID header extraction (lines 70-73 vs index/handlers.go:43)
   - ✓ ConnectInfo extractor for client IP (lines 92-95 vs monitor logging)
   - ✓ Subscribe-before-replay pattern (lines 98-100 vs index/handlers.go:43-48)

2. **ds-echarts integration** (`ds-echarts-integration-guide.md`):
   - ✓ Light DOM for CSS token inheritance (line 27 vs ds-echarts.ts:27-29)
   - ✓ Lit lifecycle hooks (firstUpdated, updated, disconnectedCallback)
   - ✓ Event sanitization for SSE (line 295-311 pattern documented)

3. **Zenoh vs NATS patterns** (`zenoh-event-bus.md`):
   - ✓ Embedded mode configuration (lines 36-44 match ironstar's no-network requirement)
   - ✓ Key expression filtering (lines 62-79 document wildcard semantics)
   - ✓ Multi-pattern subscriptions with select! (lines 76-79)

### ⚠ Gaps requiring ironstar documentation updates

1. **NATS KV Watch automatic current-value emission missing**:
   - **Northstar**: `kv.Watch(ctx, key)` emits current value first, then updates (services/todo_service.go:87)
   - **Ironstar docs**: `sse-connection-lifecycle.md` lines 98-100 describe subscribe-before-replay but don't note that Zenoh requires explicit SQLite query where NATS provides this automatically
   - **Action**: Add explicit note in `sse-connection-lifecycle.md` Phase 2 about NATS KV vs Zenoh semantic difference

2. **Session-scoped key patterns underspecified**:
   - **Northstar**: Uses flat key `sessionID` for NATS KV (services/todo_service.go:87, 147)
   - **Ironstar docs**: `zenoh-event-bus.md` lines 93-99 show `events/{type}/{id}` but don't document session-scoped pattern `events/session/{session_id}/**` for per-session SSE feeds
   - **Action**: Add session-scoped key expression section to `zenoh-event-bus.md`

3. **Command validation patterns missing**:
   - **Northstar**: index/handlers.go:106-131 shows SetMode validation (lines 119-124: reject invalid enum values)
   - **Ironstar docs**: `event-sourcing-core.md` covers Decider pattern but lacks validation-before-decide pattern
   - **Action**: Add validation section to `event-sourcing-core.md` showing `validate : Command → Result ValidationError` before `decide`

4. **Error propagation to SSE client missing**:
   - **Northstar**: Uses `sse.ConsoleError(err)` to send errors to browser console (index/handlers.go:64, 91, 99)
   - **Ironstar docs**: `sse-connection-lifecycle.md` doesn't document error event emission pattern
   - **Action**: Add error handling section showing `datastar::Event::console_error()` usage

5. **Session creation/upsert pattern missing**:
   - **Northstar**: services/todo_service.go:165-182 shows session ID generation and cookie upsert
   - **Ironstar docs**: `session-management.md` covers session cookies but lacks new-session-creation handler pattern
   - **Action**: Add session creation section to `session-implementation.md`

6. **Atomic state mutations with read-modify-write missing**:
   - **Northstar**: counter/handlers.go:62-86 shows `getUserValue → increment → Save` pattern with gorilla sessions
   - **Ironstar docs**: Assumes event sourcing for all state; lacks simple read-modify-write patterns for non-event-sourced state (like session counters)
   - **Action**: Add "Non-event-sourced state" section to architecture decisions showing when to use sessions vs events

### ⚠ Northstar patterns ironstar should NOT adopt

1. **In-memory state mutations** (counter/handlers.go:20, 101-103):
   - Northstar uses `atomic.Uint32` for global counter (not persisted)
   - Lost on server restart
   - Ironstar should use event-sourced counter or SQLite for persistence

2. **Mixed state management** (counter uses both sessions AND in-memory atomic):
   - User counter: session-based (persistent across restarts)
   - Global counter: in-memory atomic (ephemeral)
   - Creates inconsistent persistence semantics
   - Ironstar should unify: either all event-sourced or explicit ephemeral annotation

3. **Implicit MVC serialization** (services/todo_service.go:60-74):
   - Northstar stores entire `TodoMVC` struct in NATS KV as JSON
   - No event sourcing: `SaveMVC` overwrites previous state
   - Ironstar uses event sourcing: store events, project MVC view from events

## Idris2 formalization questions (for Task .17)

### Type parameter choices

1. **Phantom types for effect tracking**:
   - Should `SSEHandler` use indexed monads to track effect types?
   - Example: `SSEHandler : (effects : List Effect) → ConnectionState → Type`
   - Alternatively: leverage Idris2 `IO` monad without additional tracking?

2. **Dependent pairs for event replay consistency**:
   - Represent `(lastEventID, events)` as dependent pair where `events = filter (>lastEventID)`?
   - Type: `ReplayContext : (last : EventID) → (events : List Event) → (all e in events, eventID e > last) → Type`
   - Ensures type-level guarantee that replay is correctly filtered

3. **Session isolation via refinement types**:
   - Encode session ownership in event keys?
   - Type: `SessionEvent : (session : SessionID) → Event → Type` where `key event = "events/session/" ++ session ++ "/..."`
   - Prevents cross-session event leakage at type level

### Effect placement

1. **Pure rendering vs effectful SSE write**:
   - Northstar: `TodosMVCView(mvc)` (pure Templ render) → `sse.PatchElementTempl(c)` (effect)
   - Ironstar: `render_todo_view(&event)` (pure hypertext thunk) → `Sse::new(stream)` (effect)
   - **Question**: Should Idris2 model distinguish lazy thunk creation (pure) from thunk forcing (effect)?
   - Or treat HTML rendering as always pure since hypertext thunks don't perform I/O?

2. **Subscription vs replay ordering**:
   - Northstar: NATS KV Watch emits current then updates (single effect, built-in ordering)
   - Ironstar: Explicit `subscribe → replay → merge` (three effects, manual ordering)
   - **Question**: Model as single `SubscribeWithReplay` effect or composition `Subscribe >>= Replay >>= Merge`?
   - Composition exposes race condition risk; single effect hides it. Which is more honest?

3. **Event append vs publish atomicity**:
   - Current pattern: `appendEvent >> publishEvent` (two effects, non-atomic)
   - Risk: append succeeds, publish fails → event persisted but not notified
   - **Question**: Should type system enforce transactional append+publish?
   - Type: `AppendAndPublish : Event → IO (Either Error EventID)` (single atomic effect)
   - Or accept eventual consistency and model as separate effects?

### Composition semantics

1. **Stream fusion for SSE pipeline**:
   - Northstar: `watch → unmarshal → render → emit` as composed effects
   - Ironstar: `Stream Event → Stream Patch` via pure map, then `Sse::new(stream)` (effect)
   - **Question**: Should Idris2 model optimize fused pipelines as single effect?
   - Or preserve compositional structure for reasoning?

2. **Multiplexing with select!**:
   - Pattern: `tokio::select! { A => x, B => y }` (non-deterministic choice)
   - **Question**: Model as `Select : IO a → IO b → IO (Either a b)` with non-determinism?
   - Or use `Stream (Either a b)` to make interleaving explicit?

3. **Backpressure and buffering**:
   - Zenoh subscriber buffers events during processing
   - If SSE client is slow, buffer grows unbounded
   - **Question**: Model buffer size in type? `Subscriber : (capacity : Nat) → KeyPattern → IO (BoundedStream Event)`
   - Or leave as operational concern outside formal model?

### Algebraic properties to verify

1. **SSE event projection is total**:
   - Conjecture: `∀ e : Event. ∃! p : Patch. eventToPatch e = p`
   - Every domain event has exactly one SSE patch representation
   - **Verify**: No partial pattern matches, all event types covered

2. **Replay idempotence**:
   - Property: `replay id . replay id = replay id`
   - Re-replaying from same `lastEventID` produces same events
   - **Verify**: `filter (>id) . filter (>id) = filter (>id)` (filter idempotence)

3. **Session isolation is a partition**:
   - Property: `∀ s1 s2 : SessionID. s1 ≠ s2 ⇒ events(s1) ∩ events(s2) = ∅`
   - Sessions partition the event space (no overlap)
   - **Verify**: Key expression `events/session/{id}/**` enforces disjoint sets

4. **Monotonic sequence forms total order**:
   - Property: `∀ e1 e2 : Event. eventID(e1) ≠ eventID(e2) ⇒ (eventID(e1) < eventID(e2) ∨ eventID(e2) < eventID(e1))`
   - Event IDs are strictly ordered (no ties, no gaps in ordering)
   - **Verify**: SQLite `AUTOINCREMENT` semantics preserve total order

5. **Subscribe-before-replay prevents event loss**:
   - Property: `subscribe t1 >> replay t2 ⇒ events [t1, ∞) ⊆ received` where `t1 < t2`
   - All events from subscription time onward are received
   - **Verify**: Subscription buffer captures events during replay, no race window
