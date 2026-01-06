||| Event algebra and ordering invariants
|||
||| This module defines the algebraic structure of event streams
||| and encodes key invariants.
|||
||| Key invariants:
||| - Events form a free monoid under concatenation (append-only)
||| - State reconstruction is deterministic (same events = same state)
|||
||| Advanced invariants (MonotonicSequence, SessionIsolation) are sketched
||| but not fully proven in this initial version.
|||
||| Reference: Hoffman's Laws of Event Sourcing (Real World Event Sourcing)
module Core.Event

import Data.List
import Data.Nat
import Core.Effect

%default total

------------------------------------------------------------------------
-- Timestamp representation
------------------------------------------------------------------------

||| Unix timestamp in milliseconds
public export
record Timestamp where
  constructor MkTimestamp
  unTimestamp : Integer

public export
Eq Timestamp where
  (MkTimestamp x) == (MkTimestamp y) = x == y

public export
Ord Timestamp where
  compare (MkTimestamp x) (MkTimestamp y) = compare x y

------------------------------------------------------------------------
-- Event envelope (standard event wrapper)
------------------------------------------------------------------------

||| Standard event envelope with common metadata.
|||
||| All domain events should be wrapped in this envelope when stored.
||| The payload is polymorphic to support different bounded contexts.
|||
||| Note: This is separate from VersionedEvent (Core.Effect) which
||| focuses on repository concerns (version tracking for optimistic locking).
||| EventEnvelope focuses on domain metadata (timestamps, aggregate info).
public export
record EventEnvelope (payload : Type) where
  constructor MkEnvelope
  ||| Global monotonic sequence (assigned by store)
  envelopeEventId : EventId
  ||| Aggregate this event belongs to
  envelopeAggregateId : AggregateId
  ||| Per-aggregate sequence
  envelopeSequence : Nat
  ||| Event type discriminator (for deserialization routing)
  envelopeEventType : String
  ||| The actual domain event
  envelopePayload : payload
  ||| When the event occurred in the domain (business time)
  envelopeEventTime : Timestamp
  ||| When the event was recorded (system time)
  envelopeRecordedTime : Timestamp

------------------------------------------------------------------------
-- Monotonic ordering (simplified)
------------------------------------------------------------------------

||| Proof that one event ID is strictly less than another.
public export
data EventIdLT : EventId -> EventId -> Type where
  MkEventIdLT : {x, y : Integer} -> (0 prf : x < y = True)
             -> EventIdLT (MkEventId x) (MkEventId y)

||| Proof that a list of event IDs is monotonically increasing.
||| Simplified version without full dependent proof.
public export
data MonotonicIds : List EventId -> Type where
  MonoNil  : MonotonicIds []
  MonoOne  : (eid : EventId) -> MonotonicIds [eid]
  MonoCons : {eid : EventId} -> {eids : List EventId}
          -> MonotonicIds (eid :: eids)
          -> {auto 0 prf : NonEmpty eids}
          -> MonotonicIds eids

------------------------------------------------------------------------
-- Replay operations
------------------------------------------------------------------------

||| Filter events after a given event ID.
||| Used for SSE reconnection: replay only events the client hasn't seen.
public export
replayAfter : EventId -> List (EventEnvelope e) -> List (EventEnvelope e)
replayAfter (MkEventId threshold) = filter isAfter
  where
    isAfter : EventEnvelope e -> Bool
    isAfter env = case envelopeEventId env of
      MkEventId eid => eid > threshold

||| Filter events by domain time
public export
asOfEventTime : Timestamp -> List (EventEnvelope e) -> List (EventEnvelope e)
asOfEventTime ts = filter (\env => envelopeEventTime env <= ts)

||| Filter events by system time (what we knew at time T)
public export
asOfRecordedTime : Timestamp -> List (EventEnvelope e) -> List (EventEnvelope e)
asOfRecordedTime ts = filter (\env => envelopeRecordedTime env <= ts)

------------------------------------------------------------------------
-- Event algebra: free monoid structure
------------------------------------------------------------------------

||| Event logs form a free monoid under concatenation.
|||
||| Monoid laws:
||| - Identity: [] ++ xs = xs = xs ++ []
||| - Associativity: (xs ++ ys) ++ zs = xs ++ (ys ++ zs)
|||
||| This structure is fundamental: append-only by construction,
||| and the monoidal structure enables efficient composition.

||| Monoid identity law (left)
public export
0 appendLeftIdentity : (xs : List a) -> [] ++ xs = xs
appendLeftIdentity xs = Refl

||| Monoid identity law (right)
public export
0 appendRightIdentity : (xs : List a) -> xs ++ [] = xs
appendRightIdentity [] = Refl
appendRightIdentity (x :: xs) = cong (x ::) (appendRightIdentity xs)

||| Monoid associativity law
public export
0 appendAssociative : (xs, ys, zs : List a) -> (xs ++ ys) ++ zs = xs ++ (ys ++ zs)
appendAssociative [] ys zs = Refl
appendAssociative (x :: xs) ys zs = cong (x ::) (appendAssociative xs ys zs)

------------------------------------------------------------------------
-- Session isolation (placeholder)
------------------------------------------------------------------------

||| Predicate: session owns this event (based on aggregate ID pattern)
||| Placeholder implementation; real version would check Zenoh key expression.
public export
sessionOwns : SessionId -> EventEnvelope e -> Bool
sessionOwns _ _ = True  -- Placeholder

||| Filter events belonging to a session
public export
sessionEvents : SessionId -> List (EventEnvelope e) -> List (EventEnvelope e)
sessionEvents sid = filter (sessionOwns sid)

------------------------------------------------------------------------
-- Hoffman's Laws (as documentation)
------------------------------------------------------------------------

||| Law 1: Events are immutable and represent past-tense facts.
||| Enforced by: no mutation operations on EventEnvelope, append-only store.

||| Law 2: Event schemas are immutable.
||| Enforced by: closed event type definitions per aggregate.
||| Schema evolution uses versioned event types with upcasters.

||| Law 3: All projection data comes from events.
||| Enforced by: View takes only events as input.

||| Law 5: All projections stem from events.
||| Corollary of Law 3: projections are catamorphisms.

||| Law 6: Failure events preserve state.
||| Interface for aggregates to declare failure events.
public export
interface FailureEventPreservesState (e : Type) (s : Type) where
  ||| Predicate identifying failure events
  isFailureEvent : e -> Bool

  -- Note: Full proof would require:
  -- 0 failurePreserves : (st : s) -> (evt : e) -> (evolve : s -> e -> s)
  --                   -> isFailureEvent evt = True
  --                   -> evolve st evt = st

------------------------------------------------------------------------
-- SSE integration
------------------------------------------------------------------------

||| SSE event format for Last-Event-ID
public export
eventIdToString : EventId -> String
eventIdToString (MkEventId n) = show n

||| Parse event ID from Last-Event-ID header
||| Note: Uses cast which may produce 0 for invalid strings.
||| Production code should use proper string parsing.
public export
parseEventId : String -> Maybe EventId
parseEventId s = Just (MkEventId (cast s))
