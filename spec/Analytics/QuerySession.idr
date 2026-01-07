||| QuerySession aggregate for query execution and result management
|||
||| The QuerySession aggregate manages the lifecycle of analytical queries,
||| including execution state tracking and result caching.
|||
||| Key invariant: QuerySession commands require CatalogActive state.
||| Query execution is a state machine: Idle → Running → (Completed | Failed)
|||
||| Reference: docs/notes/architecture/core/bounded-contexts.md (Analytics section)
module Analytics.QuerySession

import Core.Decider
import Core.View
import Core.Event
import Core.Effect
import Data.List

%default total

------------------------------------------------------------------------
-- Value objects
------------------------------------------------------------------------

||| Reference to a dataset within a catalog
public export
record DatasetRef where
  constructor MkDatasetRef
  catalogUri : String
  datasetName : String

public export
Eq DatasetRef where
  (MkDatasetRef c1 d1) == (MkDatasetRef c2 d2) = c1 == c2 && d1 == d2

||| SQL query string
public export
record SqlQuery where
  constructor MkSqlQuery
  sql : String

||| Unique identifier for a query execution
public export
record QueryId where
  constructor MkQueryId
  unQueryId : String

public export
Eq QueryId where
  (MkQueryId q1) == (MkQueryId q2) = q1 == q2

||| Query execution results
||| Simplified representation; real implementation would include row data
public export
record QueryResults where
  constructor MkQueryResults
  rowCount : Nat
  columnNames : List String

||| Query execution duration
public export
record Duration where
  constructor MkDuration
  milliseconds : Integer

||| Error message from failed query execution
public export
record ErrorMessage where
  constructor MkErrorMessage
  message : String

||| Chart configuration (imported from Analytics.Chart)
||| Forward declaration to avoid circular imports
public export
record ChartConfig where
  constructor MkChartConfig
  chartType : String  -- Simplified; see Analytics.Chart for full type

------------------------------------------------------------------------
-- Commands
------------------------------------------------------------------------

||| Commands that can be issued to the QuerySession aggregate
public export
data QueryCommand
  = StartQuery DatasetRef SqlQuery (Maybe ChartConfig)
  | CancelQuery QueryId

------------------------------------------------------------------------
-- Events
------------------------------------------------------------------------

||| Events recording facts about QuerySession state changes
public export
data QueryEvent
  = QueryStarted QueryId DatasetRef SqlQuery (Maybe ChartConfig) Timestamp
  | QueryCompleted QueryId QueryResults Duration Timestamp
  | QueryFailed QueryId ErrorMessage Timestamp
  | QueryCancelled QueryId Timestamp

------------------------------------------------------------------------
-- State
------------------------------------------------------------------------

||| QuerySession aggregate state machine
||| Invariant: Only one query can be running at a time (simplified)
public export
data QueryState
  = Idle
  | Running QueryId DatasetRef SqlQuery
  | Completed QueryId QueryResults
  | Failed QueryId ErrorMessage

------------------------------------------------------------------------
-- Decider implementation
------------------------------------------------------------------------

||| QuerySession Decider: pure command handling and event folding
|||
||| Note: In production, StartQuery command would include CatalogState
||| as a precondition check (dependent types or runtime validation).
||| For this spec, we assume catalog validation happens upstream.
public export
queryDecider : Decider QueryCommand QueryState QueryEvent String
queryDecider = MkDecider
  { decide = \cmd, state => case (cmd, state) of
      (StartQuery dsRef query chartCfg, Idle) =>
        let qid = ?generateQueryId_idle
        in Right [QueryStarted qid dsRef query chartCfg ?timestamp_start_idle]
      (StartQuery _ _ _, Running _ _ _) =>
        Left "Query already running; cancel or wait for completion"
      (StartQuery dsRef query chartCfg, Completed _ _) =>
        let qid = ?generateQueryId_completed
        in Right [QueryStarted qid dsRef query chartCfg ?timestamp_start_completed]
      (StartQuery dsRef query chartCfg, Failed _ _) =>
        let qid = ?generateQueryId_failed
        in Right [QueryStarted qid dsRef query chartCfg ?timestamp_start_failed]

      (CancelQuery qid, Running currentId _ _) =>
        if currentId == qid
          then Right [QueryCancelled qid ?timestamp_cancel]
          else Left "Query ID does not match current running query"
      (CancelQuery _, Idle) =>
        Left "No query running"
      (CancelQuery _, Completed _ _) =>
        Left "Cannot cancel completed query"
      (CancelQuery _, Failed _ _) =>
        Left "Cannot cancel failed query"

  , evolve = \state, event => case event of
      QueryStarted qid dsRef query _ _ =>
        Running qid dsRef query
      QueryCompleted qid results _ _ =>
        Completed qid results
      QueryFailed qid err _ =>
        Failed qid err
      QueryCancelled _ _ =>
        Idle

  , initialState = Idle
  }

------------------------------------------------------------------------
-- Query result projection (View)
------------------------------------------------------------------------

||| Read model: history of completed queries
public export
record QueryHistory where
  constructor MkQueryHistory
  completedQueries : List (QueryId, QueryResults)
  failedQueries : List (QueryId, ErrorMessage)

||| View for query history projection
public export
queryHistoryView : View QueryHistory QueryEvent
queryHistoryView = MkView
  { evolve = \state, event => case event of
      QueryCompleted qid results _ _ =>
        MkQueryHistory
          { completedQueries = (qid, results) :: state.completedQueries
          , failedQueries = state.failedQueries
          }
      QueryFailed qid err _ =>
        MkQueryHistory
          { completedQueries = state.completedQueries
          , failedQueries = (qid, err) :: state.failedQueries
          }
      _ => state  -- Ignore QueryStarted, QueryCancelled

  , initialState = MkQueryHistory [] []
  }
