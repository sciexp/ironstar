# Discovery and specification

This document connects collaborative domain discovery (EventStorming, Domain Storytelling, Example Mapping) to ironstar's algebraic type system implementation.
Discovery sessions produce concrete artifacts that map directly to Rust types, following patterns from domain-driven design while maintaining ironstar's functional programming principles.

## Purpose

Discovery processes translate business requirements into implementable specifications.
For ironstar, this translation must preserve algebraic properties: domain events discovered during EventStorming become sum type variants, aggregates become functors, and policies become Kleisli arrows.
The workflow ensures that collaborative discovery artifacts trace directly to type signatures and implementation.

## The 8-step DDD Starter Modelling Process

Ironstar follows an adapted version of the DDD Starter Modelling Process with algebraic interpretations at each step.
This process is detailed in `~/.claude/commands/preferences/discovery-process.md`.

| Step | Activity | Algebraic Interpretation | Ironstar Output |
|------|----------|-------------------------|-----------------|
| **1. Align** | Business Model Canvas, impact mapping | Strategic goals as top-level constraints | Core/Supporting/Generic classification |
| **2. Discover** | EventStorming Big Picture | Domain events as free monoid generators | Event enum variants, timeline ordering |
| **3. Decompose** | Bounded context identification | Module boundaries (functorial relationships) | Crate architecture, module organization |
| **4. Connect** | Context mapping | ACL as functors, Open Host as natural transformations | Integration patterns, port traits |
| **5. Strategize** | Core Domain Charts | Type sophistication gradient by strategic value | Smart constructors vs simple ADTs |
| **6. Organize** | Team topologies aligned to contexts | Social structures mirror technical boundaries | Not applicable (template project) |
| **7. Specify** | Type-level specification | Refinement types or dependent types for core domains | Type signatures with invariants |
| **8. Implement** | Rust implementation following specification | ADTs, trait implementations, property tests | Production code |

The discovery process is iterative, not waterfall.
EventStorming sessions (step 2) may reveal new strategic insights requiring return to alignment (step 1).
Implementation (step 8) may uncover complexities requiring additional specification (step 7).

## EventStorming artifact mapping

EventStorming uses color-coded sticky notes to represent different domain concepts.
Each color maps to a specific Rust type in ironstar's architecture.

### Orange stickies: domain events

Domain events discovered during EventStorming become sum type variants.
Events are facts that have already happened, named in past tense.

| EventStorming | Ironstar Type |
|---------------|---------------|
| "Query Started" (orange) | `QuerySessionEvent::QueryStarted { query_id, dataset, sql, chart_config, timestamp }` |
| "Query Completed" (orange) | `QuerySessionEvent::QueryCompleted { query_id, results, duration }` |
| "Query Failed" (orange) | `QuerySessionEvent::QueryFailed { query_id, error, timestamp }` |
| "Todo Created" (orange) | `TodoEvent::Created { id, text, timestamp }` |
| "Todo Completed" (orange) | `TodoEvent::Completed { id, timestamp }` |

Events form a **free monoid** under concatenation.
The only composition is append (no event can be removed or reordered once written).
This property is enforced by the append-only SQLite event store schema.

### Blue stickies: commands

Commands represent user intentions that trigger validation and event emission.
Commands are named in imperative form.

| EventStorming | Ironstar Type |
|---------------|---------------|
| "Execute Query" (blue) | `QuerySessionCommand::StartQuery { dataset, sql, chart_config }` |
| "Cancel Query" (blue) | `QuerySessionCommand::CancelQuery { query_id }` |
| "Complete Todo" (blue) | `TodoCommand::Complete { id }` |
| "Delete Todo" (blue) | `TodoCommand::Delete { id }` |

Commands are functions: `handle_command(&State, Command) -> Result<Vec<Event>, Error>`.
This signature reflects the Kleisli arrow composition in the Result monad.

### Yellow stickies: aggregates

Aggregates are consistency boundaries discovered through event clustering.
During EventStorming, events naturally cluster around entities that enforce invariants.

