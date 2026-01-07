||| Workspace bounded context
|||
||| The Workspace bounded context manages user's persistent configuration
||| across sessions: dashboard layouts, saved queries, and UI preferences.
|||
||| Strategic classification: Supporting domain
|||
||| Aggregates:
||| - Dashboard: Layout configuration, tab organization, chart placements
||| - SavedQuery: Named queries with parameters
||| - UserPreferences: Theme, defaults, UI state
|||
||| Relationships:
||| - Customer-Supplier with Analytics: Consumes ChartDefinition references
||| - Requires authenticated User from Session: Operations only valid in auth context
|||
||| Lifetime: Persists across session boundaries
||| When a session expires (user logs out), Workspace state remains intact
||| for next login. This distinguishes Workspace (permanent) from Session (ephemeral).
|||
||| Integration notes:
||| - Dashboard.ChartPlacement.chartDefRef references Analytics.ChartDefinition IDs
||| - SavedQuery.DatasetRef references dataset URIs that DuckDB can resolve
||| - UserPreferences.defaultCatalog references DuckDB catalog names
|||
||| All aggregates use the Decider pattern from Core.Decider:
||| - Pure decision logic (decide: Command -> State -> Either Error (List Event))
||| - Total event application (evolve: State -> Event -> State)
||| - Deterministic replay (reconstruct: List Event -> State)
module Workspace.Workspace

-- Import Core patterns
import Core.Decider

-- Import all aggregates
import public Workspace.Dashboard
import public Workspace.SavedQuery
import public Workspace.Preferences

%default total

------------------------------------------------------------------------
-- Workspace context documentation
------------------------------------------------------------------------

-- Workspace ubiquitous language:
--
-- Core concepts:
-- - Dashboard: Visual organization of charts and analytics
-- - Layout: Grid-based positioning system for charts
-- - Tab: Logical grouping of charts within a dashboard
-- - ChartPlacement: Association of ChartDefinition with position/size
-- - SavedQuery: Named DuckDB query with dataset reference
-- - UserPreferences: Persistent UI configuration (theme, defaults)
--
-- Invariants enforced across aggregates:
-- 1. Dashboard layout validity:
--    - Grid positions are non-negative (row, col : Nat)
--    - Placements should not overlap (documented, not type-enforced)
-- 2. Reference integrity:
--    - ChartPlacement.chartDefRef must reference existing ChartDefinition
--    - SavedQuery.datasetRef must be valid DuckDB URI
--    - UserPreferences.defaultCatalog must be valid catalog name
-- 3. Uniqueness within user scope:
--    - Dashboard names unique per user (enforced at boundary via aggregate ID)
--    - SavedQuery names unique per user (enforced at boundary via aggregate ID)
--    - One UserPreferences per user (enforced at boundary via aggregate ID)
--
-- Aggregate ID conventions:
-- - Dashboard: "Dashboard" / "user_{user_id}/dashboard_{dashboard_name}"
-- - SavedQuery: "SavedQuery" / "user_{user_id}/query_{query_name}"
-- - UserPreferences: "UserPreferences" / "user_{user_id}"
--
-- Customer-Supplier relationships:
-- - Analytics → Workspace: ChartDefinition IDs consumed by Dashboard
-- - Workspace does not emit events that Analytics subscribes to
-- - Workspace is downstream consumer only

------------------------------------------------------------------------
-- Composition examples
------------------------------------------------------------------------

||| Example: Combined Workspace decider for all three aggregates
|||
||| This shows how to combine independent aggregates using Core.Decider.combine3.
||| In practice, each aggregate is managed separately with distinct aggregate IDs,
||| but composition enables unified command handling if needed.
|||
||| Type signature:
||| workspaceDecider : Decider
|||   (Sum3 DashboardCommand SavedQueryCommand PreferencesCommand)
|||   (DashboardState, SavedQueryState, PreferencesState)
|||   (Sum3 DashboardEvent SavedQueryEvent PreferencesEvent)
|||   String
public export
workspaceDecider : Decider
  (Sum3 DashboardCommand SavedQueryCommand PreferencesCommand)
  (DashboardState, SavedQueryState, PreferencesState)
  (Sum3 DashboardEvent SavedQueryEvent PreferencesEvent)
  String
workspaceDecider = combine3 dashboardDecider savedQueryDecider preferencesDecider

------------------------------------------------------------------------
-- Integration with Core patterns
------------------------------------------------------------------------

-- All Workspace aggregates follow the Decider pattern:
--
-- 1. Pure decision logic (decide):
--    - No IO, no side effects
--    - Returns Either for validation errors
--    - Returns List of events (empty list = no-op)
--
-- 2. Total event application (evolve):
--    - Never fails (events are historical facts)
--    - Deterministic (same event + state = same new state)
--
-- 3. Deterministic replay:
--    - reconstruct : List Event -> State
--    - Enables event sourcing, snapshots, projections
--
-- Effect boundaries (see Core.Effect):
-- - EventRepository: Persist/fetch events (IO)
-- - EventNotifier: Publish events to Zenoh (IO)
-- - EventSubscriber: Subscribe to event streams (IO)
--
-- Workspace aggregates are pure; IO happens at axum boundary:
-- - HTTP POST /dashboard/create -> decide -> append -> publish
-- - SSE GET /workspace/events -> subscribe -> SSE stream
