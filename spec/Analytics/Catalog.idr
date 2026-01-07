||| Catalog aggregate for DuckLake catalog selection and versioning
|||
||| The Catalog aggregate manages the lifecycle of DuckLake catalog selections,
||| including catalog metadata refresh and version tracking.
|||
||| Key invariant: Only one catalog can be active at a time.
||| Catalog selection is a state machine: NoCatalogSelected → CatalogActive
|||
||| Reference: docs/notes/architecture/core/bounded-contexts.md (Analytics section)
module Analytics.Catalog

import Core.Decider
import Core.Event
import Data.List

%default total

------------------------------------------------------------------------
-- Value objects
------------------------------------------------------------------------

||| Reference to a DuckLake catalog
||| Example: "ducklake://hf/sciexp/fixtures"
public export
record CatalogRef where
  constructor MkCatalogRef
  uri : String

public export
Eq CatalogRef where
  (MkCatalogRef u1) == (MkCatalogRef u2) = u1 == u2

||| Information about a dataset in the catalog
public export
record DatasetInfo where
  constructor MkDatasetInfo
  name : String
  tableCount : Nat
  schemaVersion : String

public export
Eq DatasetInfo where
  (MkDatasetInfo n1 t1 s1) == (MkDatasetInfo n2 t2 s2) =
    n1 == n2 && t1 == t2 && s1 == s2

||| Metadata about a DuckLake catalog
public export
record CatalogMetadata where
  constructor MkCatalogMetadata
  datasets : List DatasetInfo
  lastRefreshed : Timestamp

------------------------------------------------------------------------
-- Commands
------------------------------------------------------------------------

||| Commands that can be issued to the Catalog aggregate
public export
data CatalogCommand
  = SelectCatalog CatalogRef
  | RefreshCatalogMetadata

------------------------------------------------------------------------
-- Events
------------------------------------------------------------------------

||| Events recording facts about Catalog state changes
public export
data CatalogEvent
  = CatalogSelected CatalogRef Timestamp
  | CatalogMetadataRefreshed CatalogMetadata Timestamp

------------------------------------------------------------------------
-- State
------------------------------------------------------------------------

||| Catalog aggregate state machine
||| Invariant: Only one catalog can be active at a time
public export
data CatalogState
  = NoCatalogSelected
  | CatalogActive CatalogRef CatalogMetadata

------------------------------------------------------------------------
-- Decider implementation
------------------------------------------------------------------------

||| Catalog Decider: pure command handling and event folding
public export
catalogDecider : Decider CatalogCommand CatalogState CatalogEvent String
catalogDecider = MkDecider
  { decide = \cmd, state => case (cmd, state) of
      (SelectCatalog ref, NoCatalogSelected) =>
        Right [CatalogSelected ref ?timestamp_select]
      (SelectCatalog ref, CatalogActive currentRef _) =>
        if currentRef == ref
          then Right []  -- No-op: catalog already selected
          else Left "Cannot change active catalog; deselect first"
      (RefreshCatalogMetadata, NoCatalogSelected) =>
        Left "No catalog selected"
      (RefreshCatalogMetadata, CatalogActive _ _) =>
        Right [CatalogMetadataRefreshed ?metadata ?timestamp_refresh]

  , evolve = \state, event => case event of
      CatalogSelected ref ts =>
        CatalogActive ref (MkCatalogMetadata [] ts)
      CatalogMetadataRefreshed metadata _ =>
        case state of
          NoCatalogSelected => state  -- Shouldn't happen, but defensive
          CatalogActive ref _ => CatalogActive ref metadata

  , initialState = NoCatalogSelected
  }
