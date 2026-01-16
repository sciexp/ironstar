||| SavedQuery aggregate for named analytics queries
|||
||| The SavedQuery aggregate manages user-defined queries with parameters
||| that can be reused across sessions. Each query references a dataset
||| and contains SQL that will be executed by DuckDB.
|||
||| Key invariants:
||| - Query names are unique per user (enforced at boundary, not here)
||| - SQL is non-empty
||| - Dataset references are valid (enforced at runtime by DuckDB)
|||
||| Customer-Supplier relationship: Consumes dataset references from Analytics
module Workspace.SavedQuery

import Core.Decider
import Core.Event
import Data.List
import Workspace.WorkspaceAggregate  -- WorkspaceId

%default total

------------------------------------------------------------------------
-- Newtypes (Domain Vocabulary)
------------------------------------------------------------------------

||| Query name - human-readable identifier
public export
data QueryName = MkQueryName String

export
Show QueryName where
  show (MkQueryName n) = n

public export
Eq QueryName where
  (MkQueryName x) == (MkQueryName y) = x == y

||| SQL query text
public export
data SqlQuery = MkSqlQuery String

export
Show SqlQuery where
  show (MkSqlQuery s) = s

public export
Eq SqlQuery where
  (MkSqlQuery x) == (MkSqlQuery y) = x == y

||| Catalog URI reference
public export
data CatalogUri = MkCatalogUri String

export
Show CatalogUri where
  show (MkCatalogUri u) = u

public export
Eq CatalogUri where
  (MkCatalogUri x) == (MkCatalogUri y) = x == y

||| Dataset name within a catalog
public export
data DatasetName = MkDatasetName String

export
Show DatasetName where
  show (MkDatasetName n) = n

public export
Eq DatasetName where
  (MkDatasetName x) == (MkDatasetName y) = x == y

------------------------------------------------------------------------
-- Value Objects
------------------------------------------------------------------------

||| Unique identifier for a saved query
public export
record SavedQueryId where
  constructor MkSavedQueryId
  unSavedQueryId : String

public export
Eq SavedQueryId where
  (MkSavedQueryId x) == (MkSavedQueryId y) = x == y

||| Dataset reference decomposed into catalog and dataset components
|||
||| The catalogUri identifies the data catalog (e.g., "ducklake:hf://datasets/sciexp")
||| and datasetName identifies the specific dataset within that catalog (e.g., "fixtures").
||| This 2-field structure matches the canonical definition in Analytics.QuerySession
||| and enables filtering/grouping by catalog.
|||
||| Full URI reconstruction: catalogUri + "/" + datasetName
||| DuckDB resolves via httpfs or other extensions based on URI scheme.
public export
record DatasetRef where
  constructor MkDatasetRef
  catalogUri : CatalogUri
  datasetName : DatasetName

public export
Eq DatasetRef where
  (MkDatasetRef c1 d1) == (MkDatasetRef c2 d2) = c1 == c2 && d1 == d2

------------------------------------------------------------------------
-- Commands
------------------------------------------------------------------------

||| Commands for saved query management
public export
data SavedQueryCommand
  = SaveQuery WorkspaceId QueryName SqlQuery DatasetRef  -- workspaceId, name, sql, datasetRef
  | DeleteQuery
  | RenameQuery QueryName
  | UpdateQuerySql SqlQuery
  | UpdateDatasetRef DatasetRef

------------------------------------------------------------------------
-- Events
------------------------------------------------------------------------

||| Events representing saved query state changes
||| Law 1 (Hoffman): Events are past-tense and immutable
public export
data SavedQueryEvent
  = QuerySaved SavedQueryId WorkspaceId QueryName SqlQuery DatasetRef Timestamp
    -- id, workspaceId, name, sql, datasetRef, timestamp
  | QueryDeleted Timestamp
  | QueryRenamed QueryName Timestamp
  | QuerySqlUpdated SqlQuery Timestamp
  | DatasetRefUpdated DatasetRef Timestamp

------------------------------------------------------------------------
-- State
------------------------------------------------------------------------

||| SavedQuery aggregate state
||| Uses Maybe to represent existence (Nothing = query not created yet)
public export
data SavedQueryState
  = NoQuery
  | QueryExists SavedQueryId WorkspaceId QueryName SqlQuery DatasetRef
    -- id, workspaceId, name, sql, datasetRef

||| Initial state: no query saved
public export
initialSavedQueryState : SavedQueryState
initialSavedQueryState = NoQuery

------------------------------------------------------------------------
-- Helper functions
------------------------------------------------------------------------

||| Extract query ID if query exists
queryId : SavedQueryState -> Maybe SavedQueryId
queryId NoQuery = Nothing
queryId (QueryExists qid _ _ _ _) = Just qid

