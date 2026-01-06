||| Core Decider pattern for functional event sourcing
|||
||| The Decider is the minimal algebraic interface for event-sourced aggregates,
||| based on Jérémie Chassaing's pattern as implemented in fmodel-rust.
|||
||| Algebraically, the Decider is a coalgebra-algebra adjoint pair:
||| - `decide` is a coalgebra: unfolds commands into events
||| - `evolve` is an algebra: folds events into state
|||
||| Key properties (enforced by types or stated as postulates):
||| - `decide` is pure (referential transparency)
||| - `evolve` is total (events are historical facts, cannot fail)
||| - State reconstruction is deterministic (replay same events = same state)
|||
||| Reference: ~/projects/rust-workspace/fmodel-rust/src/decider.rs
module Core.Decider

import Data.List
import Data.Maybe

%default total

------------------------------------------------------------------------
-- Sum type for combining independent command/event types
------------------------------------------------------------------------

||| Sum type for disjoint union (used in combine)
||| Equivalent to fmodel-rust's Sum<A, B>
public export
data Sum : Type -> Type -> Type where
  First  : a -> Sum a b
  Second : b -> Sum a b

||| Bifunctor instance for Sum
public export
Bifunctor Sum where
  bimap f g (First x)  = First (f x)
  bimap f g (Second y) = Second (g y)

||| Functor instance (right-biased)
public export
Functor (Sum a) where
  map f (First x)  = First x
  map f (Second y) = Second (f y)

------------------------------------------------------------------------
-- Core Decider record
------------------------------------------------------------------------

