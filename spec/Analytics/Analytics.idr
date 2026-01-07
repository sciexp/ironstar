||| Analytics bounded context
|||
||| Strategic classification: Core domain (primary differentiator)
|||
||| The Analytics bounded context manages scientific data analysis through
||| DuckLake catalog selection, query execution, and chart visualization.
|||
||| This module re-exports all Analytics types and provides combined Deciders
||| for coordinated aggregate handling.
|||
||| Key aggregates:
||| - Catalog: DuckLake catalog selection and versioning
||| - QuerySession: Query execution context and result caching
||| - ChartDefinition: Visualization specifications (value objects)
|||
||| Reference: docs/notes/architecture/core/bounded-contexts.md (Analytics section)
module Analytics.Analytics

import public Analytics.Catalog
import public Analytics.QuerySession
import public Analytics.Chart

import Core.Decider
import Core.View
import Core.Event

%default total

------------------------------------------------------------------------
-- Combined Decider
------------------------------------------------------------------------

||| Combined Decider for Analytics bounded context
|||
||| This enables coordinated handling of catalog and query commands
||| while maintaining aggregate independence (each has its own state).
|||
||| Uses Core.Decider.combine to compose independent deciders.
|||
||| Command type: Sum CatalogCommand QueryCommand
||| State type: (CatalogState, QueryState)
||| Event type: Sum CatalogEvent QueryEvent
public export
analyticsDecider : Decider (Sum CatalogCommand QueryCommand) (CatalogState, QueryState) (Sum CatalogEvent QueryEvent) String
analyticsDecider = combine catalogDecider queryDecider

------------------------------------------------------------------------
-- Combined Views (example)
------------------------------------------------------------------------

||| Combined read model: Catalog metadata + Query history
public export
record AnalyticsReadModel where
  constructor MkAnalyticsReadModel
  catalogMetadata : Maybe CatalogMetadata
  queryHistory : QueryHistory

||| View projecting combined Analytics events to combined read model
public export
analyticsView : View AnalyticsReadModel (Sum CatalogEvent QueryEvent)
analyticsView = MkView
  { evolve = \state, event => case event of
      First (CatalogSelected _ _) =>
        state  -- Catalog metadata comes from refresh event
      First (CatalogMetadataRefreshed metadata _) =>
        MkAnalyticsReadModel
          { catalogMetadata = Just metadata
          , queryHistory = state.queryHistory
          }
      Second evt =>
        let updatedHistory = (evolve queryHistoryView) state.queryHistory evt
        in MkAnalyticsReadModel
          { catalogMetadata = state.catalogMetadata
          , queryHistory = updatedHistory
          }
  , initialState = MkAnalyticsReadModel
      { catalogMetadata = Nothing
      , queryHistory = initialState queryHistoryView
      }
  }

------------------------------------------------------------------------
-- Zenoh key expressions
------------------------------------------------------------------------

||| Zenoh key expression pattern for Analytics events
||| Pattern: events/Analytics/{AggregateType}/{AggregateId}
public export
analyticsKeyPattern : String
analyticsKeyPattern = "events/Analytics/**"

||| Specific key pattern for Catalog events
public export
catalogKeyPattern : String
catalogKeyPattern = "events/Analytics/Catalog/**"

||| Specific key pattern for QuerySession events
public export
queryKeyPattern : String
queryKeyPattern = "events/Analytics/QuerySession/**"
