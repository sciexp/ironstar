||| Effect boundaries for event sourcing
|||
||| This module defines the interfaces where pure domain logic meets IO.
||| The key insight is that Decider/View/Saga are pure, while
||| EventRepository operations are effectful (IO).
|||
||| Architecture:
||| ```
||| Pure Layer          │ Effectful Layer
||| ────────────────────┼────────────────────
||| Decider.decide      │ EventRepository.append
||| Decider.evolve      │ EventRepository.fetch
||| View.project        │ EventRepository.subscribe
||| Saga.react          │ Notification.publish
||| ```
|||
||| Reference: ~/projects/rust-workspace/fmodel-rust/src/aggregate.rs
module Core.Effect

import Data.List
import Core.Decider
import Core.View

%default total

------------------------------------------------------------------------
-- Event identification types
------------------------------------------------------------------------

||| Global event sequence number (monotonic)
||| Used for SSE Last-Event-ID semantics
public export
record EventId where
  constructor MkEventId
  unEventId : Integer

public export
Eq EventId where
  (MkEventId x) == (MkEventId y) = x == y

public export
Ord EventId where
  compare (MkEventId x) (MkEventId y) = compare x y

||| Aggregate identifier
public export
record AggregateId where
  constructor MkAggregateId
  aggregateType : String
  aggregateKey  : String

public export
Eq AggregateId where
  (MkAggregateId t1 k1) == (MkAggregateId t2 k2) = t1 == t2 && k1 == k2

||| Version for optimistic locking (per-aggregate sequence)
public export
record Version where
  constructor MkVersion
  unVersion : Nat

public export
Eq Version where
  (MkVersion x) == (MkVersion y) = x == y

public export
Ord Version where
  compare (MkVersion x) (MkVersion y) = compare x y

------------------------------------------------------------------------
-- Event with metadata
------------------------------------------------------------------------

||| Event wrapper with global sequence and version
public export
record VersionedEvent (e : Type) where
  constructor MkVersionedEvent
  eventId : EventId      -- Global monotonic sequence
  version : Version      -- Per-aggregate sequence (for optimistic locking)
  payload : e            -- The domain event

------------------------------------------------------------------------
-- EventRepository interface
------------------------------------------------------------------------

||| The EventRepository interface defines effectful operations for
||| persisting and retrieving events.
|||
||| This is the ONLY place where IO enters the event sourcing pipeline.
||| All other operations (decide, evolve, react, project) are pure.
|||
||| Type parameters:
||| - `e`   : Event type
||| - `err` : Error type for repository operations
|||
||| Design notes:
||| - Uses IO monad for effects (could upgrade to Eff for finer tracking)
||| - Returns Either for explicit error handling
||| - Operations correspond to fmodel-rust EventRepository trait
|||
||| Reference: fmodel-rust aggregate.rs:17-37
public export
interface EventRepository (e : Type) (err : Type) where
  ||| Append events to the store for a given aggregate.
  ||| Returns the versioned events with assigned sequence numbers.
  |||
  ||| Invariant: Assigned event IDs are strictly greater than all existing IDs.
  append : AggregateId -> List e -> IO (Either err (List (VersionedEvent e)))

  ||| Fetch all events for an aggregate.
  ||| Events are returned in sequence order (oldest first).
  fetch : AggregateId -> IO (Either err (List (VersionedEvent e)))

  ||| Fetch events after a given event ID (for SSE reconnection).
  ||| Used with Last-Event-ID header for resumable streams.
  fetchSince : EventId -> IO (Either err (List (VersionedEvent e)))

  ||| Fetch events for an aggregate after a given version (optimistic locking).
  ||| Used for conflict detection: if returned list is non-empty, concurrent write occurred.
  fetchSinceVersion : AggregateId -> Version -> IO (Either err (List (VersionedEvent e)))

------------------------------------------------------------------------
-- EventSourcedAggregate (wiring pure + effectful)
------------------------------------------------------------------------

||| EventSourcedAggregate combines a pure Decider with an effectful
||| EventRepository to create a complete aggregate implementation.
|||
||| The handle operation follows the pattern:
||| 1. Fetch current events (effect)
||| 2. Reconstruct state and decide new events (pure)
||| 3. Append new events (effect)
|||
||| Reference: fmodel-rust aggregate.rs:78-86, 168-177
public export
record EventSourcedAggregate (c : Type) (s : Type) (e : Type) (err : Type) where
  constructor MkAggregate
  decider : Decider c s e err

