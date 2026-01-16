||| UserPreferences aggregate for user-scoped UI configuration
|||
||| The UserPreferences aggregate manages personal settings that follow the
||| user across ALL workspaces they access. These are user-scoped, not
||| workspace-scoped: theme selection, locale, and arbitrary UI state
||| (stored as JSON blob for collapsed panels, sidebar width, etc.).
|||
||| For workspace-specific settings like default catalog, see WorkspacePreferences.
|||
||| Key invariants:
||| - UI state is a valid JSON string (enforced at boundary)
|||
||| Lifetime: One UserPreferences aggregate per authenticated user
||| Aggregate ID: user_{user_id}/preferences
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

||| Locale identifier for internationalization
||| Uses BCP-47 language tags (e.g., "en-US", "de-DE")
public export
record Locale where
  constructor MkLocale
  unLocale : String

public export
Eq Locale where
  (MkLocale x) == (MkLocale y) = x == y

||| Default locale (US English)
public export
defaultLocale : Locale
defaultLocale = MkLocale "en-US"

------------------------------------------------------------------------
-- Commands
------------------------------------------------------------------------

||| Commands for user preferences management
||| Note: For workspace-scoped settings like default catalog, see WorkspacePreferences
public export
data PreferencesCommand
  = InitializePreferences  -- Create initial preferences with defaults
  | SetTheme Theme
  | SetLocale Locale
  | UpdateUiState String  -- JSON blob for arbitrary UI state (collapsed panels, etc.)

------------------------------------------------------------------------
-- Events
------------------------------------------------------------------------

||| Events representing preferences state changes
||| Note: For workspace-scoped events like DefaultCatalogSet, see WorkspacePreferences
public export
data PreferencesEvent
  = PreferencesInitialized PreferencesId Timestamp
  | ThemeSet Theme Timestamp
  | LocaleSet Locale Timestamp
  | UiStateUpdated String Timestamp

------------------------------------------------------------------------
-- State
------------------------------------------------------------------------

||| UserPreferences aggregate state (user-scoped, follows user across all workspaces)
||| For workspace-scoped settings like defaultCatalog, see WorkspacePreferences
public export
record PreferencesState where
  constructor MkPreferencesState
  preferencesId : Maybe PreferencesId
  theme : Theme
  locale : Locale
  uiState : String  -- JSON blob for UI state (collapsed panels, sidebar width, etc.)

||| Initial state: no preferences created yet
||| Default theme is System (defer to browser/OS), default locale is en-US
public export
initialPreferencesState : PreferencesState
initialPreferencesState = MkPreferencesState Nothing System defaultLocale "{}"

------------------------------------------------------------------------
-- Decider implementation
------------------------------------------------------------------------

||| UserPreferences Decider: pure decision logic for user-scoped preferences
|||
||| Validation rules:
||| - InitializePreferences: Only when preferences don't exist
||| - SetTheme: Only when preferences exist
||| - SetLocale: Only when preferences exist
||| - UpdateUiState: Only when preferences exist
|||
||| Note: UI state validation (valid JSON) deferred to boundary layer
||| For workspace-scoped operations like SetDefaultCatalog, see WorkspacePreferences
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

      (SetLocale locale, Just _) =>
        Right [LocaleSet locale ?now3]
      (SetLocale _, Nothing) =>
        Left "Preferences not initialized"

      (UpdateUiState jsonBlob, Just _) =>
        -- Validation of JSON syntax deferred to boundary
        -- Empty string is allowed (will be treated as "{}" at boundary)
        Right [UiStateUpdated jsonBlob ?now4]
      (UpdateUiState _, Nothing) =>
        Left "Preferences not initialized"

  , evolve = \state, event => case event of
      PreferencesInitialized pid _ =>
        { preferencesId := Just pid } state

      ThemeSet theme _ =>
        { theme := theme } state

      LocaleSet locale _ =>
        { locale := locale } state

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

-- Invariant: Locale is a valid BCP-47 language tag
-- Enforced at boundary layer (validation on input)

-- Invariant: UI state is valid JSON
-- Pre: UpdateUiState jsonBlob => isValidJson jsonBlob
-- Enforced at boundary layer, not here.

-- Lifetime invariant: One PreferencesState per authenticated user
-- Enforced by aggregate ID construction:
--   AggregateId "UserPreferences" "user_{user_id}/preferences"
--
-- Scope: User-scoped (follows user across ALL workspaces)
-- For workspace-scoped settings like default catalog, see WorkspacePreferences
