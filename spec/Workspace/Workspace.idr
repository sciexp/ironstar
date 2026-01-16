||| Workspace bounded context
|||
||| The Workspace bounded context manages multi-tenant workspaces with ownership,
||| visibility, and persistent configuration: dashboard layouts, saved queries,
||| and preferences (both user-scoped and workspace-scoped).
|||
||| Strategic classification: Supporting domain
|||
||| Aggregates:
||| - WorkspaceAggregate: Workspace identity, ownership (UserId), visibility (Public/Private)
||| - Dashboard: Layout configuration, tab organization, chart placements (belongs to workspace)
||| - SavedQuery: Named queries with parameters (belongs to workspace)
||| - UserPreferences: User-scoped settings (theme, locale) - follows user across workspaces
||| - WorkspacePreferences: Workspace-scoped settings (default catalog, layout defaults)
|||
||| Relationships:
||| - Customer-Supplier with Analytics: Consumes ChartDefinition references
||| - Shared Kernel with Session: Imports UserId for workspace ownership
||| - Requires authenticated User from Session: Operations only valid in auth context
|||
||| Lifetime: Persists across session boundaries
||| When a session expires (user logs out), Workspace state remains intact
||| for next login. This distinguishes Workspace (permanent) from Session (ephemeral).
|||
||| Integration notes:
||| - WorkspaceAggregate.ownerId references Session.UserId (Shared Kernel)
||| - Dashboard.workspaceId and SavedQuery.workspaceId reference WorkspaceId
||| - Dashboard.ChartPlacement.chartDefRef references Analytics.ChartDefinition IDs
||| - SavedQuery.DatasetRef references dataset URIs that DuckDB can resolve
||| - WorkspacePreferences.defaultCatalog references DuckDB catalog names
|||
||| All aggregates use the Decider pattern from Core.Decider:
||| - Pure decision logic (decide: Command -> State -> Either Error (List Event))
||| - Total event application (evolve: State -> Event -> State)
||| - Deterministic replay (reconstruct: List Event -> State)
module Workspace.Workspace

-- Import Core patterns
import Core.Decider

-- Re-export all aggregates
import public Workspace.WorkspaceAggregate as WA
import public Workspace.WorkspacePreferences as WP
import public Workspace.Dashboard as D
import public Workspace.SavedQuery as SQ
import public Workspace.UserPreferences as UP

%default total

------------------------------------------------------------------------
-- Workspace context documentation
------------------------------------------------------------------------

-- Workspace ubiquitous language:
--
-- Core concepts:
-- - Workspace: Multi-tenant organizational container with ownership and visibility
-- - Dashboard: Visual organization of charts and analytics (belongs to workspace)
-- - Layout: Grid-based positioning system for charts
-- - Tab: Logical grouping of charts within a dashboard
-- - ChartPlacement: Association of ChartDefinition with position/size
-- - SavedQuery: Named DuckDB query with dataset reference (belongs to workspace)
-- - UserPreferences: User-scoped UI configuration (theme, locale) - follows user across workspaces
-- - WorkspacePreferences: Workspace-scoped settings (default catalog, layout defaults)
--
-- Invariants enforced across aggregates:
-- 1. Workspace ownership:
--    - WorkspaceId is immutable after creation
--    - OwnerId references UserId from Session (Shared Kernel)
--    - Visibility controls access (Public/Private)
-- 2. Dashboard layout validity:
--    - Grid positions are non-negative (row, col : Nat)
--    - Placements should not overlap (documented, not type-enforced)
--    - Dashboard belongs to exactly one workspace (workspaceId field)
-- 3. Reference integrity:
--    - ChartPlacement.chartDefRef must reference existing ChartDefinition
--    - SavedQuery.datasetRef must be valid DuckDB URI
--    - SavedQuery belongs to exactly one workspace (workspaceId field)
--    - WorkspacePreferences.defaultCatalog must be valid catalog name
-- 4. Uniqueness within workspace scope:
--    - Dashboard names unique per workspace (enforced at boundary via aggregate ID)
--    - SavedQuery names unique per workspace (enforced at boundary via aggregate ID)
--    - One WorkspacePreferences per workspace (enforced at boundary via aggregate ID)
--    - One UserPreferences per user (enforced at boundary via aggregate ID)
--
-- Aggregate ID conventions:
-- - Workspace: "Workspace" / "workspace_{workspace_id}"
-- - Dashboard: "Dashboard" / "workspace_{workspace_id}/dashboard_{dashboard_name}"
-- - SavedQuery: "SavedQuery" / "workspace_{workspace_id}/query_{query_name}"
-- - WorkspacePreferences: "WorkspacePreferences" / "workspace_{workspace_id}"
-- - UserPreferences: "UserPreferences" / "user_{user_id}"
--
-- Customer-Supplier relationships:
-- - Analytics → Workspace: ChartDefinition IDs consumed by Dashboard
-- - Workspace does not emit events that Analytics subscribes to
-- - Workspace is downstream consumer only
--
-- Shared Kernel:
-- - Session.UserId imported for workspace ownership

------------------------------------------------------------------------
-- Sum5 type for five-way composition
------------------------------------------------------------------------

