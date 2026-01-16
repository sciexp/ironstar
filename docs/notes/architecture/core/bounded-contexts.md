# Bounded contexts

Ironstar organizes domain logic into four bounded contexts with distinct responsibilities, invariants, and strategic classifications.
This separation emerged from Domain-Driven Design principles: contexts should have clear boundaries where the same term means different things, different invariants apply, or different change drivers exist.

## Bounded contexts overview

Ironstar has four bounded contexts with distinct responsibilities:

### Analytics (Core)

**Strategic classification**: Core domain (primary differentiator)

**Aggregates:**
- `Catalog`: DuckLake catalog selection and versioning
- `QuerySession`: Query execution context and result caching

**Shared Kernel (value objects):**
- `Chart`: Value objects for visualization specifications (ChartConfig, ChartType, ChartData) — shared with Workspace via Customer-Supplier pattern; Analytics defines them, Workspace consumes them

**Concern**: Scientific data analysis — what data to query, how to transform it

**Invariants:**
- Query validity against DuckLake schema versions
- Column existence in selected catalogs
- Chart type compatibility with query result shapes
- Transformation pipeline correctness

**Ubiquitous language**: Query, Dataset, Result, Chart, Projection, Catalog, Transformation

**Integration**: Publishes Chart value object schemas consumed by Workspace bounded context via Customer-Supplier relationship; no coupling to Generality Canary (Todo)

### Session (Supporting)

**Strategic classification**: Supporting domain

**Aggregates:**
- `Session`: Authentication lifecycle management

**Concern**: WHO is the user, authentication state

**Invariants:**
- Session expiry enforcement (TTL)
- OAuth token validity (GitHub/Google)
- CSRF token matching
- Session fixation prevention

**Ubiquitous language**: Session, User, Permission, Token, AuthProvider

**Note**: Session is per-login; does NOT include persistent user preferences, saved queries, or dashboard layouts — those belong to Workspace bounded context

**Integration**: Provides authenticated `User` identity to Workspace via Shared Kernel pattern

### Workspace (Supporting)

**Strategic classification**: Supporting domain

**Aggregates:**
- `Workspace` (aggregate root): Container for dashboards, saved queries, and workspace-scoped settings
  - Properties: id (uuid), name, owner_id (nullable — null for system-seeded public workspaces), visibility (public|private, default public), created_at, updated_at
  - `Dashboard` (child entity): Layout configuration, tab organization, chart placements
  - `SavedQuery` (child entity): Named queries with parameters
  - `WorkspacePreferences` (child entity): Workspace-scoped settings (default catalog, layout defaults)
- `UserPreferences` (separate aggregate): User-scoped personal settings (theme, locale, UI state) that follow the user across all workspaces

**Concern**: User's persistent saved state across sessions — WHERE charts appear, WHICH queries are saved, HOW the UI is configured

**Invariants:**
- Workspace name unique per owner (or globally for system workspaces where owner_id is null)
- Dashboard/SavedQuery belong to exactly one Workspace
- One WorkspacePreferences per Workspace
- Layout validity (non-overlapping regions, valid grid positions)
- Unique names within workspace scope (SavedQuery, Dashboard)
- Chart placement references valid ChartDefinitions
- Tab organization consistency

**Ubiquitous language**: Workspace, Dashboard, Layout, SavedQuery, UserPreferences, WorkspacePreferences, visibility, Tab, ChartPlacement, Grid

**Lifetime**: Persists across session boundaries — a user logs out (Session expires) but their Workspace configuration, Dashboard layouts, and SavedQuery definitions remain intact for next login

**Relationship:**
- Requires authenticated User (from Session)
- Persists across session boundaries
- Customer-Supplier relationship with Analytics (imports Chart value objects from Analytics.Chart)

**Integration:**
- Customer-Supplier with Analytics: Imports Chart value objects (ChartConfig, ChartType, ChartData) from Analytics.Chart to position charts on dashboards
- Shared Kernel with Session: UserId used to create/access workspaces; Workspace operations only valid within authenticated context
- Public workspaces (visibility=public) accessible to all authenticated users

**MVP vs future:**
- MVP: All workspaces seeded as public, owner_id null (system workspaces)
- Future: User-created private workspaces, workspace_memberships for sharing

### Todo (Generality Canary)

**Strategic classification**: Generic domain (template generality validation)