| EventStorming | Ironstar Module |
|---------------|-----------------|
| "Query Session" (yellow) | `domain::query_session::QuerySessionAggregate` |
| "Todo" (yellow) | `domain::todo::TodoAggregate` |

Aggregates are **functors** with fold/applyEvent catamorphism for state reconstruction.
The `apply_event` function is the unique fold from the initial algebra (empty state) to the final state.

### Purple stickies: policies and process managers

Policies represent reactive business rules: "Whenever X happens, then Y should occur."
Policies become event handlers that emit commands or side effects.

| EventStorming | Pattern |
|---------------|---------|
| "When query completes, invalidate cache" | Event handler subscribing to `QueryCompleted`, calling cache invalidation |
| "When todo completed, notify subscribers" | Event handler subscribing to `TodoCompleted`, broadcasting via Zenoh SSE |
| "When query times out, emit failure event" | Process manager tracking query state, emitting `QueryFailed` on timeout |

Policies are **Kleisli arrows**: `Event -> Effect [Command]`.
The effect wrapper (Future, Result) makes I/O explicit in the type signature.

### Pink stickies: hotspots

Hotspots mark areas requiring additional discovery or technical investigation.
These become implementation concerns captured in smart constructors, validation rules, or architectural decisions.

| EventStorming Hotspot | Ironstar Implementation |
|-----------------------|-------------------------|
| "What if DuckDB query times out?" | `QueryConfig` with configurable timeout, `QueryFailed` event variant |
| "How to prevent SQL injection?" | `SqlQuery::new()` smart constructor validating against safe query patterns |
| "What if SSE connection drops during query?" | SSE reconnection with Last-Event-ID, documented in `../cqrs/sse-connection-lifecycle.md` |

## Example: QuerySession domain discovery

An EventStorming session for ironstar's analytics domain produces a timeline of events and commands.
This example shows how EventStorming artifacts map to implementation.

### EventStorming timeline (left to right)

```
[Execute Query]     [Query Started]     [Results Ready]     [Query Completed]
    (blue)              (orange)            (orange)              (orange)
        |                   |                    |                     |
        v                   v                    v                     v
   +---------+         +--------+          +---------+           +----------+
   | Validate|  --->   | Pending|   --->   |Executing|   --->    |Completed |
   | Command |         | State  |          | State   |           | State    |
   +---------+         +--------+          +---------+           +----------+
                           |
                    [QuerySession]
                       (yellow)
```

### Hotspots identified (pink)

- **Query timeout**: DuckDB queries against large datasets may exceed reasonable response time
  - **Resolution**: Add timeout configuration, emit `QueryFailed` event on timeout
- **SQL injection**: User-provided SQL must be validated
  - **Resolution**: `SqlQuery::new()` smart constructor validating against parameterized query patterns
- **Concurrent queries**: What happens if user executes second query while first is running?
  - **Resolution**: Aggregate enforces invariant: only one active query per session

### Policies identified (purple)

- **On QueryStarted** → Send SSE signal update to subscribers with pending state
- **On QueryCompleted** → Invalidate moka cache entry for dataset + query hash
- **On QueryCompleted** → Send SSE patch with chart data and final state
- **On QueryFailed** → Send SSE patch with error message to user

### Rust implementation

```rust
// Events (from orange stickies)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuerySessionEvent {
    QueryStarted {
        query_id: QueryId,
        dataset: DatasetRef,
        sql: SqlQuery,
        chart_config: Option<ChartConfig>,
        timestamp: DateTime<Utc>,
    },
    QueryCompleted {
        query_id: QueryId,
        results: QueryResults,
        duration: Duration,
        timestamp: DateTime<Utc>,
    },
    QueryFailed {
        query_id: QueryId,
        error: String,
        timestamp: DateTime<Utc>,
    },
}

// Commands (from blue stickies)
#[derive(Debug, Clone)]
pub enum QuerySessionCommand {
    StartQuery {
        dataset: DatasetRef,
        sql: SqlQuery,
        chart_config: Option<ChartConfig>,
    },
    CancelQuery {
        query_id: QueryId,
    },
}

// Aggregate (from yellow sticky)
pub struct QuerySessionAggregate;

impl QuerySessionAggregate {
    pub fn handle_command(
        state: &QuerySessionState,
        cmd: QuerySessionCommand,
    ) -> Result<Vec<QuerySessionEvent>, QueryError> {
        match cmd {
            QuerySessionCommand::StartQuery { dataset, sql, chart_config } => {
                // Hotspot: enforce single active query invariant
                if state.has_active_query() {
                    return Err(QueryError::QueryAlreadyRunning);
                }

                Ok(vec![QuerySessionEvent::QueryStarted {
                    query_id: QueryId::new(),
                    dataset,
                    sql,
                    chart_config,
                    timestamp: Utc::now(),
                }])
            }
            QuerySessionCommand::CancelQuery { query_id } => {
                if state.current_query_id != Some(query_id) {
                    return Err(QueryError::NoSuchQuery);
                }
                Ok(vec![QuerySessionEvent::QueryFailed {
                    query_id,
                    error: "Cancelled by user".to_string(),
                    timestamp: Utc::now(),
                }])
            }
        }
    }
}
```