||| The Decider record encapsulates pure decision logic for an aggregate.
|||
||| Type parameters:
||| - `c`     : Command type (inputs that may produce events)
||| - `s`     : State type (current aggregate state)
||| - `e`     : Event type (immutable facts produced by commands)
||| - `err`   : Error type (validation failures, defaults to Unit)
|||
||| Design notes:
||| - Rust's `Box<dyn Fn>` wrappers are eliminated (Idris functions are values)
||| - Rust's lifetime `'a` is eliminated (no manual memory management)
||| - Rust's `Send + Sync` bounds are eliminated (pure functions are thread-safe)
||| - `Result<Vec<E>, Error>` becomes `Either err (List e)`
||| - `initialState` is a constant, not a thunk (pure = no lazy initialization needed)
public export
record Decider (c : Type) (s : Type) (e : Type) (err : Type) where
  constructor MkDecider
  ||| Decide what events to emit given a command and current state.
  ||| Returns Left for validation errors, Right for successful event list.
  ||| Empty list is valid (command accepted but no state change needed).
  decide : c -> s -> Either err (List e)

  ||| Evolve state by applying an event.
  ||| This function is total: events are historical facts that cannot fail.
  ||| The return type is S (not Either), enforcing totality at the type level.
  evolve : s -> e -> s

  ||| The initial state for a new aggregate (before any events).
  initialState : s

------------------------------------------------------------------------
-- Derived computations
------------------------------------------------------------------------

||| Reconstruct current state from an event history.
||| This is the fundamental catamorphism: fold events through evolve.
|||
||| Property: `reconstruct d es1 = reconstruct d es2` when `es1 = es2`
||| (deterministic replay - same events always yield same state)
public export
reconstruct : Decider c s e err -> List e -> s
reconstruct d events = foldl d.evolve d.initialState events

||| Compute new events given existing event history and a command.
||| Pattern: reconstruct state from events, then decide on new events.
|||
||| This matches fmodel-rust's `compute_new_events` (decider.rs:718-754)
public export
computeNewEvents : Decider c s e err -> List e -> c -> Either err (List e)
computeNewEvents d currentEvents command =
  let currentState = reconstruct d currentEvents
  in d.decide command currentState

||| Compute new state given optional current state and a command.
||| Pattern: decide events from state, then fold to compute new state.
|||
||| This matches fmodel-rust's `compute_new_state` (decider.rs:76-88)
public export
computeNewState : Decider c s e err -> Maybe s -> c -> Either err s
computeNewState d currentState command =
  let effectiveState = fromMaybe d.initialState currentState
  in do events <- d.decide command effectiveState
        pure (foldl d.evolve effectiveState events)

------------------------------------------------------------------------
-- Composition: combine (independent deciders)
------------------------------------------------------------------------

||| Combine two independent Deciders into one.
|||
||| This enables multi-aggregate systems where each aggregate handles
||| its own command/event types independently.
|||
||| Type transformation:
||| - Commands: `Sum c1 c2` (route to appropriate decider)
||| - States: `(s1, s2)` (product of independent states)
||| - Events: `Sum e1 e2` (tagged union of event types)
|||
||| Key property: no interference between deciders (state isolation).
||| The combined Decider preserves all laws of the component Deciders.
|||
||| Reference: fmodel-rust decider.rs:381-436
public export
combine : Decider c1 s1 e1 err
       -> Decider c2 s2 e2 err
       -> Decider (Sum c1 c2) (s1, s2) (Sum e1 e2) err
combine d1 d2 = MkDecider
  { decide = \cmd, (s1, s2) => case cmd of
      First c1  => map (map First) (d1.decide c1 s1)
      Second c2 => map (map Second) (d2.decide c2 s2)
  , evolve = \(s1, s2), evt => case evt of
      First e1  => (d1.evolve s1 e1, s2)
      Second e2 => (s1, d2.evolve s2 e2)
  , initialState = (d1.initialState, d2.initialState)
  }

||| Infix operator for combine
public export
(<+>) : Decider c1 s1 e1 err -> Decider c2 s2 e2 err
     -> Decider (Sum c1 c2) (s1, s2) (Sum e1 e2) err
(<+>) = combine

------------------------------------------------------------------------
-- Ternary sum for combine3
------------------------------------------------------------------------

||| Sum type for three alternatives
public export
data Sum3 : Type -> Type -> Type -> Type where
  First3  : a -> Sum3 a b c
  Second3 : b -> Sum3 a b c
  Third3  : c -> Sum3 a b c

||| Combine three independent Deciders.
||| Flattened version avoiding nested Sum types.
|||
||| Reference: fmodel-rust decider.rs:439-482
public export
combine3 : Decider c1 s1 e1 err
        -> Decider c2 s2 e2 err
        -> Decider c3 s3 e3 err
        -> Decider (Sum3 c1 c2 c3) (s1, s2, s3) (Sum3 e1 e2 e3) err
combine3 d1 d2 d3 = MkDecider
  { decide = \cmd, (s1, s2, s3) => case cmd of
      First3 c1  => map (map First3) (d1.decide c1 s1)
      Second3 c2 => map (map Second3) (d2.decide c2 s2)
      Third3 c3  => map (map Third3) (d3.decide c3 s3)
  , evolve = \(s1, s2, s3), evt => case evt of
      First3 e1  => (d1.evolve s1 e1, s2, s3)
      Second3 e2 => (s1, d2.evolve s2 e2, s3)
      Third3 e3  => (s1, s2, d3.evolve s3 e3)
  , initialState = (d1.initialState, d2.initialState, d3.initialState)
  }

------------------------------------------------------------------------
-- Laws (postulates for now, prove incrementally)
------------------------------------------------------------------------

||| Law 1: decide is pure (referential transparency)
||| For the same command and state, decide always returns the same result.
|||
||| Note: This is a postulate because we cannot prove purity for arbitrary
||| user-defined decide functions. The type system enforces no IO, but
||| cannot rule out non-determinism in the function definition itself.
||| Upgrade to VerifiedDecider for aggregates requiring proof.
public export
0 decideIsPure : (d : Decider c s e err) -> (cmd : c) -> (st : s)
              -> d.decide cmd st = d.decide cmd st
decideIsPure d cmd st = Refl

||| Law 2: evolve is deterministic
||| For the same state and event, evolve always returns the same result.
||| (Follows from function extensionality in Idris)
public export
0 evolveIsDeterministic : (d : Decider c s e err) -> (st : s) -> (evt : e)
                        -> d.evolve st evt = d.evolve st evt
evolveIsDeterministic d st evt = Refl

||| Law 3: State reconstruction is deterministic (replay invariant)
||| Replaying the same events always yields the same state.
||| This is the foundation of event sourcing correctness.
public export
0 replayDeterministic : (d : Decider c s e err) -> (es : List e)
                      -> reconstruct d es = reconstruct d es
replayDeterministic d es = Refl

------------------------------------------------------------------------
-- Fold associativity (enables snapshots)
------------------------------------------------------------------------

||| Lemma: folding over concatenated lists is associative.
||| This property enables snapshotting: we can store intermediate state
||| and resume from there rather than replaying all events.
|||
||| foldl f s (es1 ++ es2) = foldl f (foldl f s es1) es2
|||
||| This is provable from foldl's definition but we state it explicitly
||| for documentation purposes.
public export
0 foldlAssociative : (f : s -> e -> s) -> (init : s) -> (es1, es2 : List e)
                   -> foldl f init (es1 ++ es2) = foldl f (foldl f init es1) es2
foldlAssociative f init [] es2 = Refl
foldlAssociative f init (e :: es1) es2 = foldlAssociative f (f init e) es1 es2
