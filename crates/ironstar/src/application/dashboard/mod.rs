//! Dashboard aggregate application layer.
//!
//! This module wires the Dashboard Decider to the SQLite event repository,
//! providing command handling for dashboard lifecycle within workspaces.

mod handlers;

pub use handlers::{handle_dashboard_command, handle_dashboard_command_zenoh};