## Strategic domain classification

Discovery outputs must be classified for investment decisions.
Not all domains justify equal type sophistication.
The Core/Supporting/Generic classification guides implementation rigor.

| Domain | Classification | Type Sophistication | Discovery Evidence |
|--------|---------------|---------------------|-------------------|
| QuerySession | **Core** | Smart constructors, state machines, full ES, property tests | Primary differentiating capability |
| Analytics Cache | **Core** | Refined cache key types, TTL invariants, invalidation patterns | Performance-critical path |
| Todo | **Generic Example** | Simple ADTs, minimal validation | Stock pattern demonstration |
| Event sourcing infra | **Generic** | Trait abstractions, adapters, reusable patterns | Foundation library |
| Session management | **Supporting** | Standard session types, cookie security | Necessary but not differentiating |
| OAuth authentication | **Supporting** | Standard OAuth flow, provider adapters | Required infrastructure |

**Investment guidelines:**

- **Core domains**: Justify dependent types, formal verification, extensive property-based testing, comprehensive documentation
- **Supporting domains**: Use smart constructors and ADTs, standard validation patterns, integration testing
- **Generic domains**: Use library wrappers, simple types, minimal custom logic

For complete classification methodology, see `~/.claude/commands/preferences/strategic-domain-analysis.md`.

## Linking discovery to implementation

Each implementation artifact should trace back to discovery sessions.
This traceability enables reasoning about why types exist and what business rules they encode.

### Aggregate implementation checklist

When implementing an aggregate discovered during EventStorming:

- [ ] Reference EventStorming session date and artifacts (Miro board link, photo of physical stickies)
- [ ] Yellow sticky → module boundary documented in code comments
- [ ] Orange stickies → Event enum variants complete with all fields
- [ ] Blue stickies → Command enum variants complete
- [ ] Hotspots → validation rules in smart constructors or error types
- [ ] Policies → event handlers registered in application layer
- [ ] Invariants → enforced in `handle_command` validation logic

### Beads issue template

When creating implementation issues from discovery, include discovery metadata.
This links the ticket graph to the domain model evolution.

```markdown
## Discovery origin
- Session: 2025-01-15 EventStorming (QuerySession domain)
- Participants: [names if multi-person session, "solo" if individual]
- Artifacts: [Miro board link or path to photos]
- Yellow sticky: QuerySession aggregate
- Related hotspot: Query timeout handling

## Algebraic grounding
- semantic-model.md section: "Command side Kleisli composition"
- Hoffman Law: Law 7 (work is a side effect — query execution is async effect)
- Domain classification: Core (analytics is primary differentiator)

## Implementation tasks
- [ ] Define `QuerySessionEvent::QueryFailed` variant
- [ ] Add timeout configuration to `QueryConfig`
- [ ] Implement timeout tracking in query executor
- [ ] Add property test for timeout behavior
```

Use `bd create` to create the issue with this template, then wire dependencies with `bd dep add`.
See `~/.claude/commands/issues/beads-prime.md` for command reference.

## Discovery-driven type evolution

As the domain model evolves through implementation, discovery artifacts may require refinement.
This is normal and expected.

**Workflow:**

