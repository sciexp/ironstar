//! Application layer - command handlers, query handlers, projections.
//!
//! This layer orchestrates interactions between the pure domain logic and
//! effectful infrastructure services. It is the **effect boundary** where
//! async I/O meets sync computation.
//!
//! # Async/sync boundary
//!
//! The application layer is where asynchronous infrastructure calls
//! (database reads, event publishing, cache operations) wrap synchronous
//! domain logic (aggregate decisions, state transitions, validations).
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  Presentation Layer (async HTTP handlers)                       │
//! │      │                                                          │
//! │      ▼                                                          │
//! │  ┌─────────────────────────────────────────────────────────────┐│
//! │  │  Application Layer (async orchestration)                    ││
//! │  │      │                                                      ││
//! │  │      ├── Load state from infrastructure (async)             ││
//! │  │      ├── Call domain logic (SYNC - pure functions)          ││
//! │  │      └── Persist results via infrastructure (async)         ││
//! │  └─────────────────────────────────────────────────────────────┘│
//! │      │                     │                                    │
//! │      ▼                     ▼                                    │
//! │  Domain Layer          Infrastructure Layer                     │
//! │  (sync, pure)          (async, effectful)                       │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Design principle
//!
//! Functions in this module are `async` because they coordinate I/O, but
//! the core business logic they invoke is synchronous. This separation:
//!
//! - Keeps domain logic testable without async runtimes or mocks
//! - Makes effects explicit in type signatures (`async fn` = performs I/O)
//! - Enables local reasoning about pure functions in the domain layer
//!
//! # Command handler pattern
//!
//! A typical command handler follows this flow:
//!
//! ```rust,ignore
//! pub async fn handle_create_todo<S: HasEventStore + HasEventBus>(
//!     services: &S,
//!     command: TodoCommand,
//! ) -> Result<Vec<TodoEvent>, AppError> {
//!     // 1. Load current state (async - infrastructure I/O)
//!     let events = services.event_store().load_stream(aggregate_id).await?;
//!
//!     // 2. Reconstruct state (sync - pure fold)
//!     let state = events.iter().fold(TodoState::default(), |s, e| s.apply(e));
//!
//!     // 3. Execute domain logic (sync - pure decision)
//!     let new_events = TodoAggregate::handle(&state, command)?;
//!
//!     // 4. Persist events (async - infrastructure I/O)
//!     services.event_store().append(&new_events).await?;
//!
//!     // 5. Publish for projections (async - infrastructure I/O)
//!     services.event_bus().publish(&new_events).await?;
//!
//!     Ok(new_events)
//! }
//! ```
//!
//! Steps 2 and 3 are synchronous calls to pure domain functions.
//! Steps 1, 4, and 5 are async calls to infrastructure services.
//!
//! # Query handler pattern
//!
//! Query handlers read from projections (optimized read models) rather
//! than reconstructing state from events:
//!
//! ```rust,ignore
//! pub async fn get_todo_list<S: HasProjectionStore>(
//!     services: &S,
//!     filter: TodoFilter,
//! ) -> Result<Vec<TodoItemView>, AppError> {
//!     // Read from projection (async - infrastructure I/O)
//!     services.projection_store().query_todos(filter).await
//! }
//! ```
//!
//! # What belongs here
//!
//! - Command handlers that orchestrate domain operations
//! - Query handlers that read from projections
//! - Projection updaters that subscribe to events and update read models
//! - Application-level error types that map domain/infrastructure errors
//!
//! # What does NOT belong here
//!
//! - Domain logic (belongs in [`crate::domain`])
//! - Database queries or HTTP calls (belongs in [`crate::infrastructure`])
//! - HTTP routing or request parsing (belongs in [`crate::presentation`])
//! - Async primitives in domain types (domain must remain sync)

pub mod error;
pub mod query_session;
pub mod todo;

pub use error::{AggregateError, CommandPipelineError};
pub use query_session::{
    QueryExecutionParams, handle_query_session_command, handle_query_session_command_zenoh,
    spawn_query_execution,
};
pub use todo::{handle_todo_command, query_all_todos, query_todo_state};
