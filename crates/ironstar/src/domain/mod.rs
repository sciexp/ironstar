//! Domain layer - algebraic types and pure domain logic.
//!
//! This module contains the pure core of the application: types that
//! represent domain concepts and functions that implement business rules.
//! Everything here is synchronous, deterministic, and side-effect-free.
//!
//! # Module Organization
//!
//! - [`aggregate`]: The `Aggregate` trait and `AggregateRoot` wrapper
//! - [`analytics`]: Analytics value objects (QueryId, DatasetRef, SqlQuery, ChartConfig)
//! - [`query_session`]: QuerySession aggregate (commands, events, state, errors)
//! - [`todo`]: Todo aggregate (commands, events, state, values, errors)
//!
//! # Design Principles
//!
//! 1. **Parse, don't validate**: Value objects enforce invariants at
//!    construction time. If you have a `TodoText`, it's guaranteed valid.
//!
//! 2. **Make invalid states unrepresentable**: Sum types (enums) ensure
//!    only valid state combinations can exist.
//!
//! 3. **Pure functions at the core**: Aggregates are pure state machines.
//!    All I/O happens at boundaries (application/infrastructure layers).
//!
//! 4. **Effects at boundaries**: The async/sync boundary marks the effect
//!    boundary. Domain functions are sync; I/O functions are async.
//!
//! # Example
//!
//! ```rust,ignore
//! use ironstar::domain::{
//!     aggregate::AggregateRoot,
//!     TodoCommand,
//!     TodoAggregate,
//!     TodoId,
//! };
//!
//! // Create an aggregate root (tracks state + version)
//! let mut root = AggregateRoot::<TodoAggregate>::new();
//!
//! // Handle a command - returns events or error
//! let events = root.handle(TodoCommand::Create {
//!     id: TodoId::new(),
//!     text: "Buy groceries".to_string(),
//! })?;
//!
//! // Apply events to update state (normally done after persistence)
//! root.apply_all(events);
//!
//! assert!(root.state().is_active());
//! ```

pub mod aggregate;
pub mod analytics;
pub mod query_session;
pub mod todo;

// Re-export key types for ergonomic imports
pub use aggregate::{Aggregate, AggregateRoot};

// Todo re-exports (from todo/)
pub use todo::{
    TODO_TEXT_MAX_LENGTH, TodoAggregate, TodoCommand, TodoError, TodoEvent, TodoId, TodoState,
    TodoStatus, TodoText,
};

// Analytics re-exports
pub use analytics::{
    AnalyticsValidationError, ChartConfig, ChartType, DATASET_REF_MAX_LENGTH, DatasetRef, QueryId,
    SQL_QUERY_MAX_LENGTH, SqlQuery,
};

// QuerySession re-exports
pub use query_session::{
    QuerySessionAggregate, QuerySessionCommand, QuerySessionError, QuerySessionEvent,
    QuerySessionState, QuerySessionStatus,
};
