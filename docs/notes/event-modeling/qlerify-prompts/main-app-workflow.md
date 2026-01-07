# Ironstar Scientific Data Analysis Platform

## Workflow Description

This workflow models the complete user journey for a scientific data analysis platform built with event sourcing. A data analyst authenticates via OAuth, explores data catalogs, executes SQL queries, visualizes results as charts, and organizes findings into persistent dashboards.

## Actors

1. **Data Analyst** (Human) — Primary user who authenticates, queries data, and builds dashboards
2. **Automation** — Background processes handling OAuth callbacks, query execution, session lifecycle, and cache operations

## Workflow Narrative (Chronological)

### Phase 1: Authentication (Session Context)

1. Data Analyst navigates to the application and initiates login via GitHub OAuth
2. OAuth provider authenticates user and redirects with authorization code
3. **Automation** processes OAuth callback, exchanges code for tokens, and extracts user identity
4. **Automation** creates a new session linking the authenticated user identity
5. Session is now active with a time-to-live (TTL) for automatic expiration
6. Optionally, **Automation** refreshes session TTL when user activity is detected before expiration
7. Data Analyst may explicitly logout, invalidating the session immediately
8. If no activity occurs, **Automation** marks the session as expired when TTL elapses

### Phase 2: Catalog Selection (Analytics Context - Catalog Aggregate)

9. Data Analyst views available DuckLake catalogs (from external catalog registry)
10. Data Analyst selects a catalog to work with (e.g., "hf://datasets/sciexp/fixtures")
11. System records the catalog selection
12. Data Analyst may request catalog metadata refresh to see latest datasets
13. System refreshes and stores updated catalog metadata (dataset list, table counts, schema versions)

### Phase 3: Query Execution (Analytics Context - QuerySession Aggregate)

14. Data Analyst views the catalog's available datasets via the Dataset Browser read model
15. Data Analyst composes a SQL query targeting a specific dataset
16. Data Analyst optionally specifies chart configuration (chart type, axis labels, colors)
17. Data Analyst submits the query for execution
18. **Automation** begins query execution against DuckDB with the selected catalog
19. If query succeeds, **Automation** records query completion with results and duration
20. If query fails (syntax error, timeout, resource exhaustion), **Automation** records query failure with error message
21. Data Analyst may cancel a long-running query before it completes
22. System records query cancellation
23. Completed query results are available in QueryResults and ChartData read models

### Phase 4: Dashboard Management (Workspace Context - Dashboard Aggregate)

24. Data Analyst creates a new dashboard with a name (e.g., "Q4 Metrics")
25. Data Analyst adds a chart to the dashboard, specifying grid position and size
26. The chart references a ChartDefinition produced by a prior query execution
27. Data Analyst may create tabs within the dashboard for organization (e.g., "Overview", "Details")
28. Data Analyst may move charts between tabs
29. Data Analyst may remove charts that are no longer needed
30. Data Analyst may rename the dashboard
31. DashboardLayoutView read model reflects current layout for UI rendering

### Phase 5: Query Persistence (Workspace Context - SavedQuery Aggregate)

32. Data Analyst saves a frequently-used query with a descriptive name
33. Saved query stores the SQL text and dataset reference for reuse
34. Data Analyst may rename saved queries or update their SQL content
35. Data Analyst may delete saved queries that are obsolete

### Phase 6: User Preferences (Workspace Context - UserPreferences Aggregate)

36. Data Analyst initializes personal preferences on first use
37. Data Analyst sets theme preference (Light, Dark, or System)
38. Data Analyst sets a default catalog to auto-select on login
39. Data Analyst may clear the default catalog preference
40. Data Analyst may store arbitrary UI state (panel sizes, collapsed sections) as JSON

## Key Events (Past Tense)

### Session Events
- SessionCreated, SessionRefreshed, SessionInvalidated, SessionExpired

### Catalog Events
- CatalogSelected, CatalogMetadataRefreshed

### QuerySession Events
- QueryStarted, QueryCompleted, QueryFailed, QueryCancelled

### Dashboard Events
- DashboardCreated, ChartAdded, ChartRemoved, TabAdded, ChartMovedToTab, DashboardRenamed

### SavedQuery Events
- QuerySaved, QueryDeleted, QueryRenamed, QuerySqlUpdated, DatasetRefUpdated

### UserPreferences Events
- PreferencesInitialized, ThemeSet, DefaultCatalogSet, DefaultCatalogCleared, UiStateUpdated

## Exceptional Flows

- **OAuth failure**: External provider denies authentication → no session created
- **Catalog unavailable**: Selected catalog is offline → selection fails with error
- **Query timeout**: Query exceeds time limit → QueryFailed with timeout error
- **Invalid SQL**: Query syntax error → QueryFailed with parse error
- **Chart type mismatch**: ChartConfig incompatible with QueryResults → validation error
- **Session expired**: User attempts action with expired session → redirect to login

## Bounded Contexts (for Step 6 - Conway's Law)

- **Session** (Supporting): Session aggregate, UserId shared kernel export
- **Analytics** (Core): Catalog aggregate, QuerySession aggregate, Chart shared kernel export
- **Workspace** (Supporting): Dashboard, SavedQuery, UserPreferences aggregates

## Integration Patterns

- Session → Analytics/Workspace: Customer-Supplier (Session is upstream identity provider)
- Analytics → Workspace: Customer-Supplier (Workspace consumes ChartDefinition references)
- Session → Workspace: Shared Kernel (UserId flows to all Workspace aggregates)
