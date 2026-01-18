//! Todo domain implementation using fmodel-rust Decider pattern.
//!
//! The Todo decider is a pure function that embodies the state machine
//! for managing todo item lifecycles. It implements the patterns from
//! `spec/Todo/Todo.idr`.
//!
//! # State Machine
//!
//! ```text
//!                    ┌──────────┐
//!     Create ───────►│  Active  │◄────────┐
//!                    └────┬─────┘         │
//!                         │               │
//!                    Complete        Uncomplete
//!                         │               │
//!                         ▼               │
//!                    ┌──────────┐         │
//!                    │Completed │─────────┘
//!                    └────┬─────┘
//!                         │
//!                      Delete (also from Active)
//!                         │
//!                         ▼
//!                    ┌──────────┐
//!                    │ Deleted  │ (terminal)
//!                    └──────────┘
//! ```
//!
//! - `Active`: Initial state, can be completed or deleted
//! - `Completed`: Marked done, can be uncompleted or deleted
//! - `Deleted`: Terminal state, no further operations allowed
//!
//! # Idempotency
//!
//! Operations that would result in the same state return `Ok(vec![])`:
//! - Complete when already Completed
//! - Uncomplete when already Active
//! - Delete when already Deleted

pub mod commands;
pub mod decider;
pub mod errors;
pub mod events;
pub mod state;
pub mod values;

// Re-export public types for ergonomic imports
pub use commands::TodoCommand;
pub use decider::{TodoDecider, todo_decider};
pub use errors::{TodoError, TodoErrorKind};
pub use events::TodoEvent;
pub use state::{TodoState, TodoStatus};
pub use values::{TODO_TEXT_MAX_LENGTH, TodoId, TodoText};
