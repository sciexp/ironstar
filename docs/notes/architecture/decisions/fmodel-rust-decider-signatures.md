# fmodel-rust Decider pattern signatures for Idris2 formalization

## Type Signatures

### Core Decider struct

**Exact Rust signature:**

```rust
// From fmodel-rust/src/decider.rs:159-166
pub struct Decider<'a, C: 'a, S: 'a, E: 'a, Error: 'a = ()> {
    pub decide: DecideFunction<'a, C, S, E, Error>,
    pub evolve: EvolveFunction<'a, S, E>,
    pub initial_state: InitialStateFunction<'a, S>,
}
```

**Type aliases (default Send+Sync feature):**

```rust
// From fmodel-rust/src/lib.rs:332-342
pub type DecideFunction<'a, C, S, E, Error> =
    Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + 'a + Send + Sync>;

pub type EvolveFunction<'a, S, E> =
    Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>;

pub type InitialStateFunction<'a, S> =
    Box<dyn Fn() -> S + 'a + Send + Sync>;
```

**Abstract form for Idris2:**

```idris
-- Decider pattern as dependent record
record Decider (C : Type) (S : Type) (E : Type) (Error : Type) where
  constructor MkDecider
  decide        : C -> S -> Either Error (List E)
  evolve        : S -> E -> S
  initialState  : S
```

**Key observations for Idris2:**
- Rust's `Box<dyn Fn>` eliminates to simple function types in Idris2
- Lifetime parameter `'a` disappears (no explicit memory management)
- `Send + Sync` bounds disappear (all Idris2 values are pure)
- `Result<Vec<E>, Error>` maps to `Either Error (List E)`
- `InitialStateFunction<'a, S>` simplifies to constant `S` value

### EventComputation and StateComputation traits

**Exact Rust signatures:**

```rust
// From fmodel-rust/src/decider.rs:718-754
pub trait EventComputation<C, S, E, Error = ()> {
    fn compute_new_events(&self, current_events: &[E], command: &C) -> Result<Vec<E>, Error>;
}

pub trait StateComputation<C, S, E, Error = ()> {
    fn compute_new_state(&self, current_state: Option<S>, command: &C) -> Result<S, Error>;
}

// Implementation for Decider
impl<C, S, E, Error> EventComputation<C, S, E, Error> for Decider<'_, C, S, E, Error> {
    fn compute_new_events(&self, current_events: &[E], command: &C) -> Result<Vec<E>, Error> {
        let current_state: S = current_events
            .iter()
            .fold((self.initial_state)(), |state, event| {
                (self.evolve)(&state, event)
            });
        (self.decide)(command, &current_state)
    }
}

impl<C, S, E, Error> StateComputation<C, S, E, Error> for Decider<'_, C, S, E, Error> {
    fn compute_new_state(&self, current_state: Option<S>, command: &C) -> Result<S, Error> {
        let effective_current_state = current_state.unwrap_or_else(|| (self.initial_state)());
        let events = (self.decide)(command, &effective_current_state);
        events.map(|result| {
            result
                .into_iter()
                .fold(effective_current_state, |state, event| {
                    (self.evolve)(&state, &event)
                })
        })
    }
}
```

**Abstract form for Idris2:**

```idris
-- EventComputation: fold events to state, then decide new events
computeNewEvents : Decider C S E Error -> List E -> C -> Either Error (List E)
computeNewEvents decider currentEvents command =
  let currentState = foldl decider.evolve decider.initialState currentEvents
  in decider.decide command currentState

-- StateComputation: decide events from state, then fold to new state
computeNewState : Decider C S E Error -> Maybe S -> C -> Either Error S
computeNewState decider currentState command =
  let effectiveState = fromMaybe decider.initialState currentState
  in do events <- decider.decide command effectiveState
        pure (foldl decider.evolve effectiveState events)
```

**Key observations:**
- Both are derivable from the Decider triple (decide, evolve, initialState)
- EventComputation: reconstruct state from events before deciding
- StateComputation: apply new events to current state
- Both enforce the same Decider laws (purity, determinism)

### View struct