||| Handle a command using an EventSourcedAggregate.
|||
||| Pattern: fetch -> decide -> append
||| - Only three IO operations: fetch, append (and implicitly, version check)
||| - All decision logic is pure (Decider.decide)
|||
||| Note: This function takes explicit fetch and append functions rather than
||| using interface constraints, which simplifies the type signature.
||| Concrete implementations will provide these from an EventRepository instance.
public export
handleWith : (fetchFn : AggregateId -> IO (Either err (List (VersionedEvent e))))
          -> (appendFn : AggregateId -> List e -> IO (Either err (List (VersionedEvent e))))
          -> EventSourcedAggregate c s e err
          -> AggregateId
          -> c
          -> IO (Either err (List (VersionedEvent e)))
handleWith fetchFn appendFn agg aggId command = do
  fetchResult <- fetchFn aggId
  case fetchResult of
    Left err => pure (Left err)
    Right versionedEvents => do
      let events = map payload versionedEvents
      case computeNewEvents agg.decider events command of
        Left err => pure (Left err)
        Right newEvents =>
          if null newEvents
            then pure (Right [])  -- No-op: empty event list
            else appendFn aggId newEvents

------------------------------------------------------------------------
-- ViewRepository interface
------------------------------------------------------------------------

||| Interface for persisting view state (read model snapshots).
|||
||| Unlike EventRepository which stores events, ViewRepository stores
||| the current projected state. This is optional optimization:
||| views can always be rebuilt from events.
public export
interface ViewRepository (s : Type) (err : Type) where
  ||| Save the current projection state
  saveSnapshot : String -> s -> IO (Either err ())

  ||| Load the last saved projection state
  loadSnapshot : String -> IO (Either err (Maybe s))

------------------------------------------------------------------------
-- Notification interface (pub/sub)
------------------------------------------------------------------------

||| Interface for publishing events to subscribers.
|||
||| This is separate from EventRepository because:
||| 1. Persistence and notification are different concerns
||| 2. Notification might fail independently of persistence
||| 3. Different implementations (Zenoh, NATS, in-memory broadcast)
|||
||| Pattern: append THEN publish (durability first)
public export
interface EventNotifier (e : Type) where
  ||| Publish an event to all subscribers
  publish : e -> IO ()

  ||| Publish multiple events
  publishAll : List e -> IO ()
  publishAll = traverse_ publish

------------------------------------------------------------------------
-- Subscriber interface
------------------------------------------------------------------------

||| Interface for subscribing to event streams.
|||
||| Key pattern for SSE: subscribe BEFORE fetch to prevent race conditions.
||| Events arriving during fetch are buffered in the subscription.
public export
interface EventSubscriber (e : Type) where
  ||| Subscribe to events matching a key pattern
  ||| Returns an IO action that yields the next event (blocking)
  subscribe : String -> IO (IO e)

  ||| Unsubscribe from a key pattern
  unsubscribe : String -> IO ()

------------------------------------------------------------------------
-- Combined aggregate with notification
------------------------------------------------------------------------

||| Handle a command and publish resulting events.
|||
||| Extended handle that also notifies subscribers.
||| Pattern: fetch -> decide -> append -> publish
public export
handleWithPublish : (fetchFn : AggregateId -> IO (Either err (List (VersionedEvent e))))
                 -> (appendFn : AggregateId -> List e -> IO (Either err (List (VersionedEvent e))))
                 -> (publishFn : List e -> IO ())
                 -> EventSourcedAggregate c s e err
                 -> AggregateId
                 -> c
                 -> IO (Either err (List (VersionedEvent e)))
handleWithPublish fetchFn appendFn publishFn agg aggId command = do
  result <- handleWith fetchFn appendFn agg aggId command
  case result of
    Left err => pure (Left err)
    Right versionedEvents => do
      publishFn (map payload versionedEvents)
      pure (Right versionedEvents)

------------------------------------------------------------------------
-- Session-scoped operations (ironstar-specific)
------------------------------------------------------------------------

||| Session identifier (ironstar uses per-session event namespacing)
public export
record SessionId where
  constructor MkSessionId
  unSessionId : String

public export
Eq SessionId where
  (MkSessionId x) == (MkSessionId y) = x == y

||| Build Zenoh-style key expression for session events
||| Pattern: events/session/{session_id}/{aggregate_type}/{aggregate_id}
public export
sessionKeyExpr : SessionId -> AggregateId -> String
sessionKeyExpr (MkSessionId sid) (MkAggregateId atype aid) =
  "events/session/" ++ sid ++ "/" ++ atype ++ "/" ++ aid
