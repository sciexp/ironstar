||| Session bounded context for authentication lifecycle management
|||
||| This module formalizes the Session aggregate using the Decider pattern.
||| Session is a Supporting domain that provides authenticated User identity
||| to other bounded contexts via the Shared Kernel pattern.
|||
||| Strategic classification: Supporting domain
||| - Manages OAuth-based authentication (GitHub first, Google planned)
||| - Session lifecycle: create → active → expired/invalidated
||| - Provides User identity to Workspace via Shared Kernel
|||
||| Key invariants:
||| 1. Session expiration: Active state requires ExpiresAt > current time
|||    (enforced at boundary layer, not in pure decision logic)
||| 2. State transitions: Cannot refresh Expired/Invalidated session
||| 3. Provider consistency: UserId.provider matches session's OAuthProvider
|||
||| Dependencies: Core.Decider, Core.View, Core.Event, Core.Effect
module Session.Session

import Core.Decider
import Core.View
import Core.Event
import Core.Effect
import Data.List

%default total

------------------------------------------------------------------------
-- Value Objects
------------------------------------------------------------------------

||| OAuth provider enumeration
||| GitHub is primary provider, Google planned as future extension
public export
data OAuthProvider = GitHub | Google

public export
Eq OAuthProvider where
  GitHub == GitHub = True
  Google == Google = True
  _ == _ = False

public export
Show OAuthProvider where
  show GitHub = "GitHub"
  show Google = "Google"

||| User identifier combining OAuth provider and external ID
||| Shared Kernel: exported to Workspace for user identity
public export
record UserId where
  constructor MkUserId
  provider : OAuthProvider
  externalId : String

public export
Eq UserId where
  (MkUserId p1 id1) == (MkUserId p2 id2) = p1 == p2 && id1 == id2

||| Expiration timestamp for session TTL enforcement
public export
record ExpiresAt where
  constructor MkExpiresAt
  unExpiresAt : Integer

public export
Eq ExpiresAt where
  (MkExpiresAt x) == (MkExpiresAt y) = x == y

public export
Ord ExpiresAt where
  compare (MkExpiresAt x) (MkExpiresAt y) = compare x y

------------------------------------------------------------------------
-- Commands
------------------------------------------------------------------------

||| Session lifecycle commands
||| Pattern: OAuth callback → create → periodic refresh → terminate/expire
public export
data SessionCommand
  = CreateSession UserId OAuthProvider
  | RefreshSession SessionId
  | InvalidateSession SessionId

------------------------------------------------------------------------
-- Events
------------------------------------------------------------------------

||| Session events representing immutable facts about authentication state
||| Law 1 (Hoffman): Events are past-tense and immutable
public export
data SessionEvent
  = SessionCreated SessionId UserId OAuthProvider Timestamp ExpiresAt
  | SessionRefreshed SessionId ExpiresAt Timestamp
  | SessionInvalidated SessionId Timestamp
  | SessionExpired SessionId Timestamp

------------------------------------------------------------------------
-- State
------------------------------------------------------------------------

||| Session aggregate state
||| State machine: NoSession → Active → (Expired | Invalidated)
|||
||| Invariant enforcement:
||| - ExpiresAt > current time checked at boundary (not in evolve)
||| - Cannot transition from Expired/Invalidated back to Active
public export
data SessionState
  = NoSession
  | Active SessionId UserId ExpiresAt
  | Expired SessionId
  | Invalidated SessionId

------------------------------------------------------------------------
-- Decider
------------------------------------------------------------------------

