||| Saga pattern for process managers
|||
||| A Saga (process manager) maps action results (typically events) to new
||| actions (typically commands). This enables choreography between aggregates
||| without tight coupling.
|||
||| Key properties:
||| - Sagas are pure: given same event, produce same commands
||| - Sagas are stateless: each event processed independently
||| - Sagas enable eventual consistency across aggregate boundaries
|||
||| Use cases:
||| - Order fulfillment: OrderPlaced -> [ReserveInventory, ChargePayment]
||| - User onboarding: UserRegistered -> [SendWelcomeEmail, CreateDefaultSettings]
|||
||| Reference: ~/projects/rust-workspace/fmodel-rust/src/saga.rs
module Core.Saga

import Data.List
import Core.Decider

%default total

------------------------------------------------------------------------
-- Core Saga record
------------------------------------------------------------------------

||| The Saga record encapsulates pure event-to-command mapping.
|||
||| Type parameters:
||| - `ar` : ActionResult type (input, typically events)
||| - `a`  : Action type (output, typically commands)
|||
||| Design notes:
||| - AR (ActionResult) is often an Event type from an aggregate
||| - A (Action) is often a Command type for another aggregate
||| - Sagas are pure functions, no state maintained between calls
||| - Empty list return means "no action needed for this event"
public export
record Saga (ar : Type) (a : Type) where
  constructor MkSaga
  ||| React to an action result by producing zero or more actions.
  ||| Pure function: same input always produces same output.
  react : ar -> List a

------------------------------------------------------------------------
-- Derived computations
------------------------------------------------------------------------

||| Process multiple action results, collecting all resulting actions.
public export
reactAll : Saga ar a -> List ar -> List a
reactAll s actionResults = concatMap s.react actionResults

------------------------------------------------------------------------
-- Composition: merge (same action result, different action types)
------------------------------------------------------------------------

||| Merge two Sagas that react to the same action result type.
|||
||| Both sagas process the same event and their outputs are combined
||| into a Sum type. This enables multiple process managers to coordinate
||| different aspects of a workflow.
|||
||| Type transformation:
||| - ActionResult: `ar` (same input to both)
||| - Action: `Sum a1 a2` (combined output)
|||
||| Reference: fmodel-rust saga.rs:175-190
public export
merge : Saga ar a1 -> Saga ar a2 -> Saga ar (Sum a1 a2)
merge s1 s2 = MkSaga
  { react = \ar => map First (s1.react ar) ++ map Second (s2.react ar)
  }

||| Infix operator for saga merge
public export
(<|>) : Saga ar a1 -> Saga ar a2 -> Saga ar (Sum a1 a2)
(<|>) = merge

||| Merge three Sagas
public export
merge3 : Saga ar a1 -> Saga ar a2 -> Saga ar a3 -> Saga ar (Sum3 a1 a2 a3)
merge3 s1 s2 s3 = MkSaga
  { react = \ar =>
      map First3 (s1.react ar) ++
      map Second3 (s2.react ar) ++
      map Third3 (s3.react ar)
  }

------------------------------------------------------------------------
-- Composition: chain (sequential processing)
------------------------------------------------------------------------

||| Chain two Sagas where output of first feeds into second.
|||
||| This creates a pipeline: ar -> [a1] -> [[a2]] -> [a2]
||| Useful for multi-stage workflows.
|||
||| Note: This flattens the result (concatMap).
public export
chain : Saga ar a1 -> Saga a1 a2 -> Saga ar a2
chain s1 s2 = MkSaga
  { react = \ar => concatMap s2.react (s1.react ar)
  }

||| Infix operator for saga chaining
public export
(>=>) : Saga ar a1 -> Saga a1 a2 -> Saga ar a2
(>=>) = chain

------------------------------------------------------------------------
-- Functor-like operations
------------------------------------------------------------------------

||| Map over the action type of a Saga
public export
mapAction : (a1 -> a2) -> Saga ar a1 -> Saga ar a2
mapAction f s = MkSaga
  { react = \ar => map f (s.react ar)
  }

||| Functor instance for Saga (over action type)
public export
Functor (Saga ar) where
  map = mapAction

||| Contramap over the action result type of a Saga
public export
contramapResult : (ar2 -> ar1) -> Saga ar1 a -> Saga ar2 a
contramapResult f s = MkSaga
  { react = \ar2 => s.react (f ar2)
  }

------------------------------------------------------------------------
-- Identity and empty Sagas
------------------------------------------------------------------------

||| Identity Saga: passes through the input as output
||| Useful as a base case for composition
public export
identity : Saga a a
identity = MkSaga { react = \a => [a] }

||| Empty Saga: produces no actions for any input
||| Useful for optional/conditional saga composition
public export
empty : Saga ar a
empty = MkSaga { react = \_ => [] }

------------------------------------------------------------------------
-- Conditional Sagas
------------------------------------------------------------------------

||| Create a Saga that only reacts when a predicate holds
public export
filter : (ar -> Bool) -> Saga ar a -> Saga ar a
filter p s = MkSaga
  { react = \ar => if p ar then s.react ar else []
  }

||| Create a Saga from a partial function (Maybe-returning)
public export
fromMaybe : (ar -> Maybe a) -> Saga ar a
fromMaybe f = MkSaga
  { react = \ar => case f ar of
      Nothing => []
      Just a  => [a]
  }

||| Create a Saga from a partial function (List-returning with Maybe)
public export
fromMaybeList : (ar -> Maybe (List a)) -> Saga ar a
fromMaybeList f = MkSaga
  { react = \ar => case f ar of
      Nothing => []
      Just as => as
  }

------------------------------------------------------------------------
-- Laws
------------------------------------------------------------------------

||| Saga react is pure
public export
0 reactIsPure : (s : Saga actionResult action) -> (x : actionResult)
              -> s.react x = s.react x
reactIsPure s x = Refl

||| Identity law: identity saga preserves input
public export
0 identityLaw : (x : a) -> Core.Saga.identity.react x = [x]
identityLaw x = Refl

||| Empty law: empty saga produces nothing
||| Note: The empty saga for any types ar and a always returns [].
public export
0 emptyLaw : {ar, a : Type} -> (x : ar) -> (Core.Saga.empty {ar} {a}).react x = []
emptyLaw x = Refl
