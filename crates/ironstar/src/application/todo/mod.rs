//! Todo aggregate application layer.
//!
//! This module wires the Todo Decider and View to the SQLite event repository,
//! providing both command handling and query services.
//!
//! # Command handling
//!
//! The `handle_todo_command` function wires the Decider to the event repository,
//! creating an EventSourcedAggregate that processes commands and persists events.
//!
//! # Query handling
//!
//! The `query_todo_state` function fetches events and folds them through the View
//! to compute current state on demand (compute-on-demand pattern).

mod handlers;
mod queries;

pub use handlers::{handle_todo_command, handle_todo_command_zenoh};
pub use queries::{query_all_todos, query_todo_state};
