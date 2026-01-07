||| UserPreferences aggregate for UI configuration
|||
||| The UserPreferences aggregate manages user-specific UI settings
||| that persist across sessions: theme selection, default catalog,
||| and arbitrary UI state (stored as JSON blob).
|||
||| Key invariants:
||| - Default catalog reference must be valid (enforced at runtime by DuckDB)
||| - UI state is a valid JSON string (enforced at boundary)
|||
||| Lifetime: One UserPreferences aggregate per authenticated user
module Workspace.Preferences

import Core.Decider
import Core.Event
import Data.List

%default total

------------------------------------------------------------------------
-- Value Objects
------------------------------------------------------------------------

||| Unique identifier for user preferences (typically matches user ID)
public export
record PreferencesId where
  constructor MkPreferencesId
  unPreferencesId : String

public export
Eq PreferencesId where
  (MkPreferencesId x) == (MkPreferencesId y) = x == y

||| Theme selection
public export
data Theme
  = Light
  | Dark
  | System  -- Defer to system preference

public export
Eq Theme where
  Light  == Light  = True
  Dark   == Dark   = True
  System == System = True
  _      == _      = False

||| Show instance for serialization
public export
Show Theme where
  show Light  = "Light"
  show Dark   = "Dark"
  show System = "System"

||| Parse theme from string (for deserialization)
public export
parseTheme : String -> Maybe Theme
parseTheme "Light"  = Just Light
parseTheme "Dark"   = Just Dark
parseTheme "System" = Just System
parseTheme _        = Nothing

||| Default catalog for DuckDB queries
||| References a catalog name that DuckDB can resolve
public export
record CatalogName where
  constructor MkCatalogName
  unCatalogName : String

public export
Eq CatalogName where
  (MkCatalogName x) == (MkCatalogName y) = x == y

------------------------------------------------------------------------
-- Commands
------------------------------------------------------------------------

||| Commands for user preferences management
public export
data PreferencesCommand
  = InitializePreferences  -- Create initial preferences with defaults
  | SetTheme Theme
  | SetDefaultCatalog CatalogName
  | ClearDefaultCatalog
  | UpdateUiState String  -- JSON blob for arbitrary UI state

------------------------------------------------------------------------
-- Events
------------------------------------------------------------------------

||| Events representing preferences state changes
public export
data PreferencesEvent
  = PreferencesInitialized PreferencesId Timestamp
  | ThemeSet Theme Timestamp
  | DefaultCatalogSet CatalogName Timestamp
  | DefaultCatalogCleared Timestamp
  | UiStateUpdated String Timestamp

------------------------------------------------------------------------
-- State
------------------------------------------------------------------------

||| UserPreferences aggregate state
public export
record PreferencesState where
  constructor MkPreferencesState
  preferencesId : Maybe PreferencesId
  theme : Theme
  defaultCatalog : Maybe CatalogName
  uiState : String  -- JSON blob, defaults to "{}"

||| Initial state: no preferences created yet
||| Default theme is System (defer to browser/OS)
public export
initialPreferencesState : PreferencesState
initialPreferencesState = MkPreferencesState Nothing System Nothing "{}"

------------------------------------------------------------------------
-- Decider implementation
------------------------------------------------------------------------

||| UserPreferences Decider: pure decision logic for preferences operations
|||
||| Validation rules:
||| - InitializePreferences: Only when preferences don't exist
||| - SetTheme: Only when preferences exist
||| - SetDefaultCatalog: Only when preferences exist
||| - ClearDefaultCatalog: Only when preferences exist
||| - UpdateUiState: Only when preferences exist
|||
||| Note: UI state validation (valid JSON) deferred to boundary layer
public export
preferencesDecider : Decider PreferencesCommand PreferencesState PreferencesEvent String
preferencesDecider = MkDecider
  { decide = \cmd, state => case (cmd, state.preferencesId) of
      (InitializePreferences, Nothing) =>
        -- Generate new preferences ID at boundary (typically matches user ID)
        Right [PreferencesInitialized ?newPreferencesId ?now]
      (InitializePreferences, Just _) =>
        Left "Preferences already initialized"

      (SetTheme theme, Just _) =>
        Right [ThemeSet theme ?now2]
      (SetTheme _, Nothing) =>
        Left "Preferences not initialized"

      (SetDefaultCatalog catalog, Just _) =>
        Right [DefaultCatalogSet catalog ?now3]
      (SetDefaultCatalog _, Nothing) =>
        Left "Preferences not initialized"

      (ClearDefaultCatalog, Just _) =>
        Right [DefaultCatalogCleared ?now4]
      (ClearDefaultCatalog, Nothing) =>
        Left "Preferences not initialized"

      (UpdateUiState jsonBlob, Just _) =>
        -- Validation of JSON syntax deferred to boundary
        -- Empty string is allowed (will be treated as "{}" at boundary)
        Right [UiStateUpdated jsonBlob ?now5]
      (UpdateUiState _, Nothing) =>
        Left "Preferences not initialized"

  , evolve = \state, event => case event of
      PreferencesInitialized pid _ =>
        { preferencesId := Just pid } state

      ThemeSet theme _ =>
        { theme := theme } state

      DefaultCatalogSet catalog _ =>
        { defaultCatalog := Just catalog } state

      DefaultCatalogCleared _ =>
        { defaultCatalog := Nothing } state

      UiStateUpdated jsonBlob _ =>
        { uiState := jsonBlob } state

  , initialState = initialPreferencesState
  }

------------------------------------------------------------------------
-- Invariants (postconditions)
------------------------------------------------------------------------

-- Invariant: InitializePreferences creates preferences ID
-- Post: evolve (PreferencesInitialized pid) state => preferencesId = Just pid

-- Invariant: Theme is always a valid Theme value
-- Enforced by type: theme : Theme

-- Invariant: UI state is valid JSON
-- Pre: UpdateUiState jsonBlob => isValidJson jsonBlob
-- Enforced at boundary layer, not here.

-- Invariant: Default catalog reference is valid
-- Runtime validation: DuckDB will fail query if catalog doesn't exist.
-- No compile-time enforcement.

-- Lifetime invariant: One PreferencesState per authenticated user
-- Enforced by aggregate ID construction:
--   AggregateId "UserPreferences" "user_123"