||| Sum type for five alternatives (extends Sum3 from Core.Decider)
public export
data Sum5 : Type -> Type -> Type -> Type -> Type -> Type where
  Sum5_1 : a -> Sum5 a b c d e
  Sum5_2 : b -> Sum5 a b c d e
  Sum5_3 : c -> Sum5 a b c d e
  Sum5_4 : d -> Sum5 a b c d e
  Sum5_5 : e -> Sum5 a b c d e

||| Combine five independent Deciders into one.
|||
||| This enables multi-aggregate bounded contexts where each aggregate handles
||| its own command/event types independently while sharing a unified interface.
|||
||| Type transformation:
||| - Commands: `Sum5 c1 c2 c3 c4 c5` (route to appropriate decider)
||| - States: `(s1, s2, s3, s4, s5)` (product of independent states)
||| - Events: `Sum5 e1 e2 e3 e4 e5` (tagged union of event types)
|||
||| Key property: no interference between deciders (state isolation).
||| The combined Decider preserves all laws of the component Deciders.
||| Monoidal composition is associative: combine5 d1 d2 d3 d4 d5 is equivalent
||| to nested applications of combine.
public export
combine5 : Decider c1 s1 e1 err
        -> Decider c2 s2 e2 err
        -> Decider c3 s3 e3 err
        -> Decider c4 s4 e4 err
        -> Decider c5 s5 e5 err
        -> Decider (Sum5 c1 c2 c3 c4 c5) (s1, s2, s3, s4, s5) (Sum5 e1 e2 e3 e4 e5) err
combine5 d1 d2 d3 d4 d5 = MkDecider
  { decide = \cmd, (s1, s2, s3, s4, s5) => case cmd of
      Sum5_1 c1 => map (map Sum5_1) (d1.decide c1 s1)
      Sum5_2 c2 => map (map Sum5_2) (d2.decide c2 s2)
      Sum5_3 c3 => map (map Sum5_3) (d3.decide c3 s3)
      Sum5_4 c4 => map (map Sum5_4) (d4.decide c4 s4)
      Sum5_5 c5 => map (map Sum5_5) (d5.decide c5 s5)
  , evolve = \(s1, s2, s3, s4, s5), evt => case evt of
      Sum5_1 e1 => (d1.evolve s1 e1, s2, s3, s4, s5)
      Sum5_2 e2 => (s1, d2.evolve s2 e2, s3, s4, s5)
      Sum5_3 e3 => (s1, s2, d3.evolve s3 e3, s4, s5)
      Sum5_4 e4 => (s1, s2, s3, d4.evolve s4 e4, s5)
      Sum5_5 e5 => (s1, s2, s3, s4, d5.evolve s5 e5)
  , initialState = (d1.initialState, d2.initialState, d3.initialState, d4.initialState, d5.initialState)
  }

------------------------------------------------------------------------
-- Combined Workspace context command/event/state types
------------------------------------------------------------------------

||| Combined command type for all Workspace aggregates
public export
WorkspaceContextCommand : Type
WorkspaceContextCommand = Sum5
  WA.WorkspaceCommand
  WP.WorkspacePreferencesCommand
  D.DashboardCommand
  SQ.SavedQueryCommand
  UP.PreferencesCommand

||| Combined event type for all Workspace aggregates
public export
WorkspaceContextEvent : Type
WorkspaceContextEvent = Sum5
  WA.WorkspaceEvent
  WP.WorkspacePreferencesEvent
  D.DashboardEvent
  SQ.SavedQueryEvent
  UP.PreferencesEvent

||| Combined state type for all Workspace aggregates
public export
WorkspaceContextState : Type
WorkspaceContextState =
  ( WA.WorkspaceState
  , WP.WorkspacePreferencesState
  , D.DashboardState
  , SQ.SavedQueryState
  , UP.PreferencesState
  )

------------------------------------------------------------------------
-- Combined Workspace decider
------------------------------------------------------------------------

||| Combined Workspace decider for all five aggregates
|||
||| This composes all five aggregates in the Workspace bounded context:
||| 1. WorkspaceAggregate: Workspace identity, ownership (UserId), visibility
||| 2. WorkspacePreferences: Workspace-scoped settings (default catalog, layout defaults)
||| 3. Dashboard: Layout configuration, tab organization, chart placements
||| 4. SavedQuery: Named queries with parameters
||| 5. UserPreferences: User-scoped settings (theme, locale) - follows user across workspaces
|||
||| In practice, each aggregate is typically managed separately with distinct aggregate IDs,
||| but composition enables unified command handling and demonstrates that the bounded
||| context forms a coherent algebraic structure.
|||
||| Monoidal composition preserves correctness: the combined Decider satisfies all laws
||| (pure decide, total evolve, deterministic replay) when each component does.
public export
workspaceContextDecider : Decider WorkspaceContextCommand WorkspaceContextState WorkspaceContextEvent String
workspaceContextDecider = combine5
  WA.workspaceDecider
  WP.workspacePreferencesDecider
  D.dashboardDecider
  SQ.savedQueryDecider
  UP.preferencesDecider

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