**Exact Rust signature:**

```rust
// From fmodel-rust/src/view.rs:86-91
pub struct View<'a, S: 'a, E: 'a> {
    pub evolve: EvolveFunction<'a, S, E>,
    pub initial_state: InitialStateFunction<'a, S>,
}
```

**Abstract form for Idris2:**

```idris
-- View pattern for projections (read side)
record View (S : Type) (E : Type) where
  constructor MkView
  evolve        : S -> E -> S
  initialState  : S
```

**Key observation:**
View is a Decider without the `decide` function (no command handling, only event folding for projections).

### Saga struct

**Exact Rust signature:**

```rust
// From fmodel-rust/src/saga.rs:84-87
pub struct Saga<'a, AR: 'a, A: 'a> {
    pub react: ReactFunction<'a, AR, A>,
}

// From fmodel-rust/src/lib.rs:342
pub type ReactFunction<'a, AR, A> = Box<dyn Fn(&AR) -> Vec<A> + 'a + Send + Sync>;
```

**Abstract form for Idris2:**

```idris
-- Saga pattern for process managers (choreography)
record Saga (AR : Type) (A : Type) where
  constructor MkSaga
  react : AR -> List A
```

**Key observation:**
Saga is the event-to-command mapper for process managers.
AR = ActionResult (often Event), A = Action (often Command).

### EventRepository trait

**Exact Rust signature:**

```rust
// From fmodel-rust/src/aggregate.rs:17-37
#[cfg(not(feature = "not-send-futures"))]
pub trait EventRepository<C, E, Version, Error> {
    fn fetch_events(
        &self,
        command: &C,
    ) -> impl Future<Output = Result<Vec<(E, Version)>, Error>> + Send;

    fn save(&self, events: &[E]) -> impl Future<Output = Result<Vec<(E, Version)>, Error>> + Send;

    fn version_provider(
        &self,
        event: &E,
    ) -> impl Future<Output = Result<Option<Version>, Error>> + Send;
}
```

**Abstract form for Idris2 (with IO effect):**

```idris
-- EventRepository interface (effectful operations)
interface EventRepository (repo : Type) (C : Type) (E : Type) (Version : Type) (Error : Type) where
  fetchEvents      : repo -> C -> IO (Either Error (List (E, Version)))
  save             : repo -> List E -> IO (Either Error (List (E, Version)))
  versionProvider  : repo -> E -> IO (Either Error (Maybe Version))
```

**Key observations:**
- This is the pure/effect boundary in fmodel-rust
- Rust's `impl Future` maps to Idris2's `IO` effect
- Repository operations are the ONLY effectful operations in fmodel architecture

### EventSourcedAggregate struct

**Exact Rust signature:**

```rust
// From fmodel-rust/src/aggregate.rs:78-86
pub struct EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
    Decider: EventComputation<C, S, E, Error>,
{
    repository: Repository,
    decider: Decider,
    _marker: PhantomData<(C, S, E, Version, Error)>,
}
```

**Abstract form for Idris2:**

```idris
-- EventSourcedAggregate: wiring pure Decider with effectful Repository
record EventSourcedAggregate (C : Type) (S : Type) (E : Type) (Version : Type) (Error : Type)
                             (repo : Type) where
  constructor MkAggregate
  repository : repo
  decider    : Decider C S E Error

  {auto repoImpl : EventRepository repo C E Version Error}
```

**Handle method signature:**

```rust
// From fmodel-rust/src/aggregate.rs:168-177
pub async fn handle(&self, command: &C) -> Result<Vec<(E, Version)>, Error> {
    let events: Vec<(E, Version)> = self.fetch_events(command).await?;
    let mut current_events: Vec<E> = vec![];
    for (event, _) in events {
        current_events.push(event);
    }
    let new_events = self.compute_new_events(&current_events, command)?;
    let saved_events = self.save(&new_events).await?;
    Ok(saved_events)
}
```

**Idris2 equivalent:**

