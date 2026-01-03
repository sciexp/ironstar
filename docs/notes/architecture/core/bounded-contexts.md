# Bounded context design

This document describes bounded context patterns for ironstar, covering context identification, relationship types, and integration strategies.
While ironstar v1 operates as a single bounded context, this document provides the foundation for future decomposition.

## Current context structure

Ironstar v1 contains implicit context boundaries within a single deployable unit:

```
┌─────────────────────────────────────────────────────────────────┐
│                     Ironstar Monolith                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Session    │  │     Todo     │  │   Analytics  │          │
│  │   Context    │  │   Context    │  │   Context    │          │
│  │              │  │              │  │              │          │
│  │ - Auth state │  │ - Aggregates │  │ - Queries    │          │
│  │ - Permissions│  │ - Events     │  │ - Projections│          │
│  │ - User prefs │  │ - Commands   │  │ - Cache      │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│         │                  │                  │                 │
│         └──────────────────┼──────────────────┘                 │
│                            │                                    │
│                    ┌───────▼───────┐                           │
│                    │  Event Bus    │                           │
│                    │   (Zenoh)     │                           │
│                    └───────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
```

These implicit boundaries become explicit when contexts split into separate crates or services.
Current monolithic structure enables rapid prototyping while maintaining clear domain separation.

## Context relationship patterns

When contexts need to integrate, use these patterns based on team dynamics and coupling requirements.

### Partnership

Two contexts with mutual dependency and synchronized evolution.
Both teams coordinate changes.

**When to use**: Contexts under same team, tightly coupled features.

**Ironstar example**: Todo and Analytics contexts share event schemas—changes coordinated in same PR.

**Implementation**:
- Shared event types in `domain/todo/events.rs` used by both contexts
- Tests validate both contexts' expectations
- Version changes require both teams' approval

### Customer-supplier

Upstream context serves downstream context's needs.
Downstream influences upstream priorities.

**When to use**: Clear producer-consumer relationship with negotiation.

**Ironstar example**: Event sourcing infrastructure (upstream) serves domain aggregates (downstream).

**Implementation**:
- Downstream requests features via issues
- Upstream provides SLA for interface stability
- Breaking changes require migration path

### Conformist

Downstream context adopts upstream model without influence.

**When to use**: Integrating with external systems or stable libraries.

**Ironstar example**: OAuth integration conforms to GitHub/Google provider models.

**Implementation**:
```rust
// Conformist pattern: adopt GitHub's user model
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
    pub email: Option<String>,
}

// Map directly to our domain without translation
impl From<GitHubUser> for UserIdentity {
    fn from(gh: GitHubUser) -> Self {
        UserIdentity {
            provider: AuthProvider::GitHub,
            provider_id: gh.id.to_string(),
            username: gh.login,
            avatar_url: gh.avatar_url,
            email: gh.email,
        }
    }
}
```

### Anti-corruption layer (ACL)

Translation layer protecting downstream context from upstream model changes.

**When to use**: Integrating with legacy systems, external APIs, or unstable upstreams.

**Implementation**: ACL is a **functor** between type algebras—structure-preserving translation.

```rust
// ACL example: External analytics API → internal QueryResult
pub struct AnalyticsAcl;

impl AnalyticsAcl {
    /// Translate external API response to domain type
    /// This is a functor: preserves structure while changing representation
    pub fn translate_query_result(
        external: ExternalQueryResponse
    ) -> Result<QueryResult, AclError> {
        // Map external fields to domain types
        let rows = external.data
            .into_iter()
            .map(Self::translate_row)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(QueryResult {
            row_count: rows.len(),
            data: rows,
            duration: Duration::from_millis(external.execution_time_ms),
        })
    }

    fn translate_row(external: ExternalRow) -> Result<Row, AclError> {
        // Field-level translation with validation
        Ok(Row {
            values: external.fields
                .into_iter()
                .map(|f| Self::translate_value(f.value))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
```

**Functor laws**: ACL must preserve structure without introducing inconsistencies.

### Open host service

Published API with well-defined protocol for multiple consumers.

**When to use**: Context serves multiple downstream contexts or external clients.

**Ironstar example**: SSE endpoint is an Open Host Service—published protocol (Datastar SDK spec) consumed by browser clients.

**Implementation**:
- Versioned API contract: `GET /api/events` (SSE endpoint)
- Published types via ts-rs TypeScript bindings
- Compatibility guarantees in semantic versioning

### Published language

Shared model defined as explicit interchange format.

**When to use**: Multiple contexts need common vocabulary.

**Ironstar example**: Event schemas in `domain/*/events.rs` with TypeScript bindings via ts-rs.

**Implementation**:
```rust
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum TodoEvent {
    Created { id: String, text: String },
    Completed { id: String },
    Deleted { id: String },
}
```

Generated TypeScript types ensure frontend/backend type alignment without manual synchronization.

## Context canvas

For each bounded context, document using the Context Canvas pattern from DDD.

### Session context canvas

| Aspect | Description |
|--------|-------------|
| **Name** | Session Context |
| **Purpose** | Manage user authentication state and permissions |
| **Strategic classification** | Supporting |
| **Ubiquitous language** | Session, User, Permission, Token, AuthProvider |
| **Business decisions** | Session timeout (24 hours), permission inheritance (future RBAC) |
| **Inbound communication** | OAuth callbacks, session cookies via axum extractors |
| **Outbound communication** | Session validation results, user identity claims |
| **Dependencies** | OAuth providers (GitHub, Google), SQLite session store |

