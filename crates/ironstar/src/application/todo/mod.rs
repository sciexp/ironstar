//! Todo aggregate application layer.
//!
//! This module wires the Todo Decider to the SQLite event repository,
//! creating a complete EventSourcedAggregate for command handling.

mod handlers;

pub use handlers::handle_todo_command;
