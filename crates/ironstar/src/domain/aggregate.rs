//! Aggregate trait and supporting types.
//!
//! Aggregates are the consistency boundaries in event sourcing. Each aggregate:
//!
//! - Encapsulates a cluster of related domain objects
//! - Enforces invariants across those objects
//! - Is loaded and persisted as a unit
//! - Has a unique identity (aggregate ID)
//!
//! # Purity invariant
//!
//! Aggregate functions are **pure**: synchronous, deterministic, no side effects.
//! This is enforced by the trait signature (no `async`, no `&mut self` that could
//! hide state).
//!
//! ```text
//! handle_command: (&State, Command) -> Result<Vec<Event>, Error>
//! apply_event:    (State, Event) -> State
//! ```
//!
//! All I/O (database, network, time) happens in the application layer *before*
//! calling the aggregate. The aggregate receives pre-validated, timestamped data.
//!
//! # State reconstruction
//!
//! Aggregate state is derived by folding events:
//!
//! ```text
//! state = events.fold(State::default(), |s, e| Aggregate::apply_event(s, e))
//! ```
//!
//! This means `apply_event` must be deterministic — the same events in the same
//! order always produce the same state.
//!
//! # Optimistic concurrency
//!
//! The `version()` method returns the number of events applied. When persisting
//! new events, the event store checks that the expected version matches the
//! stored version. If another command modified the aggregate concurrently,
//! the append fails and the command can be retried with fresh state.

use std::error::Error;

/// Pure synchronous aggregate with no side effects.
///
/// Aggregates are the write-side consistency boundaries in CQRS. They:
///
/// - Validate commands against current state
/// - Emit events that represent state changes
/// - Derive state from their event stream
///
/// # Associated Types
///
/// - `State`: The aggregate's internal state, derived from events
/// - `Command`: Requests to change state (may be rejected)
/// - `Event`: Facts that occurred (always accepted by `apply_event`)
/// - `Error`: Domain errors from validation failures
///
/// # Example
///
/// ```rust,ignore
/// impl Aggregate for TodoAggregate {
///     const NAME: &'static str = "Todo";
///     type State = TodoState;
///     type Command = TodoCommand;
///     type Event = TodoEvent;
///     type Error = TodoError;
///
///     fn handle_command(state: &Self::State, cmd: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
///         match cmd {
///             TodoCommand::Create { id, text } => {
///                 let text = TodoText::new(text)?; // Validation here
///                 Ok(vec![TodoEvent::Created { id, text, created_at: Utc::now() }])
///             }
///             // ...
///         }
///     }
///
///     fn apply_event(state: Self::State, event: Self::Event) -> Self::State {
///         // Pure state transition - no validation, no side effects
///     }
/// }
/// ```
pub trait Aggregate: Default + Send + Sync {
    /// Unique name identifying this aggregate type.
    ///
    /// Used in the event store's `aggregate_type` column to route events.
    /// **Changing this breaks the link between existing aggregates and their events.**
    const NAME: &'static str;

    /// Internal aggregate state, derived from events.
    ///
    /// This is separate from the aggregate itself to emphasize that state
    /// is computed, not stored. The aggregate struct may hold metadata
    /// (like version), while State holds the domain data.
    type State: Default + Clone + Send + Sync;

    /// Commands represent requests to change state.
    ///
    /// Commands are external input (from users, APIs, other systems).
    /// They may be rejected if validation fails.
    type Command;

    /// Events represent facts that occurred.
    ///
    /// Events are the output of successful command handling. Once emitted,
    /// they are immutable and form the source of truth.
    type Event: Clone;

    /// Domain errors from command validation.
    ///
    /// These represent business rule violations (not infrastructure errors).
    /// They should be informative enough for user-facing error messages.
    type Error: Error;

    /// Validate a command and emit events if valid.
    ///
    /// This is a **pure function**: no async, no I/O, no side effects.
    /// Any external data (timestamps, random IDs, API lookups) must be
    /// provided in the command or resolved before calling this.
    ///
    /// # Arguments
    ///
    /// - `state`: Current aggregate state (derived from prior events)
    /// - `cmd`: The command to process
    ///
    /// # Returns
    ///
    /// - `Ok(events)`: Zero or more events to persist and apply
    /// - `Err(error)`: Validation failed; no state change
    fn handle_command(
        state: &Self::State,
        cmd: Self::Command,
    ) -> Result<Vec<Self::Event>, Self::Error>;

    /// Apply an event to produce new state.
    ///
    /// This is a **pure function**: deterministic, no side effects.
    /// It must succeed for any valid event — if it panics, that's a
    /// programmer error (the event should not have been emitted).
    ///
    /// # Panics
    ///
    /// May panic if the event is invalid for the current state. This
    /// indicates a bug in `handle_command` (it emitted an invalid event).
    fn apply_event(state: Self::State, event: Self::Event) -> Self::State;

    /// Reconstruct state from an event stream.
    ///
    /// This is the fundamental event sourcing operation: fold events
    /// into state. The default implementation is correct; override only
    /// if you need custom logic (e.g., for snapshots).
    fn fold_events(events: impl IntoIterator<Item = Self::Event>) -> Self::State {
        events
            .into_iter()
            .fold(Self::State::default(), Self::apply_event)
    }
}

