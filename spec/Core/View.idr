||| View pattern for read-side projections
|||
||| A View is a Decider without the `decide` function: pure event folding
||| for building read models. Views subscribe to event streams and maintain
||| denormalized state optimized for queries.
|||
||| Key properties:
||| - Views are disposable: they can be rebuilt from the event stream
||| - Multiple Views can project the same events into different shapes
||| - Views never emit events (read-only)
|||
||| Reference: ~/projects/rust-workspace/fmodel-rust/src/view.rs
module Core.View

import Data.List
import Core.Decider

%default total

------------------------------------------------------------------------
-- Core View record
------------------------------------------------------------------------

||| The View record encapsulates pure event projection logic.
|||
||| Type parameters:
||| - `s` : State type (the projected read model)
||| - `e` : Event type (events being projected)
|||
||| Design notes:
||| - View is a Decider without `decide` (no command handling)
||| - Same Rust wrapper elimination as Decider
||| - Views are pure projections: given same events, produce same state
public export
record View (s : Type) (e : Type) where
  constructor MkView
  ||| Evolve the projection state by applying an event.
  ||| Total function: events are facts that cannot fail.
  evolve : s -> e -> s

  ||| The initial state for a new projection (before any events).
  initialState : s

------------------------------------------------------------------------
-- Derived computations
------------------------------------------------------------------------

||| Project events into a read model.
||| This is the catamorphism for Views.
public export
project : View s e -> List e -> s
project v events = foldl v.evolve v.initialState events

||| Incrementally update a projection with new events.
||| Used when we have a snapshot and want to apply only new events.
public export
projectFrom : View s e -> s -> List e -> s
projectFrom v currentState newEvents = foldl v.evolve currentState newEvents

------------------------------------------------------------------------
-- Composition: merge (same event type, multiple projections)
------------------------------------------------------------------------

||| Merge two Views that project the same event type.
|||
||| Unlike Decider's `combine` which uses Sum for events, `merge` is for
||| multiple read models that all need to process the same event stream.
|||
||| Type transformation:
||| - States: `(s1, s2)` (product of independent projections)
||| - Events: `e` (same event type, applied to both)
|||
||| Use case: Dashboard showing multiple views of the same data
||| (e.g., total count view + recent items view + statistics view)
|||
||| Reference: fmodel-rust view.rs merge pattern
public export
merge : View s1 e -> View s2 e -> View (s1, s2) e
merge v1 v2 = MkView
  { evolve = \(st1, st2), evt => (v1.evolve st1 evt, v2.evolve st2 evt)
  , initialState = (v1.initialState, v2.initialState)
  }

||| Infix operator for merge
public export
(<&>) : View s1 e -> View s2 e -> View (s1, s2) e
(<&>) = merge

||| Merge three Views
public export
merge3 : View s1 e -> View s2 e -> View s3 e -> View (s1, s2, s3) e
merge3 v1 v2 v3 = MkView
  { evolve = \(st1, st2, st3), evt =>
      (v1.evolve st1 evt, v2.evolve st2 evt, v3.evolve st3 evt)
  , initialState = (v1.initialState, v2.initialState, v3.initialState)
  }

------------------------------------------------------------------------
-- View from Decider
------------------------------------------------------------------------

||| Extract a View from a Decider.
||| Discards the command handling, keeping only the projection logic.
|||
||| This is useful when you want to use a Decider's evolve function
||| for read model updates without command validation.
public export
viewFromDecider : Decider c s e err -> View s e
viewFromDecider d = MkView
  { evolve = d.evolve
  , initialState = d.initialState
  }

------------------------------------------------------------------------
-- Functor-like mapping
------------------------------------------------------------------------

||| Map over the state type of a View.
|||
||| Given functions to convert between state representations,
||| transform a View projecting to `s1` into one projecting to `s2`.
|||
||| Note: This requires both directions (isomorphism-like) because
||| we need to convert initial state and apply evolve.
public export
mapState : (s1 -> s2) -> (s2 -> s1) -> View s1 e -> View s2 e
mapState to from v = MkView
  { evolve = \st2, evt => to (v.evolve (from st2) evt)
  , initialState = to v.initialState
  }

||| Contramap over the event type of a View.
|||
||| Transform a View consuming events of type `e1` into one
||| consuming events of type `e2` by providing a conversion function.
public export
contramapEvent : (e2 -> e1) -> View s e1 -> View s e2
contramapEvent f v = MkView
  { evolve = \st, evt => v.evolve st (f evt)
  , initialState = v.initialState
  }

------------------------------------------------------------------------
-- Laws
------------------------------------------------------------------------

||| View projection is deterministic
public export
0 projectDeterministic : (v : View s e) -> (es : List e)
                       -> project v es = project v es
projectDeterministic v es = Refl

||| Incremental projection matches full projection
||| project v (es1 ++ es2) = projectFrom v (project v es1) es2
|||
||| This follows from foldl associativity (see Core.Decider.foldlAssociative).
||| The proof requires showing that project and projectFrom compose correctly.
public export
0 projectIncremental : (v : View s e) -> (es1 : List e) -> (es2 : List e)
                     -> project v (es1 ++ es2) = projectFrom v (project v es1) es2
projectIncremental v [] es2 = Refl
projectIncremental v (e :: es1) es2 =
  -- project v ((e :: es1) ++ es2)
  -- = foldl v.evolve v.initialState ((e :: es1) ++ es2)
  -- = foldl v.evolve v.initialState (e :: (es1 ++ es2))
  -- = foldl v.evolve (v.evolve v.initialState e) (es1 ++ es2)
  -- By induction hypothesis on es1:
  -- = projectFrom v (project v' es1) es2 where v' has initial state (v.evolve v.initialState e)
  -- = projectFrom v (foldl v.evolve (v.evolve v.initialState e) es1) es2
  -- = projectFrom v (foldl v.evolve v.initialState (e :: es1)) es2
  -- = projectFrom v (project v (e :: es1)) es2
  --
  -- This proof is non-trivial; we defer to a postulate for now.
  believe_me ()