**Aggregates:**
- `Todo`: Simple task with title, completion status

**Concern**: Prove template patterns generalize beyond scientific data analysis

**Purpose**: Todo exists to validate that ironstar's CQRS/ES infrastructure (EventRepository, Zenoh pub/sub, Session identity, fmodel-rust Deciders) works for ANY domain — not just Analytics. If Todo can be implemented cleanly without coupling to Analytics-specific concepts, the template is genuinely reusable.

**Invariants:**
- Title non-empty
- Completion state boolean

**Ubiquitous language**: Todo, Task, Complete, Delete, TodoList

**Integration**: Fully isolated — shares infrastructure (EventRepository, Zenoh, Session) but NO domain coupling with other bounded contexts. This isolation is intentional: coupling Todo to Analytics would defeat its purpose as a generality test.

## Context map

The following diagram shows integration patterns between bounded contexts.
Arrows indicate dependency direction.

```
Session (Supporting) ────[Shared Kernel: User identity]────> Workspace (Supporting)
                                                                      │
Analytics (Core) ──────[Customer-Supplier: Chart value objects]──────┘

Todo (Generality Canary) ──────[Isolated for template generality validation]
```

**Relationship patterns:**

- **Shared Kernel** (Session → Workspace): User identity is shared concept; both contexts must agree on User structure
- **Customer-Supplier** (Analytics → Workspace): Analytics defines Chart value objects (ChartConfig, ChartType, ChartData); Workspace consumes them for dashboard layout; Analytics owns the schema, Workspace uses them
- **Isolated** (Todo): No domain coupling with other contexts; validates template generality by proving CQRS/ES infrastructure works for any domain without coupling to Analytics-specific concerns

## Design decision rationale

### Why separate Workspace from Session?

Workspace was separated from Session because they have fundamentally different concerns, lifetimes, and invariants.

**Different concerns:**
- Session: Authentication lifecycle — token validation, expiry, OAuth flows
- Workspace: Persistent user state — layouts, saved queries, preferences

**Different lifetimes:**
- Session: Ephemeral, expires after TTL (hours/days)
- Workspace: Durable, persists across sessions (months/years)

**Different invariants:**
- Session: Token validity, CSRF protection, session fixation prevention
- Workspace: Layout validity, unique naming, chart reference integrity

**Different change drivers:**
- Session: Security requirements, OAuth provider changes, token formats
- Workspace: UI/UX evolution, layout features, query management

**Cleaner boundaries:**
Keeping Session focused solely on authentication prevents it from becoming a dumping ground for "user-related stuff."
Workspace explicitly owns persistent state, making the domain model clearer.

### Why separate Workspace from Analytics?

Workspace was separated from Analytics because they have fundamentally different concerns and strategic purposes.
The two contexts share Chart value objects via a Customer-Supplier pattern: Analytics defines them, Workspace consumes them for dashboard layout.

**Different concerns:**
- Analytics: Data correctness, transformation validity, catalog versioning — "what queries to execute and how to transform results"
- Workspace: Visual arrangement, layout persistence, user customization — "where to position charts on the dashboard"

**Different language (despite shared Chart value object):**
- Analytics: ChartConfig/ChartType/ChartData = specification for rendering query results (ECharts/Vega-Lite schemas)
- Workspace: ChartPlacement = metadata about where a chart appears (x/y coordinates, width/height, tab assignment)

**Different change drivers:**
- Analytics: Data source changes, new transformation types, query optimization
- Workspace: UI/UX improvements, layout features, dashboard templates

**Strategic clarity:**
- Analytics: Core domain (competitive differentiator via scientific analysis)
- Workspace: Supporting domain (necessary for usability but not differentiating)

**Avoid coupling via Customer-Supplier:**
Mixing these concerns would tangle unrelated invariants: Analytics validating chart schema correctness would be coupled with Workspace validating grid positions.
The Customer-Supplier relationship keeps them decoupled: Analytics owns and evolves Chart value object schemas independently; Workspace consumes them as stable contracts for positioning charts.

## Strategic classification

Following Eric Evans's strategic design patterns, each context is classified by its role in competitive differentiation:

- **Core domain**: Primary source of competitive advantage; requires custom development and deep domain expertise
- **Supporting domain**: Necessary but not differentiating; supports core domain operations
- **Generic domain**: Commodity functionality; could be replaced with off-the-shelf solutions

