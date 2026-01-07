||| Dashboard aggregate for layout configuration
|||
||| The Dashboard aggregate manages persistent UI layout configuration
||| including chart placements, tab organization, and grid positioning.
|||
||| Key invariants:
||| - Chart placements reference valid ChartDefinitions from Analytics context
||| - Grid positions are valid (non-negative row/col)
||| - TabIds are unique within a dashboard
||| - ChartIds are unique within a dashboard
|||
||| Customer-Supplier relationship: Consumes ChartDefinition IDs from Analytics
module Workspace.Dashboard

import Core.Decider
import Core.Event
import Data.List
import Data.Nat

%default total

------------------------------------------------------------------------
-- Value Objects
------------------------------------------------------------------------

||| Unique identifier for a dashboard
public export
record DashboardId where
  constructor MkDashboardId
  unDashboardId : String

public export
Eq DashboardId where
  (MkDashboardId x) == (MkDashboardId y) = x == y

||| Unique identifier for a tab within a dashboard
public export
record TabId where
  constructor MkTabId
  unTabId : String

public export
Eq TabId where
  (MkTabId x) == (MkTabId y) = x == y

||| Unique identifier for a chart placement
public export
record ChartId where
  constructor MkChartId
  unChartId : String

public export
Eq ChartId where
  (MkChartId x) == (MkChartId y) = x == y

||| Reference to Analytics ChartDefinition (Customer-Supplier relationship)
||| The dashboard does not own the chart definition, only references it.
public export
record ChartDefinitionRef where
  constructor MkChartDefinitionRef
  refId : String

public export
Eq ChartDefinitionRef where
  (MkChartDefinitionRef x) == (MkChartDefinitionRef y) = x == y

||| Position in grid layout (0-indexed)
public export
record GridPosition where
  constructor MkGridPosition
  row : Nat
  col : Nat

public export
Eq GridPosition where
  (MkGridPosition r1 c1) == (MkGridPosition r2 c2) = r1 == r2 && c1 == c2

||| Size in grid units
public export
record GridSize where
  constructor MkGridSize
  width : Nat
  height : Nat

public export
Eq GridSize where
  (MkGridSize w1 h1) == (MkGridSize w2 h2) = w1 == w2 && h1 == h2

||| Chart placement on dashboard
||| Associates a chart reference with position, size, and optional tab
public export
record ChartPlacement where
  constructor MkChartPlacement
  chartId : ChartId
  chartDefRef : ChartDefinitionRef
  position : GridPosition
  size : GridSize
  tabId : Maybe TabId

public export
Eq ChartPlacement where
  p1 == p2 = p1.chartId == p2.chartId
          && p1.chartDefRef == p2.chartDefRef
          && p1.position == p2.position
          && p1.size == p2.size
          && p1.tabId == p2.tabId

||| Tab metadata
public export
record TabInfo where
  constructor MkTabInfo
  tabId : TabId
  name : String

public export
Eq TabInfo where
  t1 == t2 = t1.tabId == t2.tabId && t1.name == t2.name

------------------------------------------------------------------------
-- Commands
------------------------------------------------------------------------

||| Commands for dashboard management
public export
data DashboardCommand
  = CreateDashboard String  -- name
  | AddChart ChartPlacement
  | RemoveChart ChartId
  | AddTab String  -- tab name
  | MoveChartToTab ChartId TabId
  | RenameDashboard String

------------------------------------------------------------------------
-- Events
------------------------------------------------------------------------

||| Events representing dashboard state changes
public export
data DashboardEvent
  = DashboardCreated DashboardId String Timestamp
  | ChartAdded ChartPlacement Timestamp
  | ChartRemoved ChartId Timestamp
  | TabAdded TabInfo Timestamp
  | ChartMovedToTab ChartId TabId Timestamp
  | DashboardRenamed String Timestamp

------------------------------------------------------------------------
-- State
------------------------------------------------------------------------

||| Dashboard aggregate state
||| Invariant: If dashboardId is Just, then dashboard exists
public export
record DashboardState where
  constructor MkDashboardState
  dashboardId : Maybe DashboardId
  name : String
  placements : List ChartPlacement
  tabs : List TabInfo

||| Initial state: no dashboard created yet
public export
initialDashboardState : DashboardState
initialDashboardState = MkDashboardState Nothing "" [] []

------------------------------------------------------------------------
-- Helper functions for validation
------------------------------------------------------------------------

||| Check if a chart ID exists in placements
hasChart : ChartId -> List ChartPlacement -> Bool
hasChart cid = any (\p => p.chartId == cid)