### Todo context canvas

| Aspect | Description |
|--------|-------------|
| **Name** | Todo Context |
| **Purpose** | Demonstrate aggregate patterns with familiar domain |
| **Strategic classification** | Generic Example |
| **Ubiquitous language** | Todo, Task, Complete, Delete, TodoList |
| **Business decisions** | Todo lifecycle state machine (created → completed/deleted) |
| **Inbound communication** | Commands via HTTP POST with session context |
| **Outbound communication** | Events via Zenoh, SSE updates to subscribed clients |
| **Dependencies** | Event store, Session context (for ownership), Zenoh event bus |

### Analytics context canvas

| Aspect | Description |
|--------|-------------|
| **Name** | Analytics Context |
| **Purpose** | Execute analytical queries against remote datasets |
| **Strategic classification** | Core |
| **Ubiquitous language** | Query, Dataset, Result, Chart, Projection |
| **Business decisions** | Query timeout (30s), SQL safety rules (read-only), caching policy (5min TTL) |
| **Inbound communication** | Query commands via HTTP, invalidation events via Zenoh |
| **Outbound communication** | Query events, SSE result streaming, cached projections |
| **Dependencies** | DuckDB, remote data sources (HuggingFace, S3 via httpfs), Session context, moka cache, Zenoh event bus |

## Category-theoretic view

Bounded contexts form a category where:
- **Objects** are contexts (type algebras)
- **Morphisms** are context mappings (functors preserving structure)
- **Composition** is transitive integration (ACL₁ ∘ ACL₂)

The ACL pattern is explicitly a **functor** between categories:

```
F: ContextA → ContextB

F preserves:
- Types: TypeA ↦ TypeB
- Operations: f: A₁ → A₂ ↦ F(f): F(A₁) → F(A₂)
- Identity: F(id) = id
- Composition: F(g ∘ f) = F(g) ∘ F(f)
```

This ensures the ACL is a well-behaved translation that doesn't introduce structural inconsistencies.

### Functor laws in practice

Consider an ACL between an external metrics API and ironstar's internal analytics domain:

```rust
// Functor F: ExternalMetrics → Analytics

// Type mapping
F(ExternalMetricValue) = AnalyticsValue
F(ExternalMetricSeries) = AnalyticsSeries

// Operation mapping
// External operation: aggregate :: [ExternalMetricValue] → ExternalMetricSeries
// Internal operation: project :: [AnalyticsValue] → AnalyticsSeries

// Functor preservation:
F(aggregate(values)) = project(F(values))
```

The functor laws guarantee that aggregating then translating produces the same result as translating then projecting.
This property enables safe composition of ACLs across context boundaries.

## Future decomposition

When ironstar scales beyond single-node deployment, contexts may split into separate services.

### Decomposition triggers

Consider decomposition when:
- **Team scaling**: Conway's Law—separate teams need separate contexts for autonomous delivery
- **Independent scaling requirements**: Analytics needs more compute than Todo tracking
- **Different deployment cadences**: Auth changes rarely (weeks), Analytics evolves rapidly (days)
- **Data isolation requirements**: User PII separate from analytics data for compliance

### Decomposition strategy

When triggers justify separation:

1. **Identify seams** using event flow analysis from EventStorming
   - TodoCreated flows to Analytics projection
   - QueryExecuted flows to DuckDB cache invalidation
   - SessionExpired flows to user state cleanup

2. **Introduce ACL** at identified boundaries while still monolithic
   - Define trait boundaries: `trait TodoEventTranslator`
   - Implement translation layer with functor properties
   - Test independently before extraction

3. **Extract context** to separate service behind ACL
   - Move Todo aggregate to separate crate
   - Expose events via published language (ts-rs)
   - Route commands through HTTP API

4. **Evolve independently** with ACL handling version differences
   - Old event schema: `TodoCreated { id, text }`
   - New event schema: `TodoCreated { id, text, priority }`
   - ACL upcasts old events: adds `priority: None` for backward compatibility

### Integration patterns after decomposition

Post-decomposition integration uses the patterns already established:

| Pattern | Local (monolith) | Distributed (services) |
|---------|------------------|------------------------|
| Partnership | Shared Rust types in workspace | Shared OpenAPI schema with generated clients |
| Customer-Supplier | Direct function calls | HTTP API with SLA guarantees |
| Conformist | Direct struct mapping | HTTP client adopting provider's JSON schema |
| ACL | Translation functions in same binary | Translation service at boundary |
| Open Host Service | Axum handler in same process | Separate API service with load balancer |
| Published Language | ts-rs types in workspace | Protobuf/JSON schema in schema registry |

The architectural patterns remain constant; only the mechanism changes from in-process to over-network.

## See also

- `design-principles.md` - Guiding principles and effect boundaries
- `architecture-decisions.md` - Strategic classification driving context boundaries
- `crate-architecture.md` - Multi-crate decomposition plan with HasXxx traits
- `crate-services-composition.md` - Service layer patterns and composition root
- `~/.claude/commands/preferences/bounded-context-design.md` - Full DDD pattern catalog
- `~/.claude/commands/preferences/strategic-domain-analysis.md` - Core/Supporting/Generic classification