```idris
handle : EventSourcedAggregate C S E Version Error repo -> C
      -> IO (Either Error (List (E, Version)))
handle agg command = do
  -- Fetch existing events (effectful)
  eventsWithVersion <- fetchEvents agg.repository command
  case eventsWithVersion of
    Left err => pure (Left err)
    Right evts => do
      let events = map fst evts  -- Drop versions
      -- Compute new events (pure)
      case computeNewEvents agg.decider events command of
        Left err => pure (Left err)
        Right newEvents => do
          -- Save new events (effectful)
          save agg.repository newEvents
```

**Key observations:**
- Only three I/O operations: fetchEvents, save, (implicitly versionProvider inside save)
- All decision logic is pure (computeNewEvents)
- Pattern: fetch → decide → save

## Invariants → Dependent Type Candidates

### Decider laws (from fmodel-rust implicit semantics)

These laws are NOT enforced by Rust's type system but are documented as contracts.
Idris2 can make these explicit as proof obligations.

**Law 1: decide is pure (referential transparency)**

```idris
-- For all commands c, states s
decideIsPure : (decider : Decider C S E Error) -> (c : C) -> (s : S)
            -> decider.decide c s = decider.decide c s
```

**Law 2: evolve cannot fail (events are historical facts)**

```idris
-- evolve is a total function (no Result/Either wrapper)
evolveIsTotal : (decider : Decider C S E Error) -> (s : S) -> (e : E) -> S
evolveIsTotal decider s e = decider.evolve s e  -- Always returns a state
```

**Law 3: State reconstruction (fold evolve over event stream)**

```idris
-- State can be reconstructed by folding evolve over events
stateReconstruction : (decider : Decider C S E Error) -> (events : List E)
                   -> foldl decider.evolve decider.initialState events = reconstructedState
```

**Law 4: Failure events preserve state (Law 6 from Hoffman)**

```idris
-- For failure events (e.g., CommandRejected), evolve returns state unchanged
failurePreservesState : (decider : Decider C S E Error) -> (s : S) -> (failureEvent : E)
                     -> IsFailureEvent failureEvent
                     -> decider.evolve s failureEvent = s
```

This requires a predicate `IsFailureEvent : E -> Type` to identify which events are failures.

**Law 5: Command idempotency (decide twice with same input yields same events)**

```idris
-- Applying same command twice produces same events (if state unchanged)
commandIdempotency : (decider : Decider C S E Error) -> (c : C) -> (s : S)
                  -> decider.decide c s = decider.decide c s
```

This is actually the same as Law 1 (purity), but emphasizes idempotency semantics.

**Law 6: Composition preserves laws**

```idris
-- Combining two Deciders produces a Decider that obeys the same laws
combinePreservesLaws : (d1 : Decider C1 S1 E1 Error)
                    -> (d2 : Decider C2 S2 E2 Error)
                    -> IsDecider (combine d1 d2)
```

This requires proving that combined Deciders still satisfy Laws 1-5.

### EventRepository laws

**Law 1: save is append-only (events are immutable)**

```idris
-- Saving events does not modify existing events
saveIsAppendOnly : (repo : repo) -> (events : List E)
                -> IO (Either Error (List (E, Version)))
                -> IO (Either Error ())  -- Verify existing events unchanged
```

This is harder to express in Idris2 without a state monad tracking the event store.

**Law 2: fetchEvents returns events in insertion order**

```idris
-- Events are returned in the order they were saved (monotonic Version)
fetchReturnsOrdered : (repo : repo) -> (c : C)
                   -> IO (Either Error (List (E, Version)))
                   -> All (\pair => fst pair <= snd pair) orderedPairs
```

Requires a proof that Version is monotonically increasing.

**Law 3: Optimistic locking via version_provider**

```idris
-- Concurrent saves fail if version conflicts detected
optimisticLocking : (repo : repo) -> (event : E) -> (expectedVersion : Maybe Version)
                 -> IO (Either Error Version)
                 -> Either ConcurrencyError Version
```

Requires modeling ConcurrencyError as a specific error type.

## Effect Boundaries

### Pure layer (Domain)

**Components:**
- `Decider<C, S, E, Error>`: All functions are pure (decide, evolve, initialState)
- `View<S, E>`: Pure event folding for projections
- `Saga<AR, A>`: Pure event-to-command mapping

