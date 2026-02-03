//! Workspace aggregate application layer.
//!
//! This module wires the Workspace Decider to the SQLite event repository,
//! providing command handling for the workspace lifecycle.

mod handlers;

pub use handlers::{handle_workspace_command, handle_workspace_command_zenoh};