This classification guides investment decisions: Core domains receive most architectural attention, Supporting domains balance simplicity with capability, Generic domains prefer proven patterns over innovation.

## Implementation boundaries

Each bounded context maps to a Rust module with isolated types, events, and projections:

```
crates/ironstar/src/domain/
├── analytics/       # Core domain
├── session/         # Supporting domain
├── workspace/       # Supporting domain
└── todo/            # Generality canary
```

Events from different contexts use distinct type namespaces:
- `AnalyticsEvent::QueryExecuted`
- `SessionEvent::UserLoggedIn`
- `WorkspaceEvent::DashboardCreated`
- `TodoEvent::TodoCompleted`

Zenoh key expressions enforce context isolation:
- `events/Analytics/**`
- `events/Session/**`
- `events/Workspace/**`
- `events/Todo/**`

Cross-context integration happens via:
1. **Shared Kernel**: Workspace imports `Session::User` type and `Analytics::Chart` value objects (ChartConfig, ChartType, ChartData)
2. **Customer-Supplier**: Workspace subscribes to `events/Analytics/Chart/**` keys for chart definition updates
3. **Process Managers**: Coordinate multi-context workflows (e.g., Dashboard creation after first login)

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

**Ironstar example**: Analytics (upstream) serves Workspace (downstream) with Chart value object schemas (ChartConfig, ChartType, ChartData).

**Implementation**:
- Downstream (Workspace) requests chart type extensions via issues
- Upstream (Analytics) provides SLA for interface stability of Chart value objects
- Breaking changes to Chart schema require migration path (e.g., upcasters for stored ChartPlacement references)

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

### Analytics context canvas

| Aspect | Description |
|--------|-------------|
| **Name** | Analytics Context |
| **Purpose** | Execute analytical queries against remote datasets and define visualization specifications |
| **Strategic classification** | Core |
| **Ubiquitous language** | Query, Dataset, Result, Chart, Projection, Catalog, Transformation |
| **Business decisions** | Query timeout (30s), SQL safety rules (read-only), caching policy (5min TTL) |
| **Inbound communication** | Query commands via HTTP, invalidation events via Zenoh |
| **Outbound communication** | Query events, SSE result streaming, cached projections, Chart value object updates |
| **Published Types** | Chart value objects (ChartConfig, ChartType, ChartData) consumed by Workspace via Customer-Supplier |
| **Dependencies** | DuckDB, remote data sources (HuggingFace, S3 via httpfs), Session context, moka cache, Zenoh event bus |

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

### Workspace context canvas

| Aspect | Description |
|--------|-------------|
| **Name** | Workspace Context |
| **Purpose** | Manage user's persistent saved state across sessions |
| **Strategic classification** | Supporting |
| **Ubiquitous language** | Workspace, Dashboard, Layout, SavedQuery, UserPreferences, WorkspacePreferences, visibility, Tab, ChartPlacement, Grid |
| **Business decisions** | Layout grid system (12-column), max tabs per dashboard (20), saved query retention (indefinite), MVP uses public system workspaces |
| **Inbound communication** | Layout commands via HTTP, Chart value object updates via Zenoh, User identity from Session |
| **Outbound communication** | Workspace events, SSE layout updates, saved query results |
| **Consumed Types** | Chart value objects (ChartConfig, ChartType, ChartData) from Analytics.Chart for dashboard positioning |
| **Dependencies** | Session context (User identity), Analytics context (Chart value objects), SQLite workspace store, Zenoh event bus |

### Todo context canvas

| Aspect | Description |
|--------|-------------|
| **Name** | Todo Context |
| **Purpose** | Validate template generality by proving CQRS/ES patterns work for any domain without coupling to Analytics-specific concerns |
| **Strategic classification** | Generality Canary |
| **Ubiquitous language** | Todo, Task, Complete, Delete, TodoList |
| **Business decisions** | Todo lifecycle state machine (created → completed/deleted) |
| **Inbound communication** | Commands via HTTP POST with session context |
| **Outbound communication** | Events via Zenoh, SSE updates to subscribed clients |
| **Dependencies** | Event store, Session context (for ownership), Zenoh event bus |

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

## Workspace aggregates

