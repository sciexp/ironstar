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
//! - [`commands`]: Command types (requests to change state)
//! - [`errors`]: Domain error types for validation failures
//! - [`events`]: Event types (facts that occurred)
//! - [`values`]: Value objects with smart constructors
//! - [`todo`]: Todo aggregate implementation
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
//!     commands::TodoCommand,
//!     todo::TodoAggregate,
//!     values::TodoId,
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
pub mod commands;
pub mod errors;
pub mod events;
pub mod todo;
pub mod values;

// Re-export key types for ergonomic imports
pub use aggregate::{Aggregate, AggregateRoot};
pub use commands::TodoCommand;
pub use errors::TodoError;
pub use events::TodoEvent;
pub use todo::{TodoAggregate, TodoState, TodoStatus};
pub use values::{TodoId, TodoText, TODO_TEXT_MAX_LENGTH};

// Analytics re-exports
pub use analytics::{
    AnalyticsValidationError, ChartConfig, ChartType, DatasetRef, QueryId, SqlQuery,
    DATASET_REF_MAX_LENGTH, SQL_QUERY_MAX_LENGTH,
};