||| Extract workspace ID if query exists
queryWorkspaceId : SavedQueryState -> Maybe WorkspaceId
queryWorkspaceId NoQuery = Nothing
queryWorkspaceId (QueryExists _ wsId _ _ _) = Just wsId

||| Extract query name if query exists
queryName : SavedQueryState -> Maybe QueryName
queryName NoQuery = Nothing
queryName (QueryExists _ _ name _ _) = Just name

------------------------------------------------------------------------
-- Decider implementation
------------------------------------------------------------------------

||| SavedQuery Decider: pure decision logic for query operations
|||
||| Validation rules:
||| - SaveQuery: Only when no query exists (single query per aggregate)
||| - DeleteQuery: Only when query exists
||| - RenameQuery: Only when query exists
||| - UpdateQuerySql: Only when query exists
||| - UpdateDatasetRef: Only when query exists
|||
||| Law 7 (Hoffman): Work is a side effect
||| - decide and evolve are pure functions
||| - All I/O (ID generation, timestamp) happens at boundaries
|||
||| Note: This models each SavedQuery as a separate aggregate.
||| For multiple queries per user, use multiple SavedQuery aggregates
||| with different aggregate IDs (e.g., workspace_id/query_name).
public export
savedQueryDecider : Decider SavedQueryCommand SavedQueryState SavedQueryEvent String
savedQueryDecider = MkDecider
  { decide = \cmd, state => case (cmd, state) of
      (SaveQuery wsId name sql datasetRef, NoQuery) =>
        if name == MkQueryName "" then
          Left "Query name cannot be empty"
        else if sql == MkSqlQuery "" then
          Left "Query SQL cannot be empty"
        else
          -- Generate new query ID at boundary
          Right [QuerySaved ?newQueryId wsId name sql datasetRef ?now]
      (SaveQuery _ _ _ _, QueryExists _ _ _ _ _) =>
        Left "Query already exists"

      (DeleteQuery, QueryExists _ _ _ _ _) =>
        Right [QueryDeleted ?now2]
      (DeleteQuery, NoQuery) =>
        Left "No query to delete"

      (RenameQuery newName, QueryExists _ _ _ _ _) =>
        if newName == MkQueryName "" then
          Left "Query name cannot be empty"
        else
          Right [QueryRenamed newName ?now3]
      (RenameQuery _, NoQuery) =>
        Left "No query to rename"

      (UpdateQuerySql newSql, QueryExists _ _ _ _ _) =>
        if newSql == MkSqlQuery "" then
          Left "Query SQL cannot be empty"
        else
          Right [QuerySqlUpdated newSql ?now4]
      (UpdateQuerySql _, NoQuery) =>
        Left "No query to update"

      (UpdateDatasetRef newRef, QueryExists _ _ _ _ _) =>
        Right [DatasetRefUpdated newRef ?now5]
      (UpdateDatasetRef _, NoQuery) =>
        Left "No query to update"

  , evolve = \state, event => case event of
      QuerySaved qid wsId name sql datasetRef _ =>
        QueryExists qid wsId name sql datasetRef

      QueryDeleted _ =>
        NoQuery

      QueryRenamed newName _ =>
        case state of
          NoQuery => NoQuery  -- Should not happen (event invalid for this state)
          QueryExists qid wsId _ sql datasetRef =>
            QueryExists qid wsId newName sql datasetRef

      QuerySqlUpdated newSql _ =>
        case state of
          NoQuery => NoQuery  -- Should not happen
          QueryExists qid wsId name _ datasetRef =>
            QueryExists qid wsId name newSql datasetRef

      DatasetRefUpdated newRef _ =>
        case state of
          NoQuery => NoQuery  -- Should not happen
          QueryExists qid wsId name sql _ =>
            QueryExists qid wsId name sql newRef

  , initialState = initialSavedQueryState
  }

------------------------------------------------------------------------
-- Invariants (postconditions)
------------------------------------------------------------------------

-- Invariant: SaveQuery transitions from NoQuery to QueryExists
-- Post: evolve (QuerySaved ...) NoQuery => QueryExists

-- Invariant: DeleteQuery transitions from QueryExists to NoQuery
-- Post: evolve (QueryDeleted) (QueryExists ...) => NoQuery

-- Invariant: Query names are non-empty
-- Pre: SaveQuery name => name /= ""
-- Pre: RenameQuery name => name /= ""

-- Invariant: Query SQL is non-empty
-- Pre: SaveQuery sql => sql /= ""
-- Pre: UpdateQuerySql sql => sql /= ""

-- Invariant: Query names are unique per user
-- This is enforced at the boundary by using aggregate IDs like:
--   AggregateId "SavedQuery" "user_123/my-query-name"
-- Multiple SavedQuery aggregates per user, each with unique name in ID.
