//! Workspace aggregate application layer.
//!
//! This module wires the Workspace Decider and View to the SQLite event repository,
//! providing both command handling and query services.

mod handlers;
mod queries;

pub use handlers::{handle_workspace_command, handle_workspace_command_zenoh};
pub use queries::{
    query_dashboard_layout, query_saved_query_list, query_user_preferences, query_workspace_list,
    query_workspaces_for_user,
};
