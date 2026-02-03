//! QuerySession aggregate application layer.
//!
//! This module wires the QuerySession Decider to the SQLite event repository,
//! providing command handling for the analytics query lifecycle.
//!
//! # Command handling
//!
//! The `handle_query_session_command` function wires the Decider to the event
//! repository, creating an EventSourcedAggregate that processes commands and
//! persists events.
//!
//! # Spawn-after-persist
//!
//! Unlike Todo (synchronous end-to-end), QuerySession uses the spawn-after-persist
//! pattern: after `QueryStarted` is persisted, background DuckDB execution is
//! spawned and completion/failure commands are issued back through the Decider.
//! See the `spawn` module for the async execution pattern.

mod handlers;

pub use handlers::{handle_query_session_command, handle_query_session_command_zenoh};