||| Check if a tab ID exists in tabs
hasTab : TabId -> List TabInfo -> Bool
hasTab tid = any (\t => t.tabId == tid)

||| Update tab assignment for a chart
updateTabIfMatch : ChartId -> TabId -> ChartPlacement -> ChartPlacement
updateTabIfMatch targetId newTabId p =
  if p.chartId == targetId
    then { tabId := Just newTabId } p
    else p

------------------------------------------------------------------------
-- Decider implementation
------------------------------------------------------------------------

||| Dashboard Decider: pure decision logic for dashboard operations
|||
||| Validation rules:
||| - CreateDashboard: Only when no dashboard exists
||| - AddChart: Only when dashboard exists
||| - RemoveChart: Only when dashboard exists (no check if chart exists - idempotent)
||| - AddTab: Only when dashboard exists
||| - MoveChartToTab: Only when dashboard exists
|||   (Full validation would check chart and tab exist, but kept simple for now)
||| - RenameDashboard: Only when dashboard exists
|||
||| Note: Timestamp generation is deferred to boundary layer (marked with holes)
public export
dashboardDecider : Decider DashboardCommand DashboardState DashboardEvent String
dashboardDecider = MkDecider
  { decide = \cmd, state => case (cmd, state.dashboardId) of
      (CreateDashboard name, Nothing) =>
        -- Generate new dashboard ID at boundary
        Right [DashboardCreated ?newDashId name ?now]
      (CreateDashboard _, Just _) =>
        Left "Dashboard already exists"

      (AddChart placement, Just _) =>
        -- Could validate: chart ID not already used, ChartDefinitionRef exists
        Right [ChartAdded placement ?now2]
      (AddChart _, Nothing) =>
        Left "No dashboard to add chart to"

      (RemoveChart chartId, Just _) =>
        -- Idempotent: removing non-existent chart is allowed
        Right [ChartRemoved chartId ?now3]
      (RemoveChart _, Nothing) =>
        Left "No dashboard"

      (AddTab tabName, Just _) =>
        -- Generate new tab ID at boundary
        Right [TabAdded (MkTabInfo ?newTabId tabName) ?now4]
      (AddTab _, Nothing) =>
        Left "No dashboard"

      (MoveChartToTab chartId tabId, Just _) =>
        -- Full validation would check: chart exists, tab exists
        -- Keeping simple for now - boundary can enforce stricter rules
        Right [ChartMovedToTab chartId tabId ?now5]
      (MoveChartToTab _ _, Nothing) =>
        Left "No dashboard"

      (RenameDashboard newName, Just _) =>
        Right [DashboardRenamed newName ?now6]
      (RenameDashboard _, Nothing) =>
        Left "No dashboard"

  , evolve = \state, event => case event of
      DashboardCreated did name _ =>
        { dashboardId := Just did, name := name } state

      ChartAdded placement _ =>
        { placements := placement :: state.placements } state

      ChartRemoved chartId _ =>
        { placements := filter (\p => p.chartId /= chartId) state.placements } state

      TabAdded tabInfo _ =>
        { tabs := tabInfo :: state.tabs } state

      ChartMovedToTab chartId tabId _ =>
        { placements := map (updateTabIfMatch chartId tabId) state.placements } state

      DashboardRenamed newName _ =>
        { name := newName } state

  , initialState = initialDashboardState
  }

------------------------------------------------------------------------
-- Invariants (postconditions, not enforced at compile time)
------------------------------------------------------------------------

-- Invariant: Dashboard creation makes dashboardId Just
-- Post: evolve (CreateDashboard name) Nothing => dashboardId is Just
--
-- This is a semantic invariant we expect to hold after evolve.

-- Invariant: Chart placements should not overlap
-- This would require checking position and size in decide.
-- For now, documented as intended validation for boundary layer.
--
-- Validation rule: For all placements p1, p2:
--   overlap(p1.position, p1.size, p2.position, p2.size) = False
--
-- Where overlap checks if rectangles intersect:
--   overlap(pos1, size1, pos2, size2) =
--     ¬ (pos1.row + size1.height <= pos2.row
--        || pos2.row + size2.height <= pos1.row
--        || pos1.col + size1.width <= pos2.col
--        || pos2.col + size2.width <= pos1.col)

-- Invariant: TabId references in ChartPlacement should be valid
-- For all placements p with p.tabId = Just tid:
--   tid ∈ map tabId state.tabs
--
-- This can be enforced in decide by checking hasTab before allowing
-- MoveChartToTab or AddChart with tabId set.
