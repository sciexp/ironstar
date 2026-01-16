||| Workspace aggregate for multi-tenant workspace management
|||
||| The WorkspaceAggregate manages workspace lifecycle including identity,
||| naming, ownership, and visibility settings. Workspaces serve as the
||| organizational container for dashboards, saved queries, and preferences.
|||
||| Key invariants:
||| - WorkspaceId is immutable after creation
||| - OwnerId references a valid UserId from SharedKernel
||| - Visibility controls access permissions (Public/Private)
|||
||| Law 1 (Hoffman): Events are past-tense and immutable
||| Law 7 (Hoffman): Work is a side effect - decide and evolve are pure
module Workspace.WorkspaceAggregate

import Core.Decider
import Core.Event
import SharedKernel.UserId

%default total

------------------------------------------------------------------------
-- Value Objects
------------------------------------------------------------------------

||| Unique identifier for a workspace
||| Local to Workspace bounded context
public export
record WorkspaceId where
  constructor MkWorkspaceId
  unWorkspaceId : String

public export
Eq WorkspaceId where
  (MkWorkspaceId x) == (MkWorkspaceId y) = x == y

||| Workspace name (user-facing display name)
public export
record WorkspaceName where
  constructor MkWorkspaceName
  unWorkspaceName : String

public export
Eq WorkspaceName where
  (MkWorkspaceName x) == (MkWorkspaceName y) = x == y

||| Workspace visibility controls access permissions
||| Public: visible to all authenticated users
||| Private: visible only to owner (and explicitly shared users, future)
public export
data Visibility = Public | Private

public export
Eq Visibility where
  Public  == Public  = True
  Private == Private = True
  _       == _       = False

public export
Show Visibility where
  show Public  = "Public"
  show Private = "Private"

||| Parse visibility from string (for deserialization)
public export
parseVisibility : String -> Maybe Visibility
parseVisibility "Public"  = Just Public
parseVisibility "Private" = Just Private
parseVisibility _         = Nothing

------------------------------------------------------------------------
-- Commands
------------------------------------------------------------------------

||| Commands for workspace lifecycle management
public export
data WorkspaceCommand
  = CreateWorkspace WorkspaceName (Maybe UserId) Visibility
  | RenameWorkspace WorkspaceName
  | SetVisibility Visibility

------------------------------------------------------------------------
-- Events
------------------------------------------------------------------------

||| Events representing workspace state changes
||| Law 1 (Hoffman): Events are past-tense and immutable
public export
data WorkspaceEvent
  = WorkspaceCreated WorkspaceId WorkspaceName (Maybe UserId) Visibility Timestamp
  | WorkspaceRenamed WorkspaceName Timestamp
  | VisibilityChanged Visibility Timestamp

------------------------------------------------------------------------
-- State
------------------------------------------------------------------------

||| Workspace aggregate state
||| Invariant: If workspaceId is Just, workspace exists
public export
record WorkspaceState where
  constructor MkWorkspaceState
  workspaceId : Maybe WorkspaceId
  name : WorkspaceName
  ownerId : Maybe UserId  -- From SharedKernel.UserId
  visibility : Visibility
  createdAt : Maybe Timestamp
  updatedAt : Maybe Timestamp

||| Initial state: no workspace created yet
public export
initialWorkspaceState : WorkspaceState
initialWorkspaceState = MkWorkspaceState
  Nothing
  (MkWorkspaceName "")
  Nothing
  Private
  Nothing
  Nothing

------------------------------------------------------------------------
-- Decider implementation
------------------------------------------------------------------------

||| Workspace Decider: pure decision logic for workspace operations
|||
||| Validation rules:
||| - CreateWorkspace: Only when no workspace exists
||| - RenameWorkspace: Only when workspace exists
||| - SetVisibility: Only when workspace exists
|||
||| Law 7 (Hoffman): Work is a side effect
||| - decide and evolve are pure functions
||| - All I/O (ID generation, timestamp) happens at boundaries (marked with holes)
public export
workspaceDecider : Decider WorkspaceCommand WorkspaceState WorkspaceEvent String
workspaceDecider = MkDecider
  { decide = \cmd, state => case (cmd, state.workspaceId) of
      (CreateWorkspace wsName ownerId vis, Nothing) =>
        -- Generate new workspace ID and timestamp at boundary
        Right [WorkspaceCreated ?newWsId wsName ownerId vis ?now]
      (CreateWorkspace _ _ _, Just _) =>
        Left "Workspace already exists"

      (RenameWorkspace newName, Just _) =>
        Right [WorkspaceRenamed newName ?now2]
      (RenameWorkspace _, Nothing) =>
        Left "No workspace to rename"

      (SetVisibility vis, Just _) =>
        Right [VisibilityChanged vis ?now3]
      (SetVisibility _, Nothing) =>
        Left "No workspace to modify"

  , evolve = \state, event => case event of
      WorkspaceCreated wsId wsName ownerId vis ts =>
        { workspaceId := Just wsId
        , name := wsName
        , ownerId := ownerId
        , visibility := vis
        , createdAt := Just ts
        , updatedAt := Just ts
        } state

      WorkspaceRenamed newName ts =>
        { name := newName
        , updatedAt := Just ts
        } state

      VisibilityChanged vis ts =>
        { visibility := vis
        , updatedAt := Just ts
        } state

  , initialState = initialWorkspaceState
  }

------------------------------------------------------------------------
-- Invariants (postconditions)
------------------------------------------------------------------------

-- Invariant: CreateWorkspace creates workspace ID
-- Post: evolve (WorkspaceCreated wsId ...) state => workspaceId = Just wsId

-- Invariant: WorkspaceId is immutable after creation
-- No command or event can change workspaceId once set

-- Invariant: Visibility is always a valid Visibility value
-- Enforced by type: visibility : Visibility

-- Invariant: OwnerId references valid UserId from SharedKernel
-- Enforced at boundary via SharedKernel.UserId import
