//! UserPreferences aggregate application layer.
//!
//! This module wires the UserPreferences Decider to the SQLite event
//! repository, providing command handling for per-user preference management.

mod handlers;

pub use handlers::{handle_user_preferences_command, handle_user_preferences_command_zenoh};