**No I/O operations:**
- No database calls
- No network requests
- No system time (must be passed as command/event data)
- No random number generation (must be done in application layer)

**Properties:**
- Referential transparency: same inputs always produce same outputs
- Testable without mocks (all inputs are values)
- Deterministic replay: reconstructing state from events always yields same result

### Effectful layer (Application)

**Components:**
- `EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error>`: Wires Decider + Repository
- `EventRepository<C, E, Version, Error>`: Database I/O operations
- `StateRepository<C, S, Version, Error>`: State-stored aggregate variant

**I/O operations:**
- `fetch_events`: Read events from database
- `save`: Write events to database (append-only)
- `version_provider`: Read current version for optimistic locking

**Pattern:**
```rust
// All effect boundaries are explicit in type signatures
pub async fn handle(&self, command: &C) -> Result<Vec<(E, Version)>, Error> {
    // I/O: fetch
    let events = self.fetch_events(command).await?;

    // Pure: decide
    let new_events = self.compute_new_events(&events, command)?;

    // I/O: save
    self.save(&new_events).await?;

    Ok(new_events)
}
```

**Idris2 effect type:**
```idris
-- EventRepository operations use IO effect
-- Decider operations are pure (no effect wrapper)

handle : EventSourcedAggregate -> C -> IO (Either Error (List (E, Version)))
```

**Alternative: Custom Effect type for more granular control**

```idris
-- Define custom Event Sourcing effect
data ESEffect : Type -> Type where
  FetchEvents : C -> ESEffect (List (E, Version))
  SaveEvents  : List E -> ESEffect (List (E, Version))
  GetVersion  : E -> ESEffect (Maybe Version)

-- Handle function with explicit effect
handle : EventSourcedAggregate -> C -> Eff (Either Error (List (E, Version))) [ES]
```

This allows more precise tracking of which operations are effectful.

## Composition Patterns

### combine(): Merge independent Deciders (different C/E types)

**Exact Rust signature:**

```rust
// From fmodel-rust/src/decider.rs:381-436
pub fn combine<C2, S2, E2>(
    self,
    decider2: Decider<'a, C2, S2, E2, Error>,
) -> Decider<'a, Sum<C, C2>, (S, S2), Sum<E, E2>, Error>
where
    S: Clone,
    S2: Clone,
```

**Implementation pattern:**

```rust
let new_decide = Box::new(move |c: &Sum<C, C2>, s: &(S, S2)| match c {
    Sum::First(c) => {
        let s1 = &s.0;
        let events = (self.decide)(c, s1);
        events.map(|result| {
            result.into_iter().map(|e: E| Sum::First(e)).collect::<Vec<Sum<E, E2>>>()
        })
    }
    Sum::Second(c) => {
        let s2 = &s.1;
        let events = (decider2.decide)(c, s2);
        events.map(|result| {
            result.into_iter().map(|e: E2| Sum::Second(e)).collect::<Vec<Sum<E, E2>>>()
        })
    }
});

let new_evolve = Box::new(move |s: &(S, S2), e: &Sum<E, E2>| match e {
    Sum::First(e) => {
        let s1 = &s.0;
        let new_state = (self.evolve)(s1, e);
        (new_state, s.1.to_owned())
    }
    Sum::Second(e) => {
        let s2 = &s.1;
        let new_state = (decider2.evolve)(s2, e);
        (s.0.to_owned(), new_state)
    }
});
```

**Idris2 equivalent:**

```idris
data Sum a b = First a | Second b

combine : Decider C1 S1 E1 Error -> Decider C2 S2 E2 Error
       -> Decider (Sum C1 C2) (S1, S2) (Sum E1 E2) Error
combine d1 d2 = MkDecider
  { decide = \c, (s1, s2) => case c of
      First c1  => map (map First) (d1.decide c1 s1)
      Second c2 => map (map Second) (d2.decide c2 s2)
  , evolve = \(s1, s2), e => case e of
      First e1  => (d1.evolve s1 e1, s2)
      Second e2 => (s1, d2.evolve s2 e2)
  , initialState = (d1.initialState, d2.initialState)
  }
```