This section documents the aggregates within the Workspace bounded context.
Each aggregate follows the fmodel-rust Decider pattern with pure `decide`/`evolve`/`initial_state` functions.

### UserPreferences aggregate

User-scoped personal settings that follow the user across all workspaces.

**Aggregate ID pattern**: `user_{user_id}/preferences`

**Properties:**
- `preferences_id`: Unique identifier (UUID)
- `user_id`: Reference to authenticated user (from Session context)
- `theme`: Light/dark/system preference
- `locale`: Language and regional settings (e.g., "en-US")
- `ui_state`: Optional JSON blob for persistent UI state (collapsed panels, etc.)
- `created_at`: Timestamp of initialization
- `updated_at`: Timestamp of last modification

**Commands:**
- `InitializeUserPreferences`: Create preferences for new user (idempotent)
- `SetUserTheme`: Update theme preference
- `SetUserLocale`: Update locale preference
- `UpdateUserUiState`: Persist UI state changes

**Events:**
- `UserPreferencesInitialized { preferences_id, user_id, theme, locale, created_at }`
- `UserThemeSet { preferences_id, theme, updated_at }`
- `UserLocaleSet { preferences_id, locale, updated_at }`
- `UserUiStateUpdated { preferences_id, ui_state, updated_at }`

**Invariants:**
- One UserPreferences per User (enforced by aggregate ID pattern)
- Theme must be valid enum variant (Light, Dark, System)
- Locale must be valid BCP 47 language tag

**Decider pattern alignment:**
- `decide`: Pure function `(Command, State) -> Result<Vec<Event>, Error>`
- `evolve`: Pure function `(State, Event) -> State`
- `initial_state`: Default preferences (theme=System, locale=en-US, ui_state=None)
- No async in aggregate logic; side effects at axum/Zenoh boundaries only

### WorkspacePreferences aggregate

Workspace-scoped settings that belong to a specific workspace.
These settings apply to all users within the workspace context.

**Aggregate ID pattern**: `workspace_{workspace_id}/preferences`

**Properties:**
- `preferences_id`: Unique identifier (UUID)
- `workspace_id`: Reference to containing workspace
- `default_catalog`: Optional CatalogName for new queries in this workspace
- `layout_defaults`: Optional JSON blob for workspace-wide layout settings
- `created_at`: Timestamp of initialization
- `updated_at`: Timestamp of last modification

**Commands:**
- `InitializeWorkspacePreferences`: Create preferences for new workspace (idempotent)
- `SetWorkspaceDefaultCatalog`: Set default catalog for new queries
- `ClearWorkspaceDefaultCatalog`: Remove default catalog (queries use global default)
- `UpdateWorkspaceLayoutDefaults`: Update workspace-wide layout settings

**Events:**
- `WorkspacePreferencesInitialized { preferences_id, workspace_id, created_at }`
- `WorkspaceDefaultCatalogSet { preferences_id, catalog_name, updated_at }`
- `WorkspaceDefaultCatalogCleared { preferences_id, updated_at }`
- `WorkspaceLayoutDefaultsUpdated { preferences_id, layout_defaults, updated_at }`

**Invariants:**
- One WorkspacePreferences per Workspace (enforced by aggregate ID pattern)
- `default_catalog` must reference a valid catalog if present (validated at command handling, not in aggregate)

**Decider pattern alignment:**
- `decide`: Pure function `(Command, State) -> Result<Vec<Event>, Error>`
- `evolve`: Pure function `(State, Event) -> State`
- `initial_state`: Empty preferences (default_catalog=None, layout_defaults=None)
- No async in aggregate logic; side effects at axum/Zenoh boundaries only

**Relationship to UserPreferences:**
- UserPreferences: Settings that follow the user (theme, locale)
- WorkspacePreferences: Settings that belong to the workspace (default catalog, layout defaults)
- A user working in a workspace sees both their personal preferences and the workspace preferences

## See also

- `design-principles.md` - Guiding principles and effect boundaries
- `architecture-decisions.md` - Strategic classification driving context boundaries
- `crate-architecture.md` - Multi-crate decomposition plan with HasXxx traits
- `crate-services-composition.md` - Service layer patterns and composition root
- `~/.claude/commands/preferences/bounded-context-design.md` - Full DDD pattern catalog
- `~/.claude/commands/preferences/strategic-domain-analysis.md` - Core/Supporting/Generic classification