1. Discover domain through EventStorming or Domain Storytelling
2. Translate to Rust types (events, commands, aggregates)
3. Implement and test
4. Identify gaps or inconsistencies
5. Return to discovery: refine model with new insights
6. Update types and tests
7. Document what changed and why

**Example evolution:**

Initial discovery: "Query Started" event with dataset string.

```rust
QueryStarted {
    dataset: String,  // Too loose
    sql: String,      // Allows SQL injection
}
```

After implementation attempt, hotspots emerge: invalid dataset paths, SQL injection risk.

Refined discovery: introduce value objects with smart constructors.

```rust
QueryStarted {
    dataset: DatasetRef,  // Smart constructor validates path
    sql: SqlQuery,        // Smart constructor validates query safety
}
```

This evolution is documented in commit messages and beads issues linking back to the discovery session that revealed the gap.

## From specification to implementation

Once discovery produces type signatures, implementation follows type-driven development.

### Type-driven workflow (Wlaschin pattern)

1. **Workflows drive types** — Business process identified in discovery (e.g., execute analytics query)
2. **Types drive implementation** — Define signatures first, implement with `todo!()` placeholders
3. **Effects at boundaries** — Pure aggregate logic, async I/O isolated to application layer
4. **Make illegal states unrepresentable** — Use sum types to encode state machine invariants

**Example: QuerySession workflow**

Discovery identifies workflow: User selects dataset → enters SQL query → executes → receives chart results.

Type signature from workflow:

```rust
async fn execute_query_workflow<S>(
    services: &S,
    session_id: SessionId,
    dataset: DatasetRef,
    sql: SqlQuery,
    chart_config: Option<ChartConfig>,
) -> Result<(), WorkflowError>
where
    S: HasEventStore + HasAnalytics + HasEventBus,
{
    // 1. Load current state
    let state = load_query_session_state(services, &session_id).await?;

    // 2. Execute command (pure)
    let cmd = QuerySessionCommand::StartQuery { dataset, sql, chart_config };
    let events = QuerySessionAggregate::handle_command(&state, cmd)?;

    // 3. Persist events
    for event in &events {
        services.event_store().append(event).await?;
    }

    // 4. Publish for projections and SSE
    for event in events {
        services.event_bus().publish(event).await?;
    }

    Ok(())
}
```

This signature explicitly separates pure domain logic (`handle_command`) from effects (database, pub/sub).

## Discovery and the semantic model

Each discovery artifact has a denotation in the semantic model documented in `semantic-model.md`.
Understanding these mappings connects collaborative discovery to mathematical foundations.

| Discovery Artifact | Semantic Denotation |
|--------------------|---------------------|
| Event timeline (orange stickies) | Free monoid over Event type |
| Aggregate (yellow sticky) | Coalgebra for state functor |
| Command (blue sticky) | Kleisli arrow in Result monad |
| Policy (purple sticky) | Kleisli arrow in Effect monad |
| Projection | Catamorphism (fold over events) |
| SSE stream | Deterministic function Event → Patch |

For complete categorical grounding, see `semantic-model.md`.

## Related documentation

### Discovery methodology

- `~/.claude/commands/preferences/discovery-process.md` — Complete 8-step DDD process
- `~/.claude/commands/preferences/collaborative-modeling.md` — EventStorming facilitation patterns
- `~/.claude/commands/preferences/strategic-domain-analysis.md` — Core/Supporting/Generic classification

### Ironstar architecture

- `semantic-model.md` — Categorical denotations for all architectural concepts
- `architecture-decisions.md` — Technology choices with strategic rationale
- `design-principles.md` — Foundational principles (effect boundaries, pure aggregates)
- `crate-architecture.md` — Multi-crate decomposition and scaling path

### Domain implementation

- `~/.claude/commands/preferences/domain-modeling.md` — Smart constructors, aggregates, value objects
- `~/.claude/commands/preferences/event-sourcing.md` — Hoffman's Laws, theoretical synthesis
- `../cqrs/event-sourcing-core.md` — Event store schema, aggregate patterns
- `../cqrs/command-write-patterns.md` — Command validation, optimistic locking