**Key observations:**
- Product state `(S1, S2)`: both deciders maintain independent state
- Sum command `Sum C1 C2`: route command to appropriate decider
- Sum event `Sum E1 E2`: route event to appropriate decider
- No interference between deciders (state isolation)

### merge(): Merge Deciders with same Event type

**Exact Rust signature:**

```rust
// Not present in Decider; this is View-specific pattern
// View::merge<S2> allows multiple views to subscribe to same event stream
pub fn merge<S2>(self, view2: View<'a, S2, E>) -> View<'a, (S, S2), E>
where
    S: Clone,
    S2: Clone,
```

**Key difference from combine:**
- `combine()`: Different event types (Sum E1 E2) → independent aggregates
- `merge()`: Same event type (E) → multiple projections of same event stream

**Use case:**
Multiple read models (Views) subscribe to same event stream but maintain different projections.

**Idris2 equivalent:**

```idris
merge : View S1 E -> View S2 E -> View (S1, S2) E
merge v1 v2 = MkView
  { evolve = \(s1, s2), e => (v1.evolve s1 e, v2.evolve s2 e)
  , initialState = (v1.initialState, v2.initialState)
  }
```

### Saga composition

**Exact Rust signature:**

```rust
// From fmodel-rust/src/saga.rs:175-190
pub fn merge<A2>(self, saga2: Saga<'a, AR, A2>) -> Saga<'a, AR, Sum<A2, A>> {
    let new_react = Box::new(move |ar: &AR| {
        let a: Vec<Sum<A2, A>> = (self.react)(ar)
            .into_iter()
            .map(|a: A| Sum::Second(a))
            .collect();
        let a2: Vec<Sum<A2, A>> = (saga2.react)(ar)
            .into_iter()
            .map(|a2: A2| Sum::First(a2))
            .collect();

        a.into_iter().chain(a2).collect()
    });

    Saga { react: new_react }
}
```

**Idris2 equivalent:**

```idris
mergeSaga : Saga AR A1 -> Saga AR A2 -> Saga AR (Sum A1 A2)
mergeSaga s1 s2 = MkSaga
  { react = \ar => map Second (s1.react ar) ++ map First (s2.react ar)
  }
```

**Key observation:**
Both sagas react to the same action result (AR), producing combined list of actions.
This enables process managers that coordinate multiple aggregates.

## Cross-Reference Findings

### Discrepancies between fmodel-rust and ironstar adoption evaluation

**1. Version tracking mechanism**

- **fmodel-rust implementation (aggregate.rs:33-36):**
  ```rust
  fn version_provider(
      &self,
      event: &E,
  ) -> impl Future<Output = Result<Option<Version>, Error>> + Send;
  ```

- **ironstar adoption evaluation (line 135-154):**
  Uses `previous_id UUID REFERENCES events(event_id)` for optimistic locking.

- **Actual fmodel-rust-postgres schema:**
  MISSING from source code analysis.
  The `version_provider` trait suggests Version could be any type.

**Actionable:** Verify fmodel-rust-postgres actual schema to confirm if it uses `previous_id` or numeric sequence.

**2. Error handling in decide function**

- **fmodel-rust signature:** `Result<Vec<E>, Error>`
- **ironstar docs (event-sourcing-core.md:56-59):** Discusses "failure events" vs "command errors"

**Question:** Should ironstar always return `Ok(vec![FailureEvent])` for business rule violations, or use `Err(ValidationError)` for pre-conditions?

**fmodel-rust pattern:**
```rust
// From decider.rs example (lines 38-39)
if state.order_id == update_cmd.order_id {
    Ok(vec![OrderEvent::Updated(...)])
} else {
    Ok(vec![])  // Empty vec, NOT Err()
}
```

fmodel-rust idiom: return empty event list for invalid commands, not errors.
Errors are for exceptional conditions (database failure, network timeout), not business rule violations.

**Actionable:** Clarify ironstar's error handling strategy: empty vec vs failure event vs Err().