||| Session aggregate Decider
|||
||| Pure decision logic for session lifecycle management.
||| Effect boundaries (OAuth token exchange, timestamp generation, SessionId creation)
||| are provided via holes that will be filled at the boundary layer.
|||
||| Invariants enforced by types:
||| - decide returns Either String (validation errors are explicit)
||| - evolve is total (events are historical facts, cannot fail)
||| - State transitions respect session lifecycle constraints
|||
||| Law 7 (Hoffman): Work is a side effect
||| - decide and evolve are pure functions
||| - All I/O (token validation, timestamp generation) happens at boundaries
public export
sessionDecider : Decider SessionCommand SessionState SessionEvent String
sessionDecider = MkDecider
  { decide = \cmd, state => case (cmd, state) of
      -- Create new session when none exists
      -- Holes: ?newSid (SessionId), ?now (Timestamp), ?expires (ExpiresAt)
      -- Boundary fills these from OAuth callback context
      (CreateSession userId provider, NoSession) =>
        Right [SessionCreated ?newSid userId provider ?now ?expires]
      (CreateSession _ _, Active _ _ _) =>
        Left "Session already active"
      (CreateSession _ _, Expired _) =>
        Left "Cannot create session: previous session expired"
      (CreateSession _ _, Invalidated _) =>
        Left "Cannot create session: previous session invalidated"

      -- Refresh active session (extend TTL)
      -- Hole: ?newExpires (ExpiresAt), ?now2 (Timestamp)
      (RefreshSession sid, Active activeSid _ _) =>
        if sid == activeSid
          then Right [SessionRefreshed sid ?newExpires ?now2]
          else Left "Session ID mismatch"
      (RefreshSession _, NoSession) =>
        Left "No active session to refresh"
      (RefreshSession _, Expired _) =>
        Left "Cannot refresh expired session"
      (RefreshSession _, Invalidated _) =>
        Left "Cannot refresh invalidated session"

      -- Invalidate active session (explicit logout)
      -- Hole: ?now3 (Timestamp)
      (InvalidateSession sid, Active activeSid _ _) =>
        if sid == activeSid
          then Right [SessionInvalidated sid ?now3]
          else Left "Session ID mismatch"
      (InvalidateSession _, NoSession) =>
        Left "No active session to invalidate"
      (InvalidateSession _, Expired _) =>
        Left "Session already expired"
      (InvalidateSession _, Invalidated _) =>
        Left "Session already invalidated"

  , evolve = \state, event => case event of
      -- SessionCreated: transition NoSession → Active
      SessionCreated sid userId _ _ expires => Active sid userId expires

      -- SessionRefreshed: extend TTL in Active state
      -- Invariant: only Active states should see Refreshed events
      SessionRefreshed sid newExpires _ => case state of
        Active _ userId _ => Active sid userId newExpires
        other => other  -- Defensive: shouldn't happen if decide is correct

      -- SessionInvalidated: transition Active → Invalidated
      SessionInvalidated sid _ => Invalidated sid

      -- SessionExpired: transition Active → Expired
      -- Note: SessionExpired events are generated by boundary layer
      -- when expiration TTL is reached, not by decide function
      SessionExpired sid _ => Expired sid

  , initialState = NoSession
  }

------------------------------------------------------------------------
-- View: Active Session Lookup
------------------------------------------------------------------------

||| View state for quick session lookup by SessionId
||| Optimized read model for authentication checks
public export
record ActiveSessionView where
  constructor MkActiveSessionView
  activeSession : Maybe (SessionId, UserId, ExpiresAt)

||| View for active session projection
|||
||| Law 3 (Hoffman): All projection data comes from events
||| Law 5 (Hoffman): All projections stem from events
|||
||| This View is disposable and can be rebuilt from event stream.
public export
activeSessionView : View ActiveSessionView SessionEvent
activeSessionView = MkView
  { evolve = \state, event => case event of
      -- New session created: project into active lookup
      SessionCreated sid userId _ _ expires =>
        MkActiveSessionView (Just (sid, userId, expires))

      -- Session refreshed: update expiration in projection
      SessionRefreshed sid newExpires _ => case state.activeSession of
        Just (_, userId, _) => MkActiveSessionView (Just (sid, userId, newExpires))
        Nothing => state  -- Defensive: shouldn't happen with valid event stream

      -- Session invalidated: remove from active lookup
      SessionInvalidated _ _ => MkActiveSessionView Nothing

      -- Session expired: remove from active lookup
      SessionExpired _ _ => MkActiveSessionView Nothing
  , initialState = MkActiveSessionView Nothing
  }

------------------------------------------------------------------------
-- Helper predicates for boundary layer
------------------------------------------------------------------------

||| Check if session state is currently active
||| Boundary layer should additionally check ExpiresAt > current time
public export
isActive : SessionState -> Bool
isActive (Active _ _ _) = True
isActive _ = False

||| Check if session state is terminated (expired or invalidated)
public export
isTerminated : SessionState -> Bool
isTerminated (Expired _) = True
isTerminated (Invalidated _) = True
isTerminated _ = False

||| Extract SessionId from any non-NoSession state
public export
getSessionId : SessionState -> Maybe SessionId
getSessionId NoSession = Nothing
getSessionId (Active sid _ _) = Just sid
getSessionId (Expired sid) = Just sid
getSessionId (Invalidated sid) = Just sid

||| Extract UserId from Active state
public export
getUserId : SessionState -> Maybe UserId
getUserId (Active _ userId _) = Just userId
getUserId _ = Nothing