/// Wrapper that tracks aggregate state plus version.
///
/// This separates the pure `State` (domain data) from the `version`
/// (infrastructure concern for optimistic locking).
#[derive(Debug, Clone)]
pub struct AggregateRoot<A: Aggregate> {
    /// Current state derived from events.
    state: A::State,
    /// Number of events applied (for optimistic locking).
    version: u64,
}

impl<A: Aggregate> Default for AggregateRoot<A> {
    fn default() -> Self {
        Self {
            state: A::State::default(),
            version: 0,
        }
    }
}

impl<A: Aggregate> AggregateRoot<A> {
    /// Create a new aggregate root with default state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current state.
    #[must_use]
    pub fn state(&self) -> &A::State {
        &self.state
    }

    /// Get the current version (number of events applied).
    ///
    /// Used for optimistic locking when persisting new events.
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Apply an event, updating state and incrementing version.
    pub fn apply(&mut self, event: A::Event) {
        self.state = A::apply_event(self.state.clone(), event);
        self.version += 1;
    }

    /// Apply multiple events in order.
    pub fn apply_all(&mut self, events: impl IntoIterator<Item = A::Event>) {
        for event in events {
            self.apply(event);
        }
    }

    /// Handle a command against current state.
    ///
    /// This delegates to `A::handle_command` but does NOT apply the
    /// resulting events. The caller is responsible for persisting events
    /// and then calling `apply_all` on success.
    ///
    /// # Returns
    ///
    /// The events to persist, or an error if validation failed.
    pub fn handle(&self, cmd: A::Command) -> Result<Vec<A::Event>, A::Error> {
        A::handle_command(&self.state, cmd)
    }

    /// Reconstruct an aggregate from stored events.
    #[must_use]
    pub fn from_events(events: impl IntoIterator<Item = A::Event>) -> Self {
        let mut root = Self::new();
        root.apply_all(events);
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal test aggregate for trait verification
    #[derive(Default)]
    struct TestAggregate;

    #[derive(Default, Clone)]
    struct TestState {
        value: i32,
    }

    #[derive(Clone)]
    enum TestEvent {
        Incremented,
        Decremented,
    }

    enum TestCommand {
        Increment,
        Decrement,
    }

    #[derive(Debug)]
    struct TestError;

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test error")
        }
    }

    impl Error for TestError {}

    impl Aggregate for TestAggregate {
        const NAME: &'static str = "Test";
        type State = TestState;
        type Command = TestCommand;
        type Event = TestEvent;
        type Error = TestError;

        fn handle_command(
            _state: &Self::State,
            cmd: Self::Command,
        ) -> Result<Vec<Self::Event>, Self::Error> {
            match cmd {
                TestCommand::Increment => Ok(vec![TestEvent::Incremented]),
                TestCommand::Decrement => Ok(vec![TestEvent::Decremented]),
            }
        }

        fn apply_event(mut state: Self::State, event: Self::Event) -> Self::State {
            match event {
                TestEvent::Incremented => state.value += 1,
                TestEvent::Decremented => state.value -= 1,
            }
            state
        }
    }

    #[test]
    fn aggregate_root_starts_at_version_zero() {
        let root = AggregateRoot::<TestAggregate>::new();
        assert_eq!(root.version(), 0);
        assert_eq!(root.state().value, 0);
    }

    #[test]
    fn apply_increments_version() {
        let mut root = AggregateRoot::<TestAggregate>::new();
        root.apply(TestEvent::Incremented);

        assert_eq!(root.version(), 1);
        assert_eq!(root.state().value, 1);
    }

    #[test]
    fn apply_all_applies_in_order() {
        let mut root = AggregateRoot::<TestAggregate>::new();
        root.apply_all(vec![
            TestEvent::Incremented,
            TestEvent::Incremented,
            TestEvent::Decremented,
        ]);

        assert_eq!(root.version(), 3);
        assert_eq!(root.state().value, 1); // +1 +1 -1 = 1
    }

    #[test]
    fn from_events_reconstructs_state() {
        let root = AggregateRoot::<TestAggregate>::from_events(vec![
            TestEvent::Incremented,
            TestEvent::Incremented,
        ]);

        assert_eq!(root.version(), 2);
        assert_eq!(root.state().value, 2);
    }

    #[test]
    fn handle_returns_events_without_applying() {
        let root = AggregateRoot::<TestAggregate>::new();
        let events = root.handle(TestCommand::Increment).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(root.version(), 0); // Not applied yet
        assert_eq!(root.state().value, 0);
    }

    #[test]
    fn handle_decrement_emits_decremented_event() {
        let root = AggregateRoot::<TestAggregate>::new();
        let events = root.handle(TestCommand::Decrement).unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], TestEvent::Decremented));
    }

    #[test]
    fn fold_events_produces_correct_state() {
        let state = TestAggregate::fold_events(vec![
            TestEvent::Incremented,
            TestEvent::Incremented,
            TestEvent::Decremented,
        ]);

        assert_eq!(state.value, 1);
    }
}