**3. Identifier trait usage**

- **fmodel-rust (lib.rs:468-473):**
  ```rust
  pub trait Identifier {
      fn identifier(&self) -> String;
  }
  ```

- **ironstar adoption evaluation:** Does NOT mention Identifier trait.

- **Usage in fmodel-rust (aggregate.rs:499, 553):**
  Used in `EventSourcedOrchestratingAggregate` to filter events by aggregate ID.

**Actionable:** Determine if ironstar needs `Identifier` trait for multi-aggregate coordination.
If using Zenoh key expressions (e.g., `events/Todo/{todo_id}`), Identifier might be redundant.

**4. StateStoredAggregate vs EventSourcedAggregate**

- **fmodel-rust provides both:**
  - `EventSourcedAggregate` (events as source of truth)
  - `StateStoredAggregate` (state snapshots in database)

- **ironstar adoption evaluation (line 6):**
  "SQLite is fully compatible with fmodel-rust's EventRepository trait"

**Question:** Does ironstar plan to use StateStoredAggregate for any aggregates, or pure event sourcing only?

**Actionable:** Clarify ironstar's stance on state-stored aggregates (likely not needed, but worth documenting).

### Patterns ironstar docs missed

**1. combine3, combine4, combine5, combine6 convenience methods**

fmodel-rust provides `combine3()` through `combine6()` that flatten nested Sum types:

```rust
// From decider.rs:439-482
pub fn combine3<C2, S2, E2, C3, S3, E3>(
    self,
    decider2: Decider<'a, C2, S2, E2, Error>,
    decider3: Decider<'a, C3, S3, E3, Error>,
) -> Decider3<'a, C, C2, C3, S, S2, S3, E, E2, E3, Error>
```

Type alias:
```rust
type Decider3<'a, C1, C2, C3, S1, S2, S3, E1, E2, E3, Error> =
    Decider<'a, Sum3<C1, C2, C3>, (S1, S2, S3), Sum3<E1, E2, E3>, Error>;
```

**Idris2 equivalent:**
```idris
data Sum3 a b c = First a | Second b | Third c

combine3 : Decider C1 S1 E1 Err -> Decider C2 S2 E2 Err -> Decider C3 S3 E3 Err
        -> Decider (Sum3 C1 C2 C3) (S1, S2, S3) (Sum3 E1 E2 E3) Err
```

**Actionable:** Document these convenience methods for multi-aggregate systems.

**2. not-send-futures feature flag**

fmodel-rust has dual implementations based on `Send + Sync` bounds:

```rust
#[cfg(not(feature = "not-send-futures"))]
pub type DecideFunction<'a, C, S, E, Error> =
    Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + 'a + Send + Sync>;

#[cfg(feature = "not-send-futures")]
pub type DecideFunction<'a, C, S, E, Error> =
    Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + 'a>;
```

**Actionable:** Document ironstar's choice (likely default `Send + Sync` for multi-threaded Tokio).

**3. EventSourcedOrchestratingAggregate**

Combines Decider + Saga for single-transaction multi-aggregate coordination:

```rust
// From aggregate.rs:417-425
pub struct EventSourcedOrchestratingAggregate<'a, C, S, E, Repository, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
{
    repository: Repository,
    decider: Decider<'a, C, S, E, Error>,
    saga: Saga<'a, E, C>,
    _marker: PhantomData<(C, S, E, Version, Error)>,
}
```

**Key method (aggregate.rs:517-566):**
```rust
async fn compute_new_events_dynamically(
    &self,
    current_events: &[E],
    command: &C,
) -> Result<Vec<E>, Error>
where
    E: Identifier,
    C: Identifier,
{
    // 1. Decide initial events
    let initial_events = (self.decider.decide)(command, &current_state)?;

    // 2. React with Saga to get new commands
    let commands: Vec<C> = initial_events
        .iter()
        .flat_map(|event: &E| self.saga.compute_new_actions(event))
        .collect();

    // 3. Recursively handle new commands
    for command in commands.iter() {
        let new_events = Box::pin(self.compute_new_events_dynamically(&previous_events, command)).await?;
        all_events.extend(new_events);
    }

    Ok(all_events)
}
```

