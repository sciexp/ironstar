||| WorkspacePreferences aggregate for workspace-scoped settings
|||
||| The WorkspacePreferences aggregate manages settings specific to a single
||| workspace, such as default catalog selection and layout defaults.
||| This is distinct from UserPreferences (user-scoped, follows user across
||| all workspaces).
|||
||| Key invariants:
||| - PreferencesId references the workspace it belongs to
||| - CatalogName references a valid DuckDB catalog (enforced at boundary)
||| - LayoutDefaults is valid JSON (enforced at boundary)
|||
||| Law 1 (Hoffman): Events are past-tense and immutable
||| Law 7 (Hoffman): Work is a side effect - decide and evolve are pure
module Workspace.WorkspacePreferences

import Core.Decider
import Core.Event
import Workspace.WorkspaceAggregate  -- WorkspaceId

%default total

------------------------------------------------------------------------
-- Value Objects
------------------------------------------------------------------------

||| Catalog name for DuckDB default catalog selection
||| References a catalog registered in DuckDB (e.g., "ducklake:hf://datasets/sciexp")
public export
record CatalogName where
  constructor MkCatalogName
  unCatalogName : String

public export
Eq CatalogName where
  (MkCatalogName x) == (MkCatalogName y) = x == y

||| Unique identifier for workspace preferences
||| Typically derived from workspace ID
public export
record WorkspacePreferencesId where
  constructor MkWorkspacePreferencesId
  unWorkspacePreferencesId : String

public export
Eq WorkspacePreferencesId where
  (MkWorkspacePreferencesId x) == (MkWorkspacePreferencesId y) = x == y

------------------------------------------------------------------------
-- Commands
------------------------------------------------------------------------

||| Commands for workspace preferences management
||| These are workspace-scoped, not user-scoped
public export
data WorkspacePreferencesCommand
  = InitializeWorkspacePreferences WorkspaceId
  | SetWorkspaceDefaultCatalog CatalogName
  | ClearWorkspaceDefaultCatalog
  | UpdateLayoutDefaults String  -- JSON blob for layout defaults

------------------------------------------------------------------------
-- Events
------------------------------------------------------------------------

||| Events representing workspace preferences state changes
||| Law 1 (Hoffman): Events are past-tense and immutable
public export
data WorkspacePreferencesEvent
  = WorkspacePreferencesInitialized WorkspacePreferencesId WorkspaceId Timestamp
  | WorkspaceDefaultCatalogSet CatalogName Timestamp
  | WorkspaceDefaultCatalogCleared Timestamp
  | LayoutDefaultsUpdated String Timestamp

------------------------------------------------------------------------
-- State
------------------------------------------------------------------------

||| WorkspacePreferences aggregate state (workspace-scoped)
||| For user-scoped settings, see UserPreferences (Preferences.idr)
public export
record WorkspacePreferencesState where
  constructor MkWorkspacePreferencesState
  preferencesId : Maybe WorkspacePreferencesId
  workspaceId : Maybe WorkspaceId
  defaultCatalog : Maybe CatalogName
  layoutDefaults : String  -- JSON blob for layout defaults

||| Initial state: no preferences created yet
public export
initialWorkspacePreferencesState : WorkspacePreferencesState
initialWorkspacePreferencesState = MkWorkspacePreferencesState
  Nothing
  Nothing
  Nothing
  "{}"

------------------------------------------------------------------------
-- Decider implementation
------------------------------------------------------------------------

||| WorkspacePreferences Decider: pure decision logic for workspace-scoped preferences
|||
||| Validation rules:
||| - InitializeWorkspacePreferences: Only when preferences don't exist
||| - SetWorkspaceDefaultCatalog: Only when preferences exist
||| - ClearWorkspaceDefaultCatalog: Only when preferences exist
||| - UpdateLayoutDefaults: Only when preferences exist
|||
||| Law 7 (Hoffman): Work is a side effect
||| - decide and evolve are pure functions
||| - All I/O (catalog validation, JSON validation) happens at boundaries
public export
workspacePreferencesDecider : Decider WorkspacePreferencesCommand WorkspacePreferencesState WorkspacePreferencesEvent String
workspacePreferencesDecider = MkDecider
  { decide = \cmd, state => case (cmd, state.preferencesId) of
      (InitializeWorkspacePreferences wsId, Nothing) =>
        -- Generate new preferences ID at boundary
        Right [WorkspacePreferencesInitialized ?newPrefId wsId ?now]
      (InitializeWorkspacePreferences _, Just _) =>
        Left "Workspace preferences already initialized"

      (SetWorkspaceDefaultCatalog catalogName, Just _) =>
        -- Catalog validation deferred to boundary
        Right [WorkspaceDefaultCatalogSet catalogName ?now2]
      (SetWorkspaceDefaultCatalog _, Nothing) =>
        Left "Workspace preferences not initialized"

      (ClearWorkspaceDefaultCatalog, Just _) =>
        Right [WorkspaceDefaultCatalogCleared ?now3]
      (ClearWorkspaceDefaultCatalog, Nothing) =>
        Left "Workspace preferences not initialized"

      (UpdateLayoutDefaults jsonBlob, Just _) =>
        -- JSON validation deferred to boundary
        Right [LayoutDefaultsUpdated jsonBlob ?now4]
      (UpdateLayoutDefaults _, Nothing) =>
        Left "Workspace preferences not initialized"

  , evolve = \state, event => case event of
      WorkspacePreferencesInitialized prefId wsId _ =>
        { preferencesId := Just prefId
        , workspaceId := Just wsId
        } state

      WorkspaceDefaultCatalogSet catalogName _ =>
        { defaultCatalog := Just catalogName } state

      WorkspaceDefaultCatalogCleared _ =>
        { defaultCatalog := Nothing } state

      LayoutDefaultsUpdated jsonBlob _ =>
        { layoutDefaults := jsonBlob } state

  , initialState = initialWorkspacePreferencesState
  }

------------------------------------------------------------------------
-- Invariants (postconditions)
------------------------------------------------------------------------

-- Invariant: InitializeWorkspacePreferences creates preferences ID
-- Post: evolve (WorkspacePreferencesInitialized ...) state => preferencesId = Just prefId

-- Invariant: WorkspaceId is set on initialization and immutable
-- Post: evolve (WorkspacePreferencesInitialized _ wsId _) state => workspaceId = Just wsId

-- Invariant: CatalogName references valid DuckDB catalog
-- Enforced at boundary layer (validation on input)

-- Invariant: LayoutDefaults is valid JSON
-- Enforced at boundary layer (validation on input)

-- Scope: Workspace-scoped (belongs to single workspace)
-- For user-scoped settings like theme/locale, see UserPreferences (Preferences.idr)
