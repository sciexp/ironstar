//! Todo aggregate implementation.
//!
//! The Todo aggregate manages the lifecycle of a single todo item.
//! It demonstrates the core event sourcing patterns:
//!
//! - Pure command handling with validation
//! - State derived from event stream
//! - Type-safe transitions via the Aggregate trait
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
//!                      Delete
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

pub mod aggregate;
pub mod commands;
pub mod errors;
pub mod events;
pub mod state;
pub mod values;

// Re-export public types for ergonomic imports
pub use aggregate::TodoAggregate;
pub use commands::TodoCommand;
pub use errors::TodoError;
pub use events::TodoEvent;
pub use state::{TodoState, TodoStatus};
pub use values::{TODO_TEXT_MAX_LENGTH, TodoId, TodoText};