**Actionable:** Determine if ironstar needs EventSourcedOrchestratingAggregate or if Zenoh event bus suffices for multi-aggregate coordination.

## Idris2 Formalization Questions

### 1. How to encode trait bounds as type parameters?

**Rust pattern:**
```rust
impl<C, S, E, Error> EventComputation<C, S, E, Error> for Decider<'_, C, S, E, Error>
where
    S: Clone,
    E: Clone,
```

**Idris2 options:**

**Option A: Implicit parameters (auto)**
```idris
computeNewEvents : {auto clonable : Clone S} -> Decider C S E Error -> List E -> C -> Either Error (List E)
```

**Option B: Explicit constraints in record**
```idris
record Decider (C : Type) (S : Type) (E : Type) (Error : Type) where
  constructor MkDecider
  decide        : C -> S -> Either Error (List E)
  evolve        : S -> E -> S
  initialState  : S

  stateCloneable : Clone S  -- Explicit constraint as field
```

**Option C: Constrained type alias**
```idris
CloneableDecider : (C : Type) -> (S : Type) -> (E : Type) -> (Error : Type) -> Type
CloneableDecider C S E Error = (d : Decider C S E Error ** Clone S)
```

**Recommendation:** Option A (implicit auto parameters) for most constraints.
Option C for constraints required by composition operators (combine, merge).

### 2. Effect type for EventSourcedAggregate?

**Option A: IO monad**
```idris
handle : EventSourcedAggregate C S E Version Error repo -> C -> IO (Either Error (List (E, Version)))
```

**Pros:** Simple, works with existing Idris2 IO primitives
**Cons:** Cannot distinguish different I/O operations (database vs network vs file)

**Option B: Custom effect with Eff monad**
```idris
data ESEffect : Type -> Type where
  FetchEvents : C -> ESEffect (List (E, Version))
  SaveEvents  : List E -> ESEffect (List (E, Version))

handle : EventSourcedAggregate C S E Version Error repo -> C
      -> Eff (Either Error (List (E, Version))) [ES]
```

**Pros:** Explicit effect tracking, can prove effect properties
**Cons:** More complex, requires Eff library

**Option C: Free monad**
```idris
data ESAction : Type -> Type where
  Fetch : C -> ESAction (List (E, Version))
  Save  : List E -> ESAction (List (E, Version))
  Pure  : a -> ESAction a
  Bind  : ESAction a -> (a -> ESAction b) -> ESAction b

handle : EventSourcedAggregate C S E Version Error repo -> C -> ESAction (Either Error (List (E, Version)))
```

**Pros:** Maximum control, can interpret/test without I/O
**Cons:** Most complex implementation

**Recommendation:** Start with Option A (IO), upgrade to Option B (Eff) if proof obligations require effect tracking.

### 3. Proof obligations for Decider laws?

**Law 1: decide is pure (referential transparency)**

```idris
decideIsPure : (d : Decider C S E Error) -> (c : C) -> (s : S)
            -> d.decide c s = d.decide c s
```

This is trivially true (reflexivity), but the real property is:

```idris
decideReferentiallyTransparent : (d : Decider C S E Error) -> (c : C) -> (s1 : S) -> (s2 : S)
                               -> s1 = s2
                               -> d.decide c s1 = d.decide c s2
```

**Challenge:** How to enforce this for user-defined Deciders?

**Option A: Trusted axiom (postulate)**
```idris
postulate decideIsPure : (d : Decider C S E Error) -> (c : C) -> (s1 : S) -> (s2 : S)
                      -> s1 = s2 -> d.decide c s1 = d.decide c s2
```

**Option B: Proof-carrying Decider**
```idris
record VerifiedDecider (C : Type) (S : Type) (E : Type) (Error : Type) where
  constructor MkVerifiedDecider
  decider : Decider C S E Error
  pureProof : (c : C) -> (s1 : S) -> (s2 : S) -> s1 = s2 -> decider.decide c s1 = decider.decide c s2
```

