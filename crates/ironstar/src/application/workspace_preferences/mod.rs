//! WorkspacePreferences aggregate application layer.
//!
//! This module wires the WorkspacePreferences Decider to the SQLite event
//! repository, providing command handling for workspace preference management.

mod handlers;

pub use handlers::{
    handle_workspace_preferences_command, handle_workspace_preferences_command_zenoh,
};
