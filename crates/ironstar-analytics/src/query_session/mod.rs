//! QuerySession Decider module.
//!
//! The QuerySession Decider manages the lifecycle of an analytics query session
//! using the fmodel-rust Decider pattern.

mod commands;
mod decider;
pub mod errors;
mod events;
mod state;

// Re-export all public types
pub use commands::QuerySessionCommand;
pub use decider::{QuerySessionDecider, query_session_decider};
pub use errors::{QuerySessionError, QuerySessionErrorKind};
pub use events::QuerySessionEvent;
pub use state::{QuerySessionState, QuerySessionStatus};