**Recommendation:** Start with Option A (postulate), upgrade to Option B for critical aggregates.

**Law 3: State reconstruction**

```idris
stateReconstruction : (d : Decider C S E Error) -> (events : List E)
                   -> foldl d.evolve d.initialState events = reconstructedState
```

**Challenge:** `reconstructedState` is not a known value; this is more an equality property.

**Better formulation:**
```idris
-- Applying same events in any order yields same state (if evolve is commutative)
stateReconstructionCommutative : (d : Decider C S E Error) -> (e1 : E) -> (e2 : E) -> (s : S)
                              -> d.evolve (d.evolve s e1) e2 = d.evolve (d.evolve s e2) e1
```

**Caveat:** This is NOT true for most aggregates (event order matters).

**Correct property:**
```idris
-- Folding events is deterministic
foldDeterministic : (d : Decider C S E Error) -> (events : List E)
                 -> foldl d.evolve d.initialState events
                  = foldl d.evolve d.initialState events
```

Again trivial (reflexivity). The real property:

```idris
-- Replaying same event stream always yields same state
replayDeterministic : (d : Decider C S E Error) -> (events1 : List E) -> (events2 : List E)
                   -> events1 = events2
                   -> foldl d.evolve d.initialState events1
                    = foldl d.evolve d.initialState events2
```

**Recommendation:** Document these properties as postconditions, prove for specific aggregates as needed.

### 4. Modeling Version for optimistic locking?

**Rust trait:**
```rust
fn version_provider(&self, event: &E) -> impl Future<Output = Result<Option<Version>, Error>> + Send;
```

**Idris2 encoding options:**

**Option A: Generic Version type**
```idris
interface EventRepository (repo : Type) (C : Type) (E : Type) (Version : Type) (Error : Type) where
  versionProvider : repo -> E -> IO (Either Error (Maybe Version))

  {auto versionOrd : Ord Version}  -- Version must be orderable for conflict detection
```

**Option B: Concrete Version type**
```idris
data Version = MkVersion Nat  -- Monotonic counter
             | UUIDVersion UUID  -- previous_id chain

interface EventRepository (repo : Type) (C : Type) (E : Type) (Error : Type) where
  versionProvider : repo -> E -> IO (Either Error (Maybe Version))
```

**Option C: Dependent type with proof**
```idris
data Version : Type where
  MkVersion : (n : Nat) -> (proof : IsMonotonic n) -> Version

IsMonotonic : Nat -> Type
IsMonotonic n = (m : Nat) -> m < n -> Void  -- No version less than n exists
```

**Recommendation:** Start with Option B (concrete Version type), upgrade to Option C if optimistic locking proofs required.

### 5. Modeling Sum types with pattern matching exhaustiveness?

**Rust:**
```rust
match c {
    Sum::First(c1) => ...,
    Sum::Second(c2) => ...,
}
// Compiler enforces exhaustiveness
```

**Idris2:**
```idris
case c of
  First c1 => ...
  Second c2 => ...
-- Totality checker enforces exhaustiveness
```

**No encoding issue:** Idris2's totality checker automatically enforces exhaustive pattern matching.

**Bonus:** Idris2 can prove at compile-time that all cases are handled, whereas Rust only warns.

### Summary of formalization approach

1. **Core Decider:** Dependent record with three fields (decide, evolve, initialState)
2. **Trait bounds:** Implicit auto parameters (e.g., `{auto clonable : Clone S}`)
3. **Effects:** Start with IO monad, upgrade to Eff if needed
4. **Laws:** Postulates for purity/determinism, prove for critical aggregates
5. **Version:** Concrete type (Nat or UUID) with Ord constraint
6. **Exhaustiveness:** Rely on Idris2 totality checker

## Next Steps for .17 (Idris2 Decider Formalization)

1. Define `Decider` record type with three fields
2. Prove `computeNewEvents` and `computeNewState` derivable from Decider triple
3. Define `combine` operator and prove it preserves Decider laws
4. Define EventRepository interface with IO effect
5. Implement example Todo aggregate in Idris2
6. Write property-based tests using QuickCheck-style generators
