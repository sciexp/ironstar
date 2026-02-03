//! SavedQuery aggregate application layer.
//!
//! This module wires the SavedQuery Decider to the SQLite event repository,
//! providing command handling for saved query lifecycle within workspaces.

mod handlers;

pub use handlers::{handle_saved_query_command, handle_saved_query_command_zenoh};
